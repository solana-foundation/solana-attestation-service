import { LitNodeClient } from "@lit-protocol/lit-node-client";
import { generateAuthSig, createSiweMessage, LitAccessControlConditionResource, LitActionResource } from "@lit-protocol/auth-helpers";
import { LIT_ABILITY } from "@lit-protocol/constants";
import type { ethers } from "ethers";

import { LitDecryptionResponse, SiwsMessageForFormatting } from "../types";
import { litActionCode as litActionCodeDecrypt } from "./lit-action-decrypt";

export const decryptAttestationData = async ({
    litNodeClient,
    litPayerEthersWallet,
    ciphertext,
    dataToEncryptHash,
    siwsMessage,
    siwsMessageSignature,
}: {
    litNodeClient: LitNodeClient;
    litPayerEthersWallet: ethers.Wallet;
    ciphertext: string;
    dataToEncryptHash: string;
    siwsMessage: SiwsMessageForFormatting;
    siwsMessageSignature: string;
}) => {
    try {
        const response = await litNodeClient.executeJs({
            code: litActionCodeDecrypt,
            sessionSigs: await litNodeClient.getSessionSigs({
                chain: "ethereum",
                expiration: new Date(Date.now() + 1000 * 60 * 10).toISOString(), // 10 minutes
                resourceAbilityRequests: [
                    {
                        resource: new LitActionResource("*"),
                        ability: LIT_ABILITY.LitActionExecution,
                    },
                    {
                        resource: new LitAccessControlConditionResource("*"),
                        ability: LIT_ABILITY.AccessControlConditionDecryption,
                    },
                ],
                authNeededCallback: async ({
                    uri,
                    expiration,
                    resourceAbilityRequests,
                }) => {
                    const toSign = await createSiweMessage({
                        uri,
                        expiration,
                        resources: resourceAbilityRequests,
                        walletAddress: await litPayerEthersWallet.getAddress(),
                        nonce: await litNodeClient.getLatestBlockhash(),
                        litNodeClient,
                    });

                    return await generateAuthSig({
                        signer: litPayerEthersWallet,
                        toSign,
                    });
                },
            }),
            jsParams: {
                siwsMessage: JSON.stringify(siwsMessage),
                siwsMessageSignature,
                ciphertext,
                dataToEncryptHash,
            },
        });

        const responseJSON = JSON.parse(response.response as string) as LitDecryptionResponse;

        if (!responseJSON.hasOwnProperty('success')) {
            throw new Error(`Unexpected return value from Lit decryption request: ${response.response}`);
        }

        if (responseJSON.success === false) {
            throw new Error(`Failed to decrypt attestation data: ${response.response}`);
        }

        return responseJSON.decryptedData;
    } catch (error) {
        throw new Error(`An unexpected error occurred while decrypting the attestation data: ${error instanceof Error ? error.message : 'Unknown error'}`);
    }
}