import { LitNodeClient } from "@lit-protocol/lit-node-client";
import type { LIT_NETWORKS_KEYS } from "@lit-protocol/types";
import { LIT_NETWORK, LIT_RPC } from "@lit-protocol/constants";
import { LitContracts } from "@lit-protocol/contracts-sdk";
import {
    getCreateCredentialInstruction,
    getCreateSchemaInstruction,
    serializeAttestationData,
    getCreateAttestationInstruction,
    fetchSchema,
    getChangeAuthorizedSignersInstruction,
    fetchAttestation,
    deserializeAttestationData,
    deriveAttestationPda,
    deriveCredentialPda,
    deriveSchemaPda,
    deriveEventAuthorityAddress,
    getCloseAttestationInstruction,
    SOLANA_ATTESTATION_SERVICE_PROGRAM_ADDRESS
} from "sas-lib";
import {
    airdropFactory,
    generateKeyPairSigner,
    lamports,
    Signature,
    TransactionSigner,
    KeyPairSigner,
    Instruction,
    Address,
    Blockhash,
    createSolanaClient,
    createTransaction,
    SolanaClient,
    createSignableMessage,
} from "gill";
import {
    estimateComputeUnitLimitFactory
} from "gill/programs";
import { ethers } from "ethers";

import { litActionCode as litActionCodeSessionSigs } from "./lit-actions/litActionSessionSigs";
import { encryptAttestationData, mintPkpAndAddPermittedAuthMethods } from "./lit-helpers";
import { AttestationEncryptionMetadata, PkpInfo, SiwsMessageForFormatting, SiwsMessageInput } from "./types";
import { decryptAttestationData } from "./lit-helpers/decrypt-attestation-data";

const CONFIG = {
    // TODO Revert back to devnet
    // CLUSTER_OR_RPC: 'devnet',
    CLUSTER_OR_RPC: 'localnet',
    CREDENTIAL_NAME: 'LIT-ENCRYPTED-ATTESTATIONS',
    SCHEMA_NAME: 'LIT-ENCRYPTED-METADATA',
    SCHEMA_LAYOUT: Buffer.from([12, 12, 12]),
    SCHEMA_FIELDS: ["ciphertext", "dataToEncryptHash", "accessControlConditions"],
    SCHEMA_VERSION: 1,
    SCHEMA_DESCRIPTION: 'Schema for Lit Protocol encrypted attestation metadata with access control conditions',
    ATTESTATION_EXPIRY_DAYS: 365
};

const ORIGINAL_DATA = {
    name: "test-user",
    age: 100,
    country: "usa",
};

/**
 * Creates a complete Siws message by filling in missing properties with sensible defaults
 * @param siws - Siws message object with required address field
 * @returns Complete Siws message with all fields populated
 */
function createSiwsMessage(siws: SiwsMessageInput): SiwsMessageForFormatting {
    const now = new Date();
    const expirationTime = new Date(now.getTime() + 10 * 60 * 1000); // 10 minutes

    // Generate a proper nonce if not provided (minimum 8 characters, alphanumeric)
    const generatedNonce = siws.nonce || Math.random().toString(36).substring(2, 12); // 10 character alphanumeric

    // Merge siws with defaults, siws values take precedence
    return {
        domain: siws.domain || "localhost",
        address: siws.address,
        statement: siws.statement || "Sign this message to authenticate with Lit Protocol",
        uri: siws.uri || "http://localhost",
        version: siws.version || "1",
        chainId: siws.chainId || "devnet", // Must be string as per Siws spec: mainnet, testnet, devnet, localnet, etc.
        nonce: generatedNonce,
        issuedAt: siws.issuedAt || now.toISOString(),
        expirationTime: siws.expirationTime || expirationTime.toISOString(),
        notBefore: siws.notBefore,
        requestId: siws.requestId,
        resources: siws.resources || []
    };
}

/**
 * Formats Siws message according to the ABNF specification:
 * https://github.com/phantom/sign-in-with-solana/blob/main/siws.md#abnf-message-format
 * 
 * @param siws - Siws message with required domain and address fields
 * @returns Formatted message string according to Siws ABNF specification
 */
function formatSiwsMessage(siws: SiwsMessageForFormatting): string {
    if (!siws.domain || !siws.address) {
        throw new Error("Domain and address are required for Siws message construction");
    }

    // Start with the mandatory domain and address line
    let message = `${siws.domain} wants you to sign in with your Solana account:\n${siws.address}`;

    // Add statement if provided (with double newline separator)
    if (siws.statement) {
        message += `\n\n${siws.statement}`;
    }

    // Collect advanced fields in the correct order as per ABNF spec
    const fields: string[] = [];

    if (siws.uri) {
        fields.push(`URI: ${siws.uri}`);
    }
    if (siws.version) {
        fields.push(`Version: ${siws.version}`);
    }
    if (siws.chainId) {
        fields.push(`Chain ID: ${siws.chainId}`);
    }
    if (siws.nonce) {
        fields.push(`Nonce: ${siws.nonce}`);
    }
    if (siws.issuedAt) {
        fields.push(`Issued At: ${siws.issuedAt}`);
    }
    if (siws.expirationTime) {
        fields.push(`Expiration Time: ${siws.expirationTime}`);
    }
    if (siws.notBefore) {
        fields.push(`Not Before: ${siws.notBefore}`);
    }
    if (siws.requestId) {
        fields.push(`Request ID: ${siws.requestId}`);
    }
    if (siws.resources && siws.resources.length > 0) {
        fields.push(`Resources:`);
        for (const resource of siws.resources) {
            fields.push(`- ${resource}`);
        }
    }

    // Add advanced fields if any exist (with double newline separator)
    if (fields.length > 0) {
        message += `\n\n${fields.join('\n')}`;
    }

    return message;
}

/**
 * Signs a SIWS message using a Solana keypair signer
 * @param siwsMessage - The formatted SIWS message to sign
 * @param signer - The Solana KeyPairSigner from setupWallets
 * @returns Base58-encoded signature
 */
async function signSiwsMessage(siwsMessage: SiwsMessageForFormatting, signer: KeyPairSigner): Promise<string> {
    try {
        const message = createSignableMessage(new TextEncoder().encode(formatSiwsMessage(siwsMessage)));
        const signedMessage = await signer.signMessages([message]);
        return ethers.utils.base58.encode(signedMessage[0][signer.address]);
    } catch (error) {
        throw new Error(`Failed to sign SIWS message: ${error instanceof Error ? error.message : 'Unknown error'}`);
    }
}

async function setupWallets(client: SolanaClient) {
    try {
        const payer = await generateKeyPairSigner(); // or loadKeypairSignerFromFile(path.join(process.env.PAYER));
        const authorizedSigner1 = await generateKeyPairSigner();
        const authorizedSigner2 = await generateKeyPairSigner();
        const issuer = await generateKeyPairSigner();
        const testUser = await generateKeyPairSigner();

        const airdrop = airdropFactory({ rpc: client.rpc, rpcSubscriptions: client.rpcSubscriptions });
        const airdropTx: Signature = await airdrop({
            commitment: 'processed',
            lamports: lamports(BigInt(1_000_000_000)),
            recipientAddress: payer.address
        });

        console.log(`    - Airdrop completed: ${airdropTx}`);
        return { payer, authorizedSigner1, authorizedSigner2, issuer, testUser };
    } catch (error) {
        throw new Error(`Failed to setup wallets: ${error instanceof Error ? error.message : 'Unknown error'}`);
    }
}

async function setupLit(
    {
        authorizedSolanaAddresses,
        litNetwork = LIT_NETWORK.DatilDev,
        debug = false,
        capacityCreditRequestsPerKilosecond = 10,
        capacityCreditDaysUntilUTCMidnightExpiration = 1
    }: {
        litNetwork?: LIT_NETWORKS_KEYS,
        authorizedSolanaAddresses: string[],
        debug?: boolean,
        capacityCreditRequestsPerKilosecond?: number,
        capacityCreditDaysUntilUTCMidnightExpiration?: number
    }) {
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
        ethersSigner: litPayerEthersWallet,
        authorizedSolanaAddresses,
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
        litContractsClient,
        litPayerEthersWallet,
        pkpInfo,
        capacityTokenId
    }
}

async function sendAndConfirmInstructions(
    client: SolanaClient,
    payer: TransactionSigner,
    instructions: Instruction[],
    description: string
): Promise<Signature> {
    try {
        const simulationTx = createTransaction({
            version: "legacy",
            feePayer: payer,
            instructions: instructions,
            latestBlockhash: {
                blockhash: '11111111111111111111111111111111' as Blockhash,
                lastValidBlockHeight: 0n,
            },
            computeUnitLimit: 1_400_000,
            computeUnitPrice: 1,
        });

        const estimateCompute = estimateComputeUnitLimitFactory({ rpc: client.rpc });
        const computeUnitLimit = await estimateCompute(simulationTx);
        const { value: latestBlockhash } = await client.rpc.getLatestBlockhash().send();
        const tx = createTransaction({
            version: "legacy",
            feePayer: payer,
            instructions: instructions,
            latestBlockhash,
            computeUnitLimit,
            computeUnitPrice: 1, // In production, use dynamic pricing
        });

        const signature = await client.sendAndConfirmTransaction(tx);
        console.log(`    - ${description} - Signature: ${signature}`);
        return signature;
    } catch (error) {
        throw new Error(`Failed to ${description.toLowerCase()}: ${error instanceof Error ? error.message : 'Unknown error'}`);
    }
}

async function verifyAttestation({
    client,
    schemaPda,
    userAddress,
    authorizedSigner,
    litDecryptionParams,
    attestationEncryptionMetadata
}: {
    client: SolanaClient;
    schemaPda: Address;
    userAddress: Address;
    authorizedSigner: KeyPairSigner;
    litDecryptionParams: {
        litNodeClient: LitNodeClient;
        litPayerEthersWallet: ethers.Wallet;
        pkpInfo: PkpInfo;
        capacityTokenId: string;
    };
    attestationEncryptionMetadata: AttestationEncryptionMetadata;
}): Promise<{ isVerified: boolean, decryptedAttestationData: string | null }> {
    try {
        const schema = await fetchSchema(client.rpc, schemaPda);
        if (schema.data.isPaused) {
            console.log(`    -  Schema is paused`);
            return { isVerified: false, decryptedAttestationData: null };
        }
        const [attestationPda] = await deriveAttestationPda({
            credential: schema.data.credential,
            schema: schemaPda,
            nonce: userAddress
        });
        const attestation = await fetchAttestation(client.rpc, attestationPda);
        const attestationData = deserializeAttestationData(schema.data, attestation.data.data as Uint8Array);
        console.log(`    - Attestation data:`, attestationData);

        let decryptedAttestationData: string | null = null;
        try {
            const siwsMessage = createSiwsMessage({
                address: authorizedSigner.address,
                domain: 'localhost',
                uri: 'http://localhost',
                version: '1',
            });
            const siwsMessageSignature = await signSiwsMessage(siwsMessage, authorizedSigner);

            decryptedAttestationData = await decryptAttestationData({
                ...litDecryptionParams,
                ...attestationEncryptionMetadata,
                siwsMessage,
                siwsMessageSignature,
            }) as string;
        } catch (error) {
            console.error("There was an error while decrypting the attestation data:", error);
            return { isVerified: false, decryptedAttestationData: null };
        }

        const currentTimestamp = BigInt(Math.floor(Date.now() / 1000));
        return { isVerified: currentTimestamp < attestation.data.expiry, decryptedAttestationData };
    } catch (error) {
        return { isVerified: false, decryptedAttestationData: null };
    }
}

let _litNodeClient: LitNodeClient | null = null;

async function main() {
    console.log("Starting Solana Attestation Service with Lit Protocol encrypted attestation demo\n");

    const client: SolanaClient = createSolanaClient({ urlOrMoniker: CONFIG.CLUSTER_OR_RPC });

    // Step 1: Setup wallets and fund payer
    console.log("1. Setting up wallets and funding payer...");
    const {
        payer,
        authorizedSigner1,
        authorizedSigner2,
        issuer,
        testUser
    } = await setupWallets(client);

    // Step 2: Setup Lit
    console.log("2. Setting up Lit...");
    const {
        litNodeClient,
        litContractsClient,
        litPayerEthersWallet,
        pkpInfo,
        capacityTokenId
    } = await setupLit({
        litNetwork: LIT_NETWORK.DatilDev,
        authorizedSolanaAddresses: [authorizedSigner1.address, authorizedSigner2.address]
    });
    _litNodeClient = litNodeClient;

    // Step 3: Create Credential
    console.log("\n3. Creating Credential...");
    const [credentialPda] = await deriveCredentialPda({
        authority: issuer.address,
        name: CONFIG.CREDENTIAL_NAME
    });

    const createCredentialInstruction = getCreateCredentialInstruction({
        payer,
        credential: credentialPda,
        authority: issuer,
        name: CONFIG.CREDENTIAL_NAME,
        signers: [authorizedSigner1.address]
    });

    await sendAndConfirmInstructions(client, payer, [createCredentialInstruction], 'Credential created');
    console.log(`    - Credential PDA: ${credentialPda}`);

    // Step 4: Create Schema
    console.log("\n4.  Creating Schema...");
    const [schemaPda] = await deriveSchemaPda({
        credential: credentialPda,
        name: CONFIG.SCHEMA_NAME,
        version: CONFIG.SCHEMA_VERSION
    });

    const createSchemaInstruction = getCreateSchemaInstruction({
        authority: issuer,
        payer,
        name: CONFIG.SCHEMA_NAME,
        credential: credentialPda,
        description: CONFIG.SCHEMA_DESCRIPTION,
        fieldNames: CONFIG.SCHEMA_FIELDS,
        schema: schemaPda,
        layout: CONFIG.SCHEMA_LAYOUT,
    });

    await sendAndConfirmInstructions(client, payer, [createSchemaInstruction], 'Schema created');
    console.log(`    - Schema PDA: ${schemaPda}`);

    // Step 5: Create and Encrypt Attestation
    console.log("\n5. Creating Attestation...");
    const [attestationPda] = await deriveAttestationPda({
        credential: credentialPda,
        schema: schemaPda,
        nonce: testUser.address
    });

    const schema = await fetchSchema(client.rpc, schemaPda);
    const expiryTimestamp = Math.floor(Date.now() / 1000) + (CONFIG.ATTESTATION_EXPIRY_DAYS * 24 * 60 * 60);

    const attestationEncryptionMetadata = await encryptAttestationData({
        attestationData: new TextEncoder().encode(JSON.stringify(ORIGINAL_DATA)),
        litNodeClient
    });

    const createAttestationInstruction = getCreateAttestationInstruction({
        payer,
        authority: authorizedSigner1,
        credential: credentialPda,
        schema: schemaPda,
        attestation: attestationPda,
        nonce: testUser.address,
        expiry: expiryTimestamp,
        data: serializeAttestationData(
            schema.data,
            {
                ...attestationEncryptionMetadata,
                accessControlConditions: JSON.stringify(attestationEncryptionMetadata.accessControlConditions)
            }
        ),
    });

    await sendAndConfirmInstructions(client, payer, [createAttestationInstruction], 'Attestation created');
    console.log(`    - Attestation PDA: ${attestationPda}`);

    // Step 6: Update Authorized Signers
    console.log("\n6. Updating Authorized Signers...");
    const changeAuthSignersInstruction = getChangeAuthorizedSignersInstruction({
        payer,
        authority: issuer,
        credential: credentialPda,
        signers: [authorizedSigner1.address, authorizedSigner2.address]
    });

    await sendAndConfirmInstructions(client, payer, [changeAuthSignersInstruction], 'Authorized signers updated');

    // Step 7: Verify Attestations
    console.log("\n7. Verifying Attestations...");

    const isUserVerified = await verifyAttestation({
        client,
        schemaPda,
        userAddress: testUser.address,
        authorizedSigner: authorizedSigner1, // Use one of the authorized signers
        litDecryptionParams: {
            litNodeClient,
            litPayerEthersWallet,
            pkpInfo,
            capacityTokenId
        },
        attestationEncryptionMetadata
    });
    console.log(`    - Test User is ${isUserVerified.isVerified ? 'verified' : 'not verified'}`);
    if (isUserVerified.decryptedAttestationData) {
        console.log(`    - Decrypted Attestation Data: ${isUserVerified.decryptedAttestationData}`);
    }

    const randomUser = await generateKeyPairSigner();
    const isRandomVerified = await verifyAttestation({
        client,
        schemaPda,
        userAddress: randomUser.address,
        authorizedSigner: authorizedSigner2, // Use the other authorized signer  
        litDecryptionParams: {
            litNodeClient,
            litPayerEthersWallet,
            pkpInfo,
            capacityTokenId
        },
        attestationEncryptionMetadata
    });
    console.log(`    - Random User is ${isRandomVerified.isVerified ? 'verified' : 'not verified'}`);

    // Step 7. Close Attestation
    console.log("\n7. Closing Attestation...");

    const eventAuthority = await deriveEventAuthorityAddress();
    const closeAttestationInstruction = getCloseAttestationInstruction({
        payer,
        attestation: attestationPda,
        authority: authorizedSigner1,
        credential: credentialPda,
        eventAuthority,
        attestationProgram: SOLANA_ATTESTATION_SERVICE_PROGRAM_ADDRESS
    });
    await sendAndConfirmInstructions(client, payer, [closeAttestationInstruction], 'Closed attestation');
}

main()
    .then(() => console.log("\nSolana Attestation Service with Lit Protocol encrypted attestation demo completed successfully!"))
    .catch((error) => {
        console.error("❌ Demo failed:", error);
        process.exit(1);
    })
    .finally(() => {
        _litNodeClient?.disconnect();
    });
