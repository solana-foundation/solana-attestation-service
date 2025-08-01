import { ethers } from "ethers";
import { LitContracts } from "@lit-protocol/contracts-sdk";

import { PkpInfo } from "../types";

export const getPkpInfoFromMintReceipt = async (
    txReceipt: ethers.ContractReceipt,
    litContractsClient: LitContracts
): Promise<PkpInfo> => {
    if (!txReceipt || !txReceipt.logs) {
        throw new Error("Invalid transaction receipt provided");
    }

    let tokenId: string | null = null;
    let publicKey: string | null = null;
    let ethAddress: string | null = null;

    // Iterate through all logs to find PKPMinted event
    for (const { data, topics } of txReceipt.logs) {
        try {
            const fragment = litContractsClient.pkpNftContract.read.interface.events["PKPMinted(uint256,bytes)"];
            const decodedEvent = litContractsClient.pkpNftContract.read.interface.decodeEventLog(fragment, data, topics);

            tokenId = decodedEvent.tokenId.toString();
            publicKey = decodedEvent.pubkey || decodedEvent.publicKey;
            const pubKeyHex = publicKey?.startsWith('0x') ? publicKey : `0x${publicKey}`;
            ethAddress = ethers.utils.computeAddress(pubKeyHex);

            // Break out of loop once we find the event
            break;
        } catch (e) {
            // Not a PKPMinted event or failed to decode, continue to next log
            continue;
        }
    }

    if (!tokenId || !publicKey || !ethAddress) {
        throw new Error("PKPMinted event not found in transaction receipt or missing required fields");
    }

    return {
        tokenId,
        publicKey,
        ethAddress
    };
};
