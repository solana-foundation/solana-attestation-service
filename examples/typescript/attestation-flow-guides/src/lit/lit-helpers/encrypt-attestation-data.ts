import { LitNodeClient } from "@lit-protocol/lit-node-client";
import ipfsOnlyHash from "typestub-ipfs-only-hash";

import { AttestationEncryptionMetadata } from "../types";
import { litActionCode as litActionCodeDecrypt } from "./lit-action-decrypt";

export const encryptAttestationData = async ({
    litNodeClient,
    attestationData,
}: {
    litNodeClient: LitNodeClient;
    attestationData: Uint8Array;
}): Promise<AttestationEncryptionMetadata> => {
    const { ciphertext, dataToEncryptHash } = await litNodeClient.encrypt({
        dataToEncrypt: attestationData,
        solRpcConditions: [
            {
                method: "",
                params: [":currentActionIpfsId"],
                pdaParams: [],
                pdaInterface: { offset: 0, fields: {} },
                pdaKey: "",
                chain: "solana",
                returnValueTest: {
                    key: "",
                    comparator: "=",
                    value: await ipfsOnlyHash.of(litActionCodeDecrypt),
                },
            },
        ],
        // @ts-ignore
        chain: "solana",
    });

    return { ciphertext, dataToEncryptHash };
}