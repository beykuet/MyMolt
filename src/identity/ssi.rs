// SPDX-License-Identifier: EUPL-1.2
// Copyright (c) 2026 Benjamin Küttner <benjamin.kuettner@icloud.com>
// Patent Pending — DE Gebrauchsmuster, filed 2026-02-23

use anyhow::{anyhow, Result};
use didkit::{ContextLoader, DID_METHODS, LinkedDataProofOptions, VerifiablePresentation};

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct VerificationResult {
    pub is_valid: bool,
    pub issuer_did: Option<String>,
    pub holder_did: Option<String>,
    pub errors: Vec<String>,
}

pub struct SSIGuardian;

impl SSIGuardian {
    /// Verifies a Verifiable Presentation (VP) string (JSON-LD).
    ///
    /// Performs real cryptographic verification using didkit's linked-data
    /// proof verification pipeline:
    ///   1. Parse the VP JSON-LD
    ///   2. Verify all proofs using DID resolution + LD signatures
    ///   3. Extract holder/issuer DIDs
    pub async fn verify_vp(vp_string: &str) -> Result<VerificationResult> {
        // 1. Parse VP
        let vp: VerifiablePresentation = serde_json::from_str(vp_string)
            .map_err(|e| anyhow!("Failed to parse VP JSON: {}", e))?;

        // 2. Resolve DIDs using the built-in resolver that covers did:key, did:web, etc.
        let resolver = &*DID_METHODS;

        // 3. Real cryptographic verification via didkit/ssi linked-data proofs
        let mut context_loader = ContextLoader::default();
        let ldp_options = LinkedDataProofOptions::default();

        let ssi_result = vp
            .verify(Some(ldp_options), resolver, &mut context_loader)
            .await;

        let is_valid = ssi_result.errors.is_empty();
        let mut errors: Vec<String> = ssi_result.errors;

        // Append any warnings as informational
        if !ssi_result.warnings.is_empty() {
            for w in &ssi_result.warnings {
                errors.push(format!("warning: {w}"));
            }
        }

        // 4. Extract DIDs
        let mut issuer_did = None;
        let mut holder_did = None;

        // Holder is usually a URI in the `holder` field
        if let Some(holder) = &vp.holder {
             holder_did = Some(holder.to_string());
        }

        // Issuer is inside the credential(s)
        if let Some(creds) = &vp.verifiable_credential {
            for cred in creds {
                 // cred is CredentialOrJWT
                 match cred {
                     didkit::ssi::vc::CredentialOrJWT::Credential(c) => {
                         if let Some(issuer) = &c.issuer {
                             issuer_did = Some(issuer.get_id());
                             break; 
                         }
                     },
                     didkit::ssi::vc::CredentialOrJWT::JWT(_) => {
                         errors.push("JWT credential extraction not yet implemented".into());
                     }
                 }
            }
        }

        Ok(VerificationResult {
            is_valid,
            issuer_did,
            holder_did,
            errors,
        })
    }
}
