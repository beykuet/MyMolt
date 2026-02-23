use anyhow::Result;
use image::{Rgba, RgbaImage};
use mymolt_core::security::workers::ImageWorker;
use tempfile::TempDir;

#[test]
fn test_image_redaction() -> Result<()> {
    // 1. Create a simple 100x100 white image
    let width = 100;
    let height = 100;
    let mut img = RgbaImage::new(width, height);
    for pixel in img.pixels_mut() {
        *pixel = Rgba([255, 255, 255, 255]); // White
    }

    let tmp = TempDir::new()?;
    let input_path = tmp.path().join("input.png");
    let output_path = tmp.path().join("output.png");

    img.save(&input_path)?;

    // 2. Redact a 10x10 square at (10, 10)
    let regions = vec![(10, 10, 10, 10)];

    // 3. Execute ImageWorker
    ImageWorker::redact(&input_path, &output_path, &regions)?;

    // 4. Verify output
    let result_img = image::open(&output_path)?.into_rgba8();

    // Check redacted region (should be black)
    let pixel = result_img.get_pixel(15, 15);
    assert_eq!(
        *pixel,
        Rgba([0, 0, 0, 255]),
        "Pixel in redacted region should be black"
    );

    // Check non-redacted region (should be white)
    let pixel = result_img.get_pixel(5, 5);
    assert_eq!(
        *pixel,
        Rgba([255, 255, 255, 255]),
        "Pixel outsde redacted region should be white"
    );

    Ok(())
}
