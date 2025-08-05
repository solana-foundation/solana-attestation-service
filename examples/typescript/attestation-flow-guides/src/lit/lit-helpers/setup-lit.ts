import { LIT_NETWORK, LIT_RPC } from "@lit-protocol/constants";
import { LitNodeClient } from "@lit-protocol/lit-node-client";
import { LIT_NETWORKS_KEYS } from "@lit-protocol/types";
import { ethers } from "ethers";

export async function setupLit(
    {
        litNetwork = LIT_NETWORK.DatilDev,
        debug = false,
    }: {
        litNetwork?: LIT_NETWORKS_KEYS,
        debug?: boolean,
    } = {}) {
    const litPayerEthersWallet = new ethers.Wallet(
        ethers.Wallet.createRandom().privateKey,
        new ethers.providers.JsonRpcProvider(LIT_RPC.CHRONICLE_YELLOWSTONE)
    );

    const litNodeClient = new LitNodeClient({
        litNetwork,
        debug,
    });
    await litNodeClient.connect();

    return {
        litNodeClient,
        litPayerEthersWallet,
    }
}