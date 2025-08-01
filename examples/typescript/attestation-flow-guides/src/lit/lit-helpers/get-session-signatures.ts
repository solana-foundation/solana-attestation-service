import { LitNodeClient } from "@lit-protocol/lit-node-client";
import { LitAccessControlConditionResource, LitActionResource, LitPKPResource } from "@lit-protocol/auth-helpers";
import { LIT_ABILITY } from "@lit-protocol/constants";
import type { SessionSigs } from "@lit-protocol/types";
import type { ethers } from "ethers";

import { PkpInfo, SiwsMessageForFormatting } from "../types";

export const getSessionSigs = async ({
    litNodeClient,
    ethersSigner,
    pkpInfo,
    capacityTokenId,
    litActionCodeSessionSigs,
    siwsMessage,
    siwsMessageSignature,
}: {
    litNodeClient: LitNodeClient;
    ethersSigner: ethers.Signer;
    pkpInfo: PkpInfo;
    capacityTokenId: string;
    litActionCodeSessionSigs: string;
    siwsMessage: SiwsMessageForFormatting;
    siwsMessageSignature: string;
}): Promise<SessionSigs> => {
    const { capacityDelegationAuthSig } =
        await litNodeClient.createCapacityDelegationAuthSig({
            dAppOwnerWallet: ethersSigner,
            capacityTokenId,
            delegateeAddresses: [pkpInfo.ethAddress],
            uses: "1",
        });

    return litNodeClient.getLitActionSessionSigs({
        pkpPublicKey: pkpInfo.publicKey,
        litActionCode: Buffer.from(litActionCodeSessionSigs).toString("base64"),
        capabilityAuthSigs: [capacityDelegationAuthSig],
        chain: "ethereum",
        resourceAbilityRequests: [
            {
                resource: new LitPKPResource("*"),
                ability: LIT_ABILITY.PKPSigning,
            },
            {
                resource: new LitActionResource("*"),
                ability: LIT_ABILITY.LitActionExecution,
            },
            {
                resource: new LitAccessControlConditionResource("*"),
                ability: LIT_ABILITY.AccessControlConditionDecryption,
            },
        ],
        jsParams: {
            siwsMessage: JSON.stringify(siwsMessage),
            siwsMessageSignature,
            pkpTokenId: pkpInfo.tokenId,
        },
    });
};