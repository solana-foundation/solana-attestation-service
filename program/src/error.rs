use codama::CodamaErrors;
use pinocchio::program_error::ProgramError;

/// Errors that may be returned by the Attestation Service program.
#[derive(Clone, Debug, Eq, PartialEq, CodamaErrors)]
pub enum AttestationServiceError {
    #[codama(error("Incorrect Credential account"))]
    InvalidCredential,
    #[codama(error("Incorrect Schema account"))]
    InvalidSchema,
    #[codama(error("Incorrect Attestation account"))]
    InvalidAttestation,
    #[codama(error("Authority was not found in Credential authorized_signatures"))]
    InvalidAuthority,
    #[codama(error("Incorrect Schema data type"))]
    InvalidSchemaDataType,
    #[codama(error("The signer is not one of the Credential's authorized signers"))]
    SignerNotAuthorized,
    #[codama(error("Attestation data does not conform to the Schema"))]
    InvalidAttestationData,
    #[codama(error("Incorrect Event Authority"))]
    InvalidEventAuthority,
    #[codama(error("Incorrect Mint"))]
    InvalidMint,
    #[codama(error("Incorrect Program Signer"))]
    InvalidProgramSigner,
    #[codama(error("Incorrect Token Account"))]
    InvalidTokenAccount,
    #[codama(error("Schema is paused"))]
    SchemaPaused,
}

impl From<AttestationServiceError> for ProgramError {
    fn from(e: AttestationServiceError) -> Self {
        ProgramError::Custom(e as u32)
    }
}
