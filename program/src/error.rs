use pinocchio::program_error::ProgramError;
use thiserror::Error;

/// Errors that may be returned by the Attestation Service program.
#[derive(Clone, Debug, Eq, PartialEq, Error)]
pub enum AttestationServiceError {
    #[error("Incorrect Credential account")]
    InvalidCredential,
    #[error("Incorrect Schema account")]
    InvalidSchema,
    #[error("Incorrect Attestation account")]
    InvalidAttestation,
    #[error("Authority was not found in Credential authorized_signatures")]
    InvalidAuthority,
    #[error("Incorrect Schema data type")]
    InvalidSchemaDataType,
    #[error("The signer is not one of the Credential's authorized signers")]
    SignerNotAuthorized,
    #[error("Attestation data does not conform to the Schema")]
    InvalidAttestationData,
    #[error("Incorrect Event Authority")]
    InvalidEventAuthority,
    #[error("Incorrect Mint")]
    InvalidMint,
    #[error("Incorrect Program Signer")]
    InvalidProgramSigner,
    #[error("Incorrect Token Account")]
    InvalidTokenAccount,
    #[error("Schema is paused")]
    SchemaPaused,
    #[error("Invalid Address Tree (not the allowed tree for compressed attestations)")]
    InvalidAddressTree,
}

impl From<AttestationServiceError> for ProgramError {
    fn from(e: AttestationServiceError) -> Self {
        ProgramError::Custom(e as u32)
    }
}
