pub mod aieos;
pub mod crypto;
pub mod oidc;
pub mod soul;

pub use aieos::{AieosIdentity, load_aieos_identity, aieos_to_system_prompt, is_aieos_configured};
pub use soul::Soul;
