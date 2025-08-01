import { LIT_NETWORK, LIT_RPC } from "@lit-protocol/constants";
import { LitContracts } from "@lit-protocol/contracts-sdk";
import { LitNodeClient } from "@lit-protocol/lit-node-client";
import { LIT_NETWORKS_KEYS } from "@lit-protocol/types";
import { ethers } from "ethers";

import { litActionCode as litActionCodeSessionSigs } from "../lit-actions/litActionSessionSigs";
import { mintPkpAndAddPermittedAuthMethods } from "./mint-pkp";

export async function setupLit(
    {
        litNetwork = LIT_NETWORK.DatilDev,
        debug = false,
        capacityCreditRequestsPerKilosecond = 10,
        capacityCreditDaysUntilUTCMidnightExpiration = 1
    }: {
        litNetwork?: LIT_NETWORKS_KEYS,
        debug?: boolean,
        capacityCreditRequestsPerKilosecond?: number,
        capacityCreditDaysUntilUTCMidnightExpiration?: number
    } = {}) {
    if (
        process.env.LIT_PAYER_ETH_PRIVATE_KEY === undefined ||
        process.env.LIT_PAYER_ETH_PRIVATE_KEY === ""
    ) {
        throw new Error('LIT_PAYER_ETH_PRIVATE_KEY is not set. Please generate an Ethereum private key and fund the address with Lit test tokens using the faucet: https://chronicle-yellowstone-faucet.getlit.dev before continuing.');
    }

    const litPayerEthersWallet = new ethers.Wallet(
        process.env.LIT_PAYER_ETH_PRIVATE_KEY,
        new ethers.providers.JsonRpcProvider(LIT_RPC.CHRONICLE_YELLOWSTONE)
    );

    const litNodeClient = new LitNodeClient({
        litNetwork,
        debug,
    });
    await litNodeClient.connect();

    const litContractsClient = new LitContracts({
        signer: litPayerEthersWallet,
        network: litNetwork,
        debug,
    });
    await litContractsClient.connect();

    const pkpInfo = await mintPkpAndAddPermittedAuthMethods({
        litContractsClient,
        litActionCodeSessionSigs
    });

    const capacityTokenId = (
        await litContractsClient.mintCapacityCreditsNFT({
            requestsPerKilosecond: capacityCreditRequestsPerKilosecond,
            daysUntilUTCMidnightExpiration: capacityCreditDaysUntilUTCMidnightExpiration,
        })
    ).capacityTokenIdStr;

    return {
        litNodeClient,
        litPayerEthersWallet,
        pkpInfo,
        capacityTokenId
    }
}