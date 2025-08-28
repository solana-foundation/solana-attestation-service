import { createSignableMessage, KeyPairSigner } from "gill";

import { SiwsMessageForFormatting } from "../types";
import { formatSiwsMessage } from "./format-siws-message";

/**
 * Signs a SIWS message using a Solana keypair signer
 * @param siwsMessage - The formatted SIWS message to sign
 * @param signer - The Solana KeyPairSigner from setupWallets
 * @returns Hex-encoded signature
 */
export async function signSiwsMessage(siwsMessage: SiwsMessageForFormatting, signer: KeyPairSigner): Promise<string> {
    try {
        const message = createSignableMessage(new TextEncoder().encode(formatSiwsMessage(siwsMessage)));
        const signedMessage = await signer.signMessages([message]);
        // Convert signature bytes to hex string
        return Buffer.from(signedMessage[0][signer.address]).toString('hex');
    } catch (error) {
        throw new Error(`Failed to sign SIWS message: ${error instanceof Error ? error.message : 'Unknown error'}`);
    }
}