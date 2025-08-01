import { SiwsMessageForFormatting, SiwsMessageInput } from "../types";

/**
 * Creates a complete Siws message by filling in missing properties with sensible defaults
 * @param siws - Siws message object with required address field
 * @returns Complete Siws message with all fields populated
 */
export function createSiwsMessage(siws: SiwsMessageInput): SiwsMessageForFormatting {
    const now = new Date();
    const expirationTime = new Date(now.getTime() + 10 * 60 * 1000); // 10 minutes

    // Generate a proper nonce if not provided (minimum 8 characters, alphanumeric)
    const generatedNonce = siws.nonce || Math.random().toString(36).substring(2, 12); // 10 character alphanumeric

    // Merge siws with defaults, siws values take precedence
    return {
        domain: siws.domain || "localhost",
        address: siws.address,
        statement: siws.statement || "Sign this message to authenticate with Lit Protocol",
        uri: siws.uri || "http://localhost",
        version: siws.version || "1",
        chainId: siws.chainId || "devnet", // Must be string as per Siws spec: mainnet, testnet, devnet, localnet, etc.
        nonce: generatedNonce,
        issuedAt: siws.issuedAt || now.toISOString(),
        expirationTime: siws.expirationTime || expirationTime.toISOString(),
        notBefore: siws.notBefore,
        requestId: siws.requestId,
        resources: siws.resources || []
    };
}