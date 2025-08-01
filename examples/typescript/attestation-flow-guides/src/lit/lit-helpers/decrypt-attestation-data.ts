import { LitNodeClient } from "@lit-protocol/lit-node-client";
import { SolRpcConditions } from "@lit-protocol/types";
import { ethers } from "ethers";

import { PkpInfo, SiwsMessageForFormatting } from "../types";
import { litActionCode as litActionCodeDecrypt } from "../lit-actions/litActionDecrypt";
import { litActionCode as litActionCodeSessionSigs } from "../lit-actions/litActionSessionSigs";
import { getSessionSigs } from "./get-session-signatures";

export const decryptAttestationData = async ({
    litNodeClient,
    litPayerEthersWallet,
    pkpInfo,
    capacityTokenId,
    ciphertext,
    dataToEncryptHash,
    accessControlConditions,
    siwsMessage,
    siwsMessageSignature,
}: {
    litNodeClient: LitNodeClient;
    litPayerEthersWallet: ethers.Wallet;
    pkpInfo: PkpInfo;
    capacityTokenId: string;
    ciphertext: string;
    dataToEncryptHash: string;
    accessControlConditions: SolRpcConditions;
    siwsMessage: SiwsMessageForFormatting;
    siwsMessageSignature: string;
}) => {
    const response = await litNodeClient.executeJs({
        code: litActionCodeDecrypt,
        sessionSigs: await getSessionSigs({
            litNodeClient,
            ethersSigner: litPayerEthersWallet,
            pkpInfo,
            capacityTokenId,
            litActionCodeSessionSigs,
            siwsMessage,
            siwsMessageSignature,
        }),
        jsParams: {
            siwsMessage: JSON.stringify(siwsMessage),
            siwsMessageSignature,
            solRpcConditions: accessControlConditions,
            ciphertext,
            dataToEncryptHash,
        },
    });

    return response.response;
}