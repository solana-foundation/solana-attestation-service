import {
  findAttestationMintPda,
  findAttestationPda,
  findCredentialPda,
  findEventAuthorityPda,
  findSasAuthorityPda,
  findSchemaMintPda,
  findSchemaPda,
} from "./generated/pdas";

export const ATTESTATION_SEED = "attestation";
export const CREDENTIAL_SEED = "credential";
export const SCHEMA_SEED = "schema";
export const EVENT_AUTHORITY_SEED = "__event_authority";
export const SAS_SEED = "sas";
export const SCHEMA_MINT_SEED = "schemaMint";
export const ATTESTATION_MINT_SEED = "attestationMint";

/**
 * @deprecated Use {@link findEventAuthorityPda}, which also returns the bump.
 */
export const deriveEventAuthorityAddress = async () =>
  (await findEventAuthorityPda())[0];

/**
 * @deprecated Use {@link findSasAuthorityPda}, which also returns the bump.
 */
export const deriveSasAuthorityAddress = async () =>
  (await findSasAuthorityPda())[0];

/**
 * @deprecated Use {@link findCredentialPda}.
 */
export const deriveCredentialPda = findCredentialPda;

/**
 * @deprecated Use {@link findSchemaPda}.
 */
export const deriveSchemaPda = findSchemaPda;

/**
 * @deprecated Use {@link findAttestationPda}.
 */
export const deriveAttestationPda = findAttestationPda;

/**
 * @deprecated Use {@link findSchemaMintPda}.
 */
export const deriveSchemaMintPda = findSchemaMintPda;

/**
 * @deprecated Use {@link findAttestationMintPda}.
 */
export const deriveAttestationMintPda = findAttestationMintPda;
