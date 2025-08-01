import { LitNodeClient } from "@lit-protocol/lit-node-client";

import { AttestationEncryptionMetadata } from "../types";
import { litActionCode as litActionCodeDecrypt } from "../lit-actions/litActionDecrypt";
import { getDecryptionAccessControlConditions } from "./get-decryption-access-control-conditions";

export const encryptAttestationData = async ({
    litNodeClient,
    attestationData,
}: {
    litNodeClient: LitNodeClient;
    attestationData: Uint8Array;
}): Promise<AttestationEncryptionMetadata> => {
    const accessControlConditions = await getDecryptionAccessControlConditions(litActionCodeDecrypt);
    const { ciphertext, dataToEncryptHash } = await litNodeClient.encrypt({
        dataToEncrypt: attestationData,
        solRpcConditions: accessControlConditions,
        // @ts-ignore
        chain: "solana",
    });

    return { ciphertext, dataToEncryptHash, accessControlConditions };
}