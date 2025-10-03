import { Address, getAddressEncoder } from "@solana/kit";
import { PublicKey } from "@solana/web3.js";
import {
  deriveAddressSeedV2,
  deriveAddressV2,
  bn,
  Rpc as LightRpc,
} from "@lightprotocol/stateless.js";
import { SOLANA_ATTESTATION_SERVICE_PROGRAM_ADDRESS } from "./generated/programs";
import { getAttestationDecoder, Attestation } from "./generated/accounts";
import { ALLOWED_ADDRESS_TREE } from "./constants";

/**
 * Helper function to convert Address to PublicKey
 */
function addressToPublicKey(address: Address): PublicKey {
  return new PublicKey(address as string);
}

/**
 * Helper function to convert PublicKey to Address
 */
function publicKeyToAddress(pubkey: PublicKey): Address {
  return pubkey.toBase58() as Address;
}

/**
 * Derives the compressed address for an attestation using V2 address derivation.
 *
 * @param credential The Credential (aka Issuer) that controls the Attestation state
 * @param schema The Schema that the Attestation adheres to
 * @param nonce An Address that may either represent the Wallet the Attestation is associated with OR a randomly generated Address to prevent PDA collision
 * @returns The compressed address as an Address
 *
 * @example
 * ```typescript
 * const compressedAddress = await deriveCompressedAttestationPda({
 *   credential: credentialPda,
 *   schema: schemaPda,
 *   nonce: userAddress
 * });
 * ```
 */
export async function deriveCompressedAttestationPda({
  credential,
  schema,
  nonce,
}: {
  credential: Address;
  schema: Address;
  nonce: Address;
}): Promise<Address> {
  const { deriveAttestationPda } = await import("./pdas");
  const [attestationPda] = await deriveAttestationPda({
    credential,
    schema,
    nonce,
  });

  const addressTreePubkey = addressToPublicKey(ALLOWED_ADDRESS_TREE);
  const attestationPdaPubkey = addressToPublicKey(attestationPda);
  const programPubkey = addressToPublicKey(SOLANA_ATTESTATION_SERVICE_PROGRAM_ADDRESS);

  const seed = deriveAddressSeedV2([attestationPdaPubkey.toBytes()]);
  const compressedAddress = deriveAddressV2(
    seed,
    addressTreePubkey,
    programPubkey
  );

  return publicKeyToAddress(compressedAddress);
}

/**
 * Fetches and deserializes a compressed attestation from the Light Protocol indexer.
 *
 * @param lightRpc The Light Protocol RPC client
 * @param compressedAddress The compressed address to fetch
 * @returns An object containing the compressed account and deserialized attestation, or null if not found
 *
 * @example
 * ```typescript
 * const compressedAddress = await deriveCompressedAttestationPda({
 *   credential: credentialPda,
 *   schema: schemaPda,
 *   nonce: userAddress
 * });
 * const result = await fetchCompressedAttestation(lightRpc, compressedAddress);
 *
 * if (result) {
 *   const { compressedAccount, attestation } = result;
 *   console.log('Attestation data:', attestation.data);
 * }
 * ```
 */
export async function fetchCompressedAttestation(
  lightRpc: LightRpc,
  compressedAddress: Address
) {
  try {
    const compressedAddressPubkey = addressToPublicKey(compressedAddress);
    const compressedAccount = await lightRpc.getCompressedAccount(
      bn(compressedAddressPubkey.toBytes())
    );

    if (!compressedAccount || !compressedAccount.data) {
      return null;
    }

    const attestationDecoder = getAttestationDecoder();
    const attestation = attestationDecoder.decode(
      Uint8Array.from(compressedAccount.data.data)
    );

    return { compressedAccount, attestation };
  } catch (error) {
    return null;
  }
}
