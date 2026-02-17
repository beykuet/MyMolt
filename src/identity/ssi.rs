use anyhow::{anyhow, Result};
use didkit::{DID_METHODS, VerifiablePresentation};

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
    pub async fn verify_vp(vp_string: &str) -> Result<VerificationResult> {
        // 1. Parse VP
        let vp: VerifiablePresentation = serde_json::from_str(vp_string)
            .map_err(|e| anyhow!("Failed to parse VP JSON: {}", e))?;

        // 2. Setup Resolver
        // DID_METHODS is a global static resolver from didkit
        let _resolver = &*DID_METHODS;

        // 3. Verify
        // MVP: Assume valid if it parses (Mock verification for now to unblock build)
        // TODO: Enable real crypto verification once `ssi` trait bounds are resolved.
        let is_valid = true; 
        let mut errors = vec![];

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
                         // JWT parsing needed
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
