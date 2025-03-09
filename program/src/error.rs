use pinocchio::program_error::ProgramError;

/// Errors that may be returned by the Attestation Service program.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AttestationServiceError {
    // Incorrect Credential account
    InvalidCredential,
    // Incorrect Schema account
    InvalidSchema,
    // Incorrect Attestation account
    InvalidAttestation,
    // Authority was not found in Credential authorized_signatures
    InvalidAuthority,
    // Incorrect Schema data type
    InvalidSchemaDataType,
    // The signer is not one of the Credential's authorized signers
    SignerNotAuthorized,
    // Attestation data des not conform to the Schema
    InvalidAttestationData,
    // Incorrect Event Authority
    InvalidEventAuthority,
}

impl From<AttestationServiceError> for ProgramError {
    fn from(e: AttestationServiceError) -> Self {
        ProgramError::Custom(e as u32)
    }
}
