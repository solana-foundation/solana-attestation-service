import { SiwsMessageForFormatting } from "../types";

/**
 * Formats Siws message according to the ABNF specification:
 * https://github.com/phantom/sign-in-with-solana/tree/main
 * 
 * @param siws - Siws message with required domain and address fields
 * @returns Formatted message string according to Siws ABNF specification
 */
export function formatSiwsMessage(siws: SiwsMessageForFormatting): string {
    if (!siws.domain || !siws.address) {
        throw new Error("Domain and address are required for Siws message construction");
    }

    // Start with the mandatory domain and address line
    let message = `${siws.domain} wants you to sign in with your Solana account:\n${siws.address}`;

    // Add statement if provided (with double newline separator)
    if (siws.statement) {
        message += `\n\n${siws.statement}`;
    }

    // Collect advanced fields in the correct order as per ABNF spec
    const fields: string[] = [];

    if (siws.uri) {
        fields.push(`URI: ${siws.uri}`);
    }
    if (siws.version) {
        fields.push(`Version: ${siws.version}`);
    }
    if (siws.chainId) {
        fields.push(`Chain ID: ${siws.chainId}`);
    }
    if (siws.nonce) {
        fields.push(`Nonce: ${siws.nonce}`);
    }
    if (siws.issuedAt) {
        fields.push(`Issued At: ${siws.issuedAt}`);
    }
    if (siws.expirationTime) {
        fields.push(`Expiration Time: ${siws.expirationTime}`);
    }
    if (siws.notBefore) {
        fields.push(`Not Before: ${siws.notBefore}`);
    }
    if (siws.requestId) {
        fields.push(`Request ID: ${siws.requestId}`);
    }
    if (siws.resources && siws.resources.length > 0) {
        fields.push(`Resources:`);
        for (const resource of siws.resources) {
            fields.push(`- ${resource}`);
        }
    }

    // Add advanced fields if any exist (with double newline separator)
    if (fields.length > 0) {
        message += `\n\n${fields.join('\n')}`;
    }

    return message;
}