import { Address, getProgramDerivedAddress } from "@solana/kit";
import { SOLANA_ATTESTATION_SERVICE_PROGRAM_ADDRESS } from "./generated/programs";

export const ATTESTATION_SEED = "attestation";
export const CREDENTIAL_SEED = "credential";
export const SCHEMA_SEED = "schema";
export const EVENT_AUTHORITY_SEED = "eventAuthority";
export const SAS_SEED = "sas";
export const SCHEMA_MINT_SEED = "schemaMint";
export const ATTESTATION_MINT_SEED = "attestationMint";

// Note: event authority and sas address could be constant, but
// to keep the SDK dynamic in the event the program ID changes we
// always derive the addresses.

/**
 * Derive the EVENT_AUTHORITY_ADDRESS.
 */
export const deriveEventAuthorityAddress = async () =>
  (
    await getProgramDerivedAddress({
      programAddress: SOLANA_ATTESTATION_SERVICE_PROGRAM_ADDRESS,
      seeds: [EVENT_AUTHORITY_SEED],
    })
  )[0];

/**
 * Derive the SAS_ADDRESS.
 */
export const deriveSasAuthorityAddress = async () =>
  (
    await getProgramDerivedAddress({
      programAddress: SOLANA_ATTESTATION_SERVICE_PROGRAM_ADDRESS,
      seeds: [SAS_SEED],
    })
  )[0];

/**
 * Derive a Credential (aka Issuer) PDA.
 * @param authority The Address that controls the Credential account.
 * @param name A name for the credential. NOTE that only the first 32 bytes
 * will be used for the PDA due to seed size limits.
 * @returns
 */
export const deriveCredentialPda = ({
  authority,
  name,
}: {
  authority: Address;
  name: string;
}) =>
  getProgramDerivedAddress({
    programAddress: SOLANA_ATTESTATION_SERVICE_PROGRAM_ADDRESS,
    seeds: [CREDENTIAL_SEED, authority, name],
  });

/**
 * Derive a Schema PDA.
 * @param credential The Credential that the Schema is associated with.
 * @param name A name for the schema. NOTE that only the first 32 bytes
 * will be used for the PDA due to seed size limits.
 * @param version The version number (up to 255) of the Schema.
 * @returns
 */
export const deriveSchemaPda = ({
  credential,
  name,
  version,
}: {
  credential: Address;
  name: string;
  version: number;
}) => {
  const versionSeed = Uint8Array.from([version]);
  return getProgramDerivedAddress({
    programAddress: SOLANA_ATTESTATION_SERVICE_PROGRAM_ADDRESS,
    seeds: [SCHEMA_SEED, credential, name, versionSeed],
  });
};

/**
 * Derive an Attestation PDA.
 * @param credential The Credential (aka Issuer) that controls the Attestation state.
 * @param schema The Schema that the Attestation adheres to.
 * @param nonce An Address that may either represent the Wallet the Attestation
 * is associated with OR a randomly generated Address to prevent PDA collision.
 * @returns
 */
export const deriveAttestationPda = ({
  credential,
  schema,
  nonce,
}: {
  credential: Address;
  schema: Address;
  nonce: Address;
}) =>
  getProgramDerivedAddress({
    programAddress: SOLANA_ATTESTATION_SERVICE_PROGRAM_ADDRESS,
    seeds: [ATTESTATION_SEED, credential, schema, nonce],
  });

/* PDAs for tokenization */

/**
 * Derive the Token2022 Mint address of the tokenized Schema.
 * @param schema The Schema that the token belongs to.
 * @returns
 */
export const deriveSchemaMintPda = ({ schema }: { schema: Address }) =>
  getProgramDerivedAddress({
    programAddress: SOLANA_ATTESTATION_SERVICE_PROGRAM_ADDRESS,
    seeds: [SCHEMA_MINT_SEED, schema],
  });

/**
 * Derive the Token2022 Mint address of the tokenized Attestation.
 * @param attestations The Attestation that the token belongs to.
 * @returns
 */
export const deriveAttestationMintPda = ({
  attestation,
}: {
  attestation: Address;
}) =>
  getProgramDerivedAddress({
    programAddress: SOLANA_ATTESTATION_SERVICE_PROGRAM_ADDRESS,
    seeds: [ATTESTATION_MINT_SEED, attestation],
  });
