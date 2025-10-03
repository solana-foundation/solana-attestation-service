use pinocchio::program_error::ProgramError;

/// Errors that may be returned by the Attestation Service program.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AttestationServiceError {
    // 0 Incorrect Credential account
    InvalidCredential,
    // 1 Incorrect Schema account
    InvalidSchema,
    // 2 Incorrect Attestation account
    InvalidAttestation,
    // 3 Authority was not found in Credential authorized_signatures
    InvalidAuthority,
    // 4 Incorrect Schema data type
    InvalidSchemaDataType,
    // 5 The signer is not one of the Credential's authorized signers
    SignerNotAuthorized,
    // 6 Attestation data des not conform to the Schema
    InvalidAttestationData,
    // 7 Incorrect Event Authority
    InvalidEventAuthority,
    // 8 Incorrect Mint
    InvalidMint,
    // 9 Incorrect Program Signer
    InvalidProgramSigner,
    // 10 Incorrect Token Account
    InvalidTokenAccount,
    // 11 Schema is paused
    SchemaPaused,
    // 12 Invalid Address Tree (not the allowed tree for compressed attestations)
    InvalidAddressTree,
}

impl From<AttestationServiceError> for ProgramError {
    fn from(e: AttestationServiceError) -> Self {
        ProgramError::Custom(e as u32)
    }
}
