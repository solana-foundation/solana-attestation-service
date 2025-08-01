import { AUTH_METHOD_SCOPE, AUTH_METHOD_TYPE, LIT_NETWORK } from "@lit-protocol/constants";
import { LitContracts } from "@lit-protocol/contracts-sdk";
import { ethers } from "ethers";
import ipfsOnlyHash from "typestub-ipfs-only-hash";

import { getPkpInfoFromMintReceipt } from "./get-pkp-info-from-tx-receipt";

export const mintPkpAndAddPermittedAuthMethods = async ({
    litContractsClient,
    authorizedSolanaAddresses,
    litActionCodeSessionSigs
}: {
    litContractsClient: LitContracts;
    ethersSigner: ethers.Signer;
    authorizedSolanaAddresses: string[];
    litActionCodeSessionSigs: string;
}) => {
    const permittedAuthMethodTypes: ethers.BigNumberish[] = [];
    const permittedAuthMethodIds = [];
    const permittedAuthMethodPubkeys = [];
    const permittedAuthMethodScopes: ethers.BigNumberish[][] = [];

    console.log("LIT ACTION IPFS CID", await ipfsOnlyHash.of(litActionCodeSessionSigs))

    /**
     * Grant SignAnything permission to the Lit Action
     * which validates a SIWS message.
     */
    const litActionAuthMethodType = AUTH_METHOD_TYPE.LitAction;
    const litActionAuthMethodId = `0x${Buffer.from(
        ethers.utils.base58.decode(
            await ipfsOnlyHash.of(litActionCodeSessionSigs)
        )
    ).toString("hex")}`;
    const litActionAuthMethodPubkey = "0x";
    const litActionAuthMethodScope = [AUTH_METHOD_SCOPE.SignAnything];

    permittedAuthMethodTypes.push(litActionAuthMethodType);
    permittedAuthMethodIds.push(litActionAuthMethodId);
    permittedAuthMethodPubkeys.push(litActionAuthMethodPubkey);
    permittedAuthMethodScopes.push(litActionAuthMethodScope);

    /**
     * Grant NoPermissions to the custom SIWS Auth Method,
     * which is used to identify which Solana public keys are
     * authorized sign using the above Lit Action.
     */
    const siwsAuthMethodType = ethers.utils.keccak256(
        ethers.utils.toUtf8Bytes("Solana Attestation Service Lit Encryption Guide")
    );

    for (const solanaAddress of authorizedSolanaAddresses) {
        const siwsAuthMethodId = ethers.utils.keccak256(
            ethers.utils.toUtf8Bytes(`siws:${solanaAddress}`)
        );

        permittedAuthMethodTypes.push(siwsAuthMethodType);
        permittedAuthMethodIds.push(siwsAuthMethodId);
        permittedAuthMethodPubkeys.push("0x");
        permittedAuthMethodScopes.push([AUTH_METHOD_SCOPE.NoPermissions]);
    }

    const estimatedGas =
        await litContractsClient.pkpHelperContract.write.estimateGas.mintNextAndAddAuthMethods(
            AUTH_METHOD_TYPE.LitAction, // keyType
            permittedAuthMethodTypes,
            permittedAuthMethodIds,
            permittedAuthMethodPubkeys,
            permittedAuthMethodScopes,
            // addPkpEthAddressAsPermittedAddress
            // This allows the PKP to update it's own Lit Auth Method,
            // or transfer ownership of the PKP to another address
            true,
            // sendPkpToItself
            // This means the PKP ETH address is the owner of the PKP NFT
            true,
            { value: await litContractsClient.pkpNftContract.read.mintCost() }
        );

    const tx = await litContractsClient.pkpHelperContract.write.mintNextAndAddAuthMethods(
        AUTH_METHOD_TYPE.LitAction, // keyType
        permittedAuthMethodTypes,
        permittedAuthMethodIds,
        permittedAuthMethodPubkeys,
        permittedAuthMethodScopes,
        // addPkpEthAddressAsPermittedAddress
        // This allows the PKP to update it's own Lit Auth Method,
        // or transfer ownership of the PKP to another address
        true,
        // sendPkpToItself
        // This means the PKP ETH address is the owner of the PKP NFT
        true,
        { value: await litContractsClient.pkpNftContract.read.mintCost(), gasLimit: estimatedGas.mul(105).div(100) }
    );

    return getPkpInfoFromMintReceipt(
        await tx.wait(),
        litContractsClient
    );
};