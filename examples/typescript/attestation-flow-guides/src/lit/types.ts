import { SolRpcConditions } from "@lit-protocol/types";

/**
 * Sign-in With Solana (Siws) messages (following Phantom's specification)
 * https://github.com/phantom/sign-in-with-solana/tree/main
 */
export interface SiwsMessage {
    /**
     * Optional EIP-4361 domain requesting the sign-in.
     * If not provided, the wallet must determine the domain to include in the message.
     */
    domain?: string;
    /**
     * Optional Solana address performing the sign-in. The address is case-sensitive.
     * If not provided, the wallet must determine the Address to include in the message.
     */
    address?: string;
    /**
     * Optional EIP-4361 Statement. The statement is a human readable string and should not have new-line characters (\n).
     * If not provided, the wallet must not include Statement in the message.
     */
    statement?: string;
    /**
     * Optional EIP-4361 URI. The URL that is requesting the sign-in.
     * If not provided, the wallet must not include URI in the message.
     */
    uri?: string;
    /**
     * Optional EIP-4361 version.
     * If not provided, the wallet must not include Version in the message.
     */
    version?: string;
    /**
     * Optional EIP-4361 Chain ID.
     * The chainId can be one of the following: mainnet, testnet, devnet, localnet, solana:mainnet, solana:testnet, solana:devnet.
     * If not provided, the wallet must not include Chain ID in the message.
     */
    chainId?: string;
    /**
     * Optional EIP-4361 Nonce.
     * It should be an alphanumeric string containing a minimum of 8 characters.
     * If not provided, the wallet must not include Nonce in the message.
     */
    nonce?: string;
    /**
     * Optional ISO 8601 datetime string.
     * This represents the time at which the sign-in request was issued to the wallet.
     * Note: For Phantom, issuedAt has a threshold and it should be within +/- 10 minutes from the timestamp at which verification is taking place.
     * If not provided, the wallet must not include Issued At in the message.
     */
    issuedAt?: string;
    /**
     * Optional ISO 8601 datetime string.
     * This represents the time at which the sign-in request should expire.
     * If not provided, the wallet must not include Expiration Time in the message.
     */
    expirationTime?: string;
    /**
     * Optional ISO 8601 datetime string.
     * This represents the time at which the sign-in request becomes valid.
     * If not provided, the wallet must not include Not Before in the message.
     */
    notBefore?: string;
    /**
     * Optional EIP-4361 Request ID.
     * In addition to using nonce to avoid replay attacks, dapps can also choose to include a unique signature in the requestId.
     * Once the wallet returns the signed message, dapps can then verify this signature against the state to add an additional, strong layer of security.
     * If not provided, the wallet must not include Request ID in the message.
     */
    requestId?: string;
    /**
     * Optional EIP-4361 Resources.
     * Usually a list of references in the form of URIs that the dapp wants the user to be aware of.
     * These URIs should be separated by \n-, i.e., URIs in new lines starting with the character '-'.
     * If not provided, the wallet must not include Resources in the message.
     */
    resources?: string[];
}

/**
 * Type-safe Siws message for formatting - requires mandatory fields domain and address
 */
export interface SiwsMessageForFormatting extends SiwsMessage {
    domain: string;  // Required for message construction
    address: string; // Required for message construction
}

/**
 * Input for createSiwsMessage - requires address, all other fields optional
 */
export interface SiwsMessageInput extends Partial<SiwsMessage> {
    address: string; // Address is mandatory for creation
}

export interface AttestationEncryptionMetadata {
    ciphertext: string;
    dataToEncryptHash: string;
    accessControlConditions: SolRpcConditions;
}

export interface PkpInfo {
    ethAddress: string;
    publicKey: string;
    tokenId: string;
}
