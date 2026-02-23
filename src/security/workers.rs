// SPDX-License-Identifier: EUPL-1.2
// Copyright (c) 2026 Benjamin Küttner <benjamin.kuettner@icloud.com>
// Patent Pending — DE Gebrauchsmuster, filed 2026-02-23

use anyhow::{Context, Result};
use image::{GenericImage, GenericImageView, ImageReader, Rgba};
use std::path::Path;
use std::process::Command;

/// Worker for surgical audio redaction (requires ffmpeg).
pub struct AudioWorker;

impl AudioWorker {
    /// Redact specific time intervals from an audio file by silencing them.
    ///
    /// # Arguments
    /// * `input` - Path to the source audio file.
    /// * `output` - Path where the redacted audio will be saved.
    /// * `intervals` - A list of (start_seconds, end_seconds) tuples to silence.
    pub fn redact(input: &Path, output: &Path, intervals: &[(f64, f64)]) -> Result<()> {
        // If no intervals to redact, just copy the file.
        if intervals.is_empty() {
            std::fs::copy(input, output)?;
            return Ok(());
        }

        // Verify ffmpeg exists
        if Command::new("ffmpeg").arg("-version").output().is_err() {
            anyhow::bail!(
                "ffmpeg not found. Audio redaction requires ffmpeg to be installed and in PATH."
            );
        }

        // Build the complex filter string for ffmpeg
        // Format: volume=0:enable='between(t,start,end)+between(t,start,end)...'
        let enables: Vec<String> = intervals
            .iter()
            .map(|(s, e)| format!("between(t,{:.3},{:.3})", s, e))
            .collect();
        let enable_str = enables.join("+");
        let filter = format!("volume=0:enable='{}'", enable_str);

        // Execute ffmpeg
        let status = Command::new("ffmpeg")
            .arg("-y") // Overwrite output file without asking
            .arg("-i")
            .arg(input)
            .arg("-af")
            .arg(&filter) // Apply audio filter
            .arg(output)
            .status()
            .context("Failed to execute ffmpeg command")?;

        if !status.success() {
            anyhow::bail!("ffmpeg process failed with status: {}", status);
        }

        Ok(())
    }
}

/// Worker for surgical image redaction (pure Rust).
pub struct ImageWorker;

impl ImageWorker {
    /// Redact specific rectangular regions from an image by blacking them out.
    ///
    /// # Arguments
    /// * `input` - Path to the source image file.
    /// * `output` - Path where the redacted image will be saved.
    /// * `regions` - A list of (x, y, width, height) tuples defining areas to redaction.
    pub fn redact(input: &Path, output: &Path, regions: &[(u32, u32, u32, u32)]) -> Result<()> {
        // If no regions to redact, just copy the file.
        if regions.is_empty() {
            std::fs::copy(input, output)?;
            return Ok(());
        }

        // Load the image
        let mut img = ImageReader::open(input)
            .with_context(|| format!("Failed to open input image: {:?}", input))?
            .decode()
            .context("Failed to decode image data")?;

        let (width, height) = img.dimensions();

        // Apply redaction (draw black rectangles)
        for &(rx, ry, w, h) in regions {
            // Iterate over pixels in the region
            for x in rx..(rx + w) {
                for y in ry..(ry + h) {
                    // Safe bounds check
                    if x < width && y < height {
                        // Set pixel to opaque black
                        img.put_pixel(x, y, Rgba([0, 0, 0, 255]));
                    }
                }
            }
        }

        // Save the result
        img.save(output)
            .with_context(|| format!("Failed to save redacted image to {:?}", output))?;

        Ok(())
    }
}
