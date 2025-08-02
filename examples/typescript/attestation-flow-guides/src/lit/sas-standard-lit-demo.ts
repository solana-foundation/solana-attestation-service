import type { LitNodeClient } from "@lit-protocol/lit-node-client";
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
    SOLANA_ATTESTATION_SERVICE_PROGRAM_ADDRESS,
    fetchCredential
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
} from "gill";
import {
    estimateComputeUnitLimitFactory
} from "gill/programs";
import { ethers } from "ethers";

import { createSiwsMessage, decryptAttestationData, encryptAttestationData, setupLit, signSiwsMessage } from "./lit-helpers";
import { AttestationEncryptionMetadata } from "./types";
import { SAS_STANDARD_CONFIG, ORIGINAL_DATA } from "./constants";
import { getAuthorizedSigner1Keypair, getAuthorizedSigner2Keypair, getIssuerKeypair } from "./get-keypair";

async function setupWallets(client: SolanaClient) {
    try {
        const payer = await generateKeyPairSigner(); // or loadKeypairSignerFromFile(path.join(process.env.PAYER));
        const authorizedSigner1 = await getAuthorizedSigner1Keypair();
        const authorizedSigner2 = await getAuthorizedSigner2Keypair();
        const issuer = await getIssuerKeypair();
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
        console.error(`Transaction failed for ${description}:`, error);
        if (error instanceof Error && 'cause' in error) {
            console.error('Cause:', error.cause);
        }
        throw new Error(`Failed to ${description.toLowerCase()}: ${error instanceof Error ? error.message : 'Unknown error'}`);
    }
}

async function verifyAttestation({
    client,
    schemaPda,
    userAddress,
    authorizedSigner,
    litDecryptionParams,
}: {
    client: SolanaClient;
    schemaPda: Address;
    userAddress: Address;
    authorizedSigner: KeyPairSigner;
    litDecryptionParams: {
        litNodeClient: LitNodeClient;
        litPayerEthersWallet: ethers.Wallet;
    };
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
        const attestationData = deserializeAttestationData(schema.data, attestation.data.data as Uint8Array) as AttestationEncryptionMetadata;
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
                ...attestationData,
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

    const client: SolanaClient = createSolanaClient({ urlOrMoniker: SAS_STANDARD_CONFIG.CLUSTER_OR_RPC });

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
        litPayerEthersWallet,
    } = await setupLit();
    _litNodeClient = litNodeClient;

    // Step 3: Create Credential (if it doesn't exist)
    console.log("\n3. Setting up Credential...");
    const [credentialPda] = await deriveCredentialPda({
        authority: issuer.address,
        name: SAS_STANDARD_CONFIG.CREDENTIAL_NAME
    });

    try {
        // Check if credential already exists
        const existingCredential = await fetchCredential(client.rpc, credentialPda);
        console.log(`    - Credential already exists: ${credentialPda}`);
        console.log(`    - Authority: ${existingCredential.data.authority}`);
        console.log(`    - Authorized signers: ${existingCredential.data.authorizedSigners.length}`);
    } catch (error) {
        // Credential doesn't exist, create it
        console.log(`    - Creating new credential: ${credentialPda}`);
        const createCredentialInstruction = getCreateCredentialInstruction({
            payer,
            credential: credentialPda,
            authority: issuer,
            name: SAS_STANDARD_CONFIG.CREDENTIAL_NAME,
            signers: [authorizedSigner1.address]
        });

        await sendAndConfirmInstructions(client, payer, [createCredentialInstruction], 'Credential created');
        console.log(`    - Credential created successfully`);
    }

    // Step 4: Create Schema
    console.log("\n4.  Creating Schema...");
    const [schemaPda] = await deriveSchemaPda({
        credential: credentialPda,
        name: SAS_STANDARD_CONFIG.SCHEMA_NAME,
        version: SAS_STANDARD_CONFIG.SCHEMA_VERSION
    });

    try {
        // Check if schema already exists
        const existingSchema = await fetchSchema(client.rpc, schemaPda);
        console.log(`    - Schema already exists: ${schemaPda}`);
        console.log(`    - Schema name: ${new TextDecoder().decode(existingSchema.data.name)}`);
        console.log(`    - Version: ${existingSchema.data.version}`);
    } catch (error) {
        // Schema doesn't exist, create it
        console.log(`    - Creating new schema: ${schemaPda}`);
        const createSchemaInstruction = getCreateSchemaInstruction({
            authority: issuer,
            payer,
            name: SAS_STANDARD_CONFIG.SCHEMA_NAME,
            credential: credentialPda,
            description: SAS_STANDARD_CONFIG.SCHEMA_DESCRIPTION,
            fieldNames: SAS_STANDARD_CONFIG.SCHEMA_FIELDS,
            schema: schemaPda,
            layout: SAS_STANDARD_CONFIG.SCHEMA_LAYOUT,
        });

        await sendAndConfirmInstructions(client, payer, [createSchemaInstruction], 'Schema created');
        console.log(`    - Schema created successfully`);
    }

    // Step 5: Create and Encrypt Attestation
    console.log("\n5. Creating Attestation...");
    const [attestationPda] = await deriveAttestationPda({
        credential: credentialPda,
        schema: schemaPda,
        nonce: testUser.address
    });

    const schema = await fetchSchema(client.rpc, schemaPda);
    const expiryTimestamp = Math.floor(Date.now() / 1000) + (SAS_STANDARD_CONFIG.ATTESTATION_EXPIRY_DAYS * 24 * 60 * 60);

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
        },
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
        },
    });
    console.log(`    - Random User is ${isRandomVerified.isVerified ? 'verified' : 'not verified'}`);

    // Test with unauthorized signer (should fail)
    console.log("\n    Testing with unauthorized signer (should fail)...");
    const unauthorizedSigner = await generateKeyPairSigner();
    console.log(`    - Unauthorized signer address: ${unauthorizedSigner.address}`);

    const unauthorizedResult = await verifyAttestation({
        client,
        schemaPda,
        userAddress: testUser.address,
        authorizedSigner: unauthorizedSigner, // This signer is NOT in the credential
        litDecryptionParams: {
            litNodeClient,
            litPayerEthersWallet,
        },
    });

    if (unauthorizedResult.isVerified) {
        console.log(`    - ❌ Unauthorized signer is verified`);
    } else {
        console.log(`    - ✅ Unauthorized signer is not verified`);
    }

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

    // Return summary data for pretty printing
    return {
        addresses: {
            credentialPda,
            schemaPda,
            attestationPda,
            testUserAddress: testUser.address
        },
        verification: isUserVerified,
        randomVerification: isRandomVerified,
        unauthorizedResult,
        config: SAS_STANDARD_CONFIG,
        attestationEncryptionMetadata
    };
}

main()
    .then((results) => {
        console.log("\n" + "=".repeat(80));
        console.log("SOLANA ATTESTATION SERVICE WITH LIT PROTOCOL ENCRYPTED ATTESTATION DEMO");
        console.log("=".repeat(80));

        console.log("\n📋 DEMO CONFIGURATION:");
        console.log(`   Network: ${results.config.CLUSTER_OR_RPC}`);
        console.log(`   Organization: ${results.config.CREDENTIAL_NAME}`);
        console.log(`   Schema: ${results.config.SCHEMA_NAME} (v${results.config.SCHEMA_VERSION})`);

        console.log("\n🔑 CREATED ACCOUNTS:");
        console.log(`   Credential PDA:    ${results.addresses.credentialPda}`);
        console.log(`   Schema PDA:        ${results.addresses.schemaPda}`);
        console.log(`   Attestation PDA:   ${results.addresses.attestationPda}`);
        console.log(`   Test User:         ${results.addresses.testUserAddress}`);

        console.log("\n🧪 VERIFICATION TEST RESULTS:");

        // Test User Verification
        const testUserStatus = results.verification.isVerified ? "✅ PASSED" : "❌ FAILED";
        console.log(`   Test User Verification:     ${testUserStatus}`);
        if (results.verification.isVerified) {
            console.log(`   Encrypted Metadata:`);
            console.log(`     - Ciphertext: ${results.attestationEncryptionMetadata.ciphertext.substring(0, 50)}...`);
            console.log(`     - Data Hash: ${results.attestationEncryptionMetadata.dataToEncryptHash}`);
            if (results.verification.decryptedAttestationData) {
                console.log(`   Decrypted Attestation Data: ${results.verification.decryptedAttestationData}`);
            }
        }

        // Random User Verification (should fail)
        const randomUserStatus = !results.randomVerification.isVerified ? "✅ PASSED" : "❌ FAILED";
        console.log(`   Random User Verification:   ${randomUserStatus} (correctly rejected)`);

        // Unauthorized Signer Test (should fail)
        const unauthorizedStatus = !results.unauthorizedResult.isVerified ? "✅ PASSED" : "❌ FAILED";
        console.log(`   Unauthorized Signer Test:   ${unauthorizedStatus} (correctly rejected)`);

        const allTestsPassed = results.verification.isVerified &&
            !results.randomVerification.isVerified &&
            !results.unauthorizedResult.isVerified;

        if (allTestsPassed) {
            console.log("   ✅ ALL TESTS PASSED! Demo completed successfully.");
        } else {
            console.log("   ❌ Some tests failed. Please review the results above.");
        }

        console.log("\n" + "=".repeat(80));
    })
    .catch((error) => {
        console.error("\n❌ Demo failed:", error);
        process.exit(1);
    })
    .finally(() => {
        _litNodeClient?.disconnect();
    });
