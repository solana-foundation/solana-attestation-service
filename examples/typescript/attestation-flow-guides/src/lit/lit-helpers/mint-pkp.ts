import { AUTH_METHOD_SCOPE, AUTH_METHOD_TYPE, LIT_NETWORK } from "@lit-protocol/constants";
import { LitContracts } from "@lit-protocol/contracts-sdk";
import { ethers } from "ethers";
import ipfsOnlyHash from "typestub-ipfs-only-hash";

import { getPkpInfoFromMintReceipt } from "./get-pkp-info-from-tx-receipt";

export const mintPkpAndAddPermittedAuthMethods = async ({
    litContractsClient,
    litActionCodeSessionSigs
}: {
    litContractsClient: LitContracts;
    litActionCodeSessionSigs: string;
}) => {
    const keyType = AUTH_METHOD_TYPE.LitAction;
    const permittedAuthMethodTypes = [keyType];
    const permittedAuthMethodIds = [
        `0x${Buffer.from(
            ethers.utils.base58.decode(
                await ipfsOnlyHash.of(litActionCodeSessionSigs)
            )
        ).toString("hex")}`,
    ];
    const permittedAuthMethodPubkeys = ["0x"];
    const permittedAuthMethodScopes = [[AUTH_METHOD_SCOPE.SignAnything]];
    // This allows the PKP to update it's own Lit Auth Method,
    // or transfer ownership of the PKP to another address
    const addPkpEthAddressAsPermittedAddress = true;
    // This means the PKP ETH address is the owner of the PKP NFT
    const sendPkpToItself = true;

    const mintCost = await litContractsClient.pkpNftContract.read.mintCost();

    const gasEstimation =
        await litContractsClient.pkpHelperContract.write.estimateGas.mintNextAndAddAuthMethods(
            keyType,
            permittedAuthMethodTypes,
            permittedAuthMethodIds,
            permittedAuthMethodPubkeys,
            permittedAuthMethodScopes,
            addPkpEthAddressAsPermittedAddress,
            sendPkpToItself,
            { value: mintCost }
        );

    const tx =
        await litContractsClient.pkpHelperContract.write.mintNextAndAddAuthMethods(
            keyType,
            permittedAuthMethodTypes,
            permittedAuthMethodIds,
            permittedAuthMethodPubkeys,
            permittedAuthMethodScopes,
            addPkpEthAddressAsPermittedAddress,
            sendPkpToItself,
            {
                value: mintCost,
                gasLimit: gasEstimation.mul(105).div(100)
            }
        );

    return getPkpInfoFromMintReceipt(
        await tx.wait(),
        litContractsClient
    );
};