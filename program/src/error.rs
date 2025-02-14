use pinocchio::program_error::ProgramError;

/// Errors that may be returned by the Attestation Service program.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AttestationServiceError {
  // Incorrect Credential account
  InvalidCredential,
  // Incorrect Schema account
  InvalidSchema,
}

impl From<AttestationServiceError> for ProgramError {
  fn from(e: AttestationServiceError) -> Self {
      ProgramError::Custom(e as u32)
  }
}