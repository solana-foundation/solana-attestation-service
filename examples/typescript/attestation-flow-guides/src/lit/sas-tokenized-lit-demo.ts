import type { LitNodeClient } from "@lit-protocol/lit-node-client";
import {
    getCreateCredentialInstruction,
    getCreateSchemaInstruction,
    serializeAttestationData,
    fetchSchema,
    fetchCredential,
    deriveAttestationPda,
    deriveCredentialPda,
    deriveSchemaPda,
    getTokenizeSchemaInstruction,
    deriveSchemaMintPda,
    deriveSasAuthorityAddress,
    deriveAttestationMintPda,
    getCreateTokenizedAttestationInstruction,
    getCloseTokenizedAttestationInstruction,
    SOLANA_ATTESTATION_SERVICE_PROGRAM_ADDRESS,
    deriveEventAuthorityAddress,
    deserializeAttestationData,
    fetchAttestation,
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
    SolanaClient
} from "gill";
import {
    ASSOCIATED_TOKEN_PROGRAM_ADDRESS,
    fetchMint,
    findAssociatedTokenPda,
    getMintSize,
    TOKEN_2022_PROGRAM_ADDRESS,
    estimateComputeUnitLimitFactory
} from "gill/programs";
import { ethers } from "ethers";

import { createSiwsMessage, decryptAttestationData, encryptAttestationData, setupLit, signSiwsMessage } from "./lit-helpers";
import { AttestationEncryptionMetadata } from "./types";
import { getAuthorizedSigner1Keypair, getAuthorizedSigner2Keypair, getIssuerKeypair } from "./get-keypair";
import { ORIGINAL_DATA, SAS_TOKENIZED_CONFIG } from "./constants";

async function setupWallets(client: SolanaClient) {
    try {
        const payer = await generateKeyPairSigner();
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
            computeUnitPrice: 1,
        });

        const signature = await client.sendAndConfirmTransaction(tx, {
            skipPreflight: true,
            commitment: "processed"
        });
        console.log(`    - ${description}: ${signature}`);
        return signature;
    } catch (error) {
        console.error(`Transaction failed for ${description}:`, error);
        if (error instanceof Error && 'cause' in error) {
            console.error('Cause:', error.cause);
        }
        throw new Error(`Failed to ${description.toLowerCase()}: ${error instanceof Error ? error.message : 'Unknown error'}`);
    }
}

async function verifyTokenAttestation({
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
            console.log(`    - Schema is paused`);
            return { isVerified: false, decryptedAttestationData: null };
        }

        const [attestationPda] = await deriveAttestationPda({
            credential: schema.data.credential,
            schema: schemaPda,
            nonce: userAddress
        });
        const [attestationMint] = await deriveAttestationMintPda({
            attestation: attestationPda
        })

        let mintAccount;
        try {
            mintAccount = await fetchMint(client.rpc, attestationMint);
        } catch (error) {
            // Mint doesn't exist - user doesn't have a tokenized attestation
            return { isVerified: false, decryptedAttestationData: null };
        }

        if (!mintAccount) return { isVerified: false, decryptedAttestationData: null };
        if (mintAccount.data.extensions.__option === 'None') {
            return { isVerified: false, decryptedAttestationData: null };
        }
        const { value: foundExtensions } = mintAccount.data.extensions;

        // Verify member of group
        const [schemaMint] = await deriveSchemaMintPda({
            schema: schemaPda
        });
        const tokenGroupMember = foundExtensions.find(ext => ext.__kind === 'TokenGroupMember');
        if (!tokenGroupMember) return { isVerified: false, decryptedAttestationData: null };
        if (tokenGroupMember.group !== schemaMint) return { isVerified: false, decryptedAttestationData: null };

        // Verify token metadata
        const tokenMetadata = foundExtensions.find(ext => ext.__kind === 'TokenMetadata');
        if (!tokenMetadata) return { isVerified: false, decryptedAttestationData: null };

        // Verify attestation PDA matches
        const attestationInMetadata = tokenMetadata.additionalMetadata.get('attestation');
        if (attestationInMetadata !== attestationPda) return { isVerified: false, decryptedAttestationData: null };

        // Verify schema PDA matches  
        const schemaInMetadata = tokenMetadata.additionalMetadata.get('schema');
        if (schemaInMetadata !== schemaPda) return { isVerified: false, decryptedAttestationData: null };

        // Fetch attestation data from chain to get encryption metadata
        const attestation = await fetchAttestation(client.rpc, attestationPda);
        const attestationData = deserializeAttestationData(schema.data, attestation.data.data as Uint8Array) as AttestationEncryptionMetadata;
        console.log(`    - Retrieved attestation data from chain`);

        // Decrypt attestation data
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
            console.error("Error while decrypting the attestation data:", error);
            return { isVerified: false, decryptedAttestationData: null };
        }

        return { isVerified: true, decryptedAttestationData };
    } catch (error) {
        console.error("There was an error while verifying the tokenized attestation:", error);
        return { isVerified: false, decryptedAttestationData: null };
    }
}

let _litNodeClient: LitNodeClient | null = null;

async function main() {
    console.log("Starting Solana Attestation Service with Lit Protocol encrypted tokenized attestation demo\n");

    const client: SolanaClient = createSolanaClient({ urlOrMoniker: SAS_TOKENIZED_CONFIG.CLUSTER_OR_RPC });

    // Step 1: Setup wallets and fund payer
    console.log("1. Setting up wallets and funding payer...");
    const { payer, authorizedSigner1, authorizedSigner2, issuer, testUser } = await setupWallets(client);

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
        name: SAS_TOKENIZED_CONFIG.CREDENTIAL_NAME
    });

    try {
        const existingCredential = await fetchCredential(client.rpc, credentialPda);
        console.log(`    - Credential already exists: ${credentialPda}`);
        console.log(`    - Authority: ${existingCredential.data.authority}`);
        console.log(`    - Current authorized signers: ${existingCredential.data.authorizedSigners.length}`);
    } catch (error) {
        console.log(`    - Creating new credential: ${credentialPda}`);
        const createCredentialInstruction = getCreateCredentialInstruction({
            payer,
            credential: credentialPda,
            authority: issuer,
            name: SAS_TOKENIZED_CONFIG.CREDENTIAL_NAME,
            signers: [authorizedSigner1.address, authorizedSigner2.address]
        });
        await sendAndConfirmInstructions(client, payer, [createCredentialInstruction], 'Credential created');
        console.log(`    - Credential created successfully`);
    }

    // Step 4: Create Schema (if it doesn't exist)
    console.log("\n4. Setting up Schema...");
    const [schemaPda] = await deriveSchemaPda({
        credential: credentialPda,
        name: SAS_TOKENIZED_CONFIG.SCHEMA_NAME,
        version: SAS_TOKENIZED_CONFIG.SCHEMA_VERSION
    });

    try {
        const existingSchema = await fetchSchema(client.rpc, schemaPda);
        console.log(`    - Schema already exists: ${schemaPda}`);
        console.log(`    - Schema name: ${new TextDecoder().decode(existingSchema.data.name)}`);
        console.log(`    - Version: ${existingSchema.data.version}`);
    } catch (error) {
        console.log(`    - Creating new schema: ${schemaPda}`);
        const createSchemaInstruction = getCreateSchemaInstruction({
            authority: issuer,
            payer,
            name: SAS_TOKENIZED_CONFIG.SCHEMA_NAME,
            credential: credentialPda,
            description: SAS_TOKENIZED_CONFIG.SCHEMA_DESCRIPTION,
            fieldNames: SAS_TOKENIZED_CONFIG.SCHEMA_FIELDS,
            schema: schemaPda,
            layout: SAS_TOKENIZED_CONFIG.SCHEMA_LAYOUT
        });
        await sendAndConfirmInstructions(client, payer, [createSchemaInstruction], 'Schema created');
        console.log(`    - Schema created successfully`);
    }

    // Step 5: Tokenize Schema (if not already tokenized)
    console.log("\n5. Tokenizing Schema...");
    const [schemaMint] = await deriveSchemaMintPda({
        schema: schemaPda
    });

    try {
        const existingMint = await fetchMint(client.rpc, schemaMint);
        console.log(`    - Schema already tokenized: ${schemaMint}`);
    } catch (error) {
        console.log(`    - Tokenizing schema...`);
        const sasPda = await deriveSasAuthorityAddress();
        const schemaMintAccountSpace = getMintSize([
            {
                __kind: "GroupPointer",
                authority: sasPda,
                groupAddress: schemaMint
            },
        ]);

        const createTokenizeSchemaInstruction = getTokenizeSchemaInstruction({
            payer,
            authority: issuer,
            credential: credentialPda,
            schema: schemaPda,
            mint: schemaMint,
            sasPda,
            maxSize: schemaMintAccountSpace,
            tokenProgram: TOKEN_2022_PROGRAM_ADDRESS,
        });

        await sendAndConfirmInstructions(client, payer, [createTokenizeSchemaInstruction], 'Schema tokenized');
        console.log(`    - Schema Mint: ${schemaMint}`);
    }

    // Step 6: Create and Encrypt Tokenized Attestation
    console.log("\n6. Creating Tokenized Attestation...");
    const [attestationPda] = await deriveAttestationPda({
        credential: credentialPda,
        schema: schemaPda,
        nonce: testUser.address
    });
    const [attestationMint] = await deriveAttestationMintPda({
        attestation: attestationPda
    });

    const schema = await fetchSchema(client.rpc, schemaPda);
    const expiryTimestamp = Math.floor(Date.now() / 1000) + (SAS_TOKENIZED_CONFIG.ATTESTATION_EXPIRY_DAYS * 24 * 60 * 60);
    const sasPda = await deriveSasAuthorityAddress();
    const [recipientTokenAccount] = await findAssociatedTokenPda({
        mint: attestationMint,
        owner: testUser.address,
        tokenProgram: TOKEN_2022_PROGRAM_ADDRESS,
    });

    // Encrypt the attestation data
    const attestationEncryptionMetadata = await encryptAttestationData({
        attestationData: new TextEncoder().encode(JSON.stringify(ORIGINAL_DATA)),
        litNodeClient
    });

    const attestationMintAccountSpace = getMintSize([
        {
            __kind: "GroupMemberPointer",
            authority: sasPda,
            memberAddress: attestationMint,
        },
        { __kind: "NonTransferable" },
        {
            __kind: "MetadataPointer",
            authority: sasPda,
            metadataAddress: attestationMint,
        },
        { __kind: "PermanentDelegate", delegate: sasPda },
        { __kind: "MintCloseAuthority", closeAuthority: sasPda },
        {
            __kind: "TokenMetadata",
            updateAuthority: sasPda,
            mint: attestationMint,
            name: SAS_TOKENIZED_CONFIG.TOKEN_NAME,
            symbol: SAS_TOKENIZED_CONFIG.TOKEN_SYMBOL,
            uri: SAS_TOKENIZED_CONFIG.TOKEN_METADATA,
            additionalMetadata: new Map([
                ["attestation", attestationPda],
                ["schema", schemaPda]
            ]),
        },
        {
            __kind: "TokenGroupMember",
            group: schemaMint,
            mint: attestationMint,
            memberNumber: 1,
        },
    ]);

    const createTokenizedAttestationInstruction = await getCreateTokenizedAttestationInstruction({
        payer,
        authority: authorizedSigner1,
        credential: credentialPda,
        schema: schemaPda,
        attestation: attestationPda,
        schemaMint: schemaMint,
        attestationMint,
        sasPda,
        recipient: testUser.address,
        nonce: testUser.address,
        expiry: expiryTimestamp,
        data: serializeAttestationData(
            schema.data,
            {
                ...attestationEncryptionMetadata,
            }
        ),
        name: SAS_TOKENIZED_CONFIG.TOKEN_NAME,
        uri: SAS_TOKENIZED_CONFIG.TOKEN_METADATA,
        symbol: SAS_TOKENIZED_CONFIG.TOKEN_SYMBOL,
        mintAccountSpace: attestationMintAccountSpace,
        recipientTokenAccount: recipientTokenAccount,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ADDRESS,
        tokenProgram: TOKEN_2022_PROGRAM_ADDRESS,
    });

    await sendAndConfirmInstructions(client, payer, [createTokenizedAttestationInstruction], 'Tokenized attestation created');
    console.log(`    - Attestation PDA: ${attestationPda}`);
    console.log(`    - Attestation Mint: ${attestationMint}`);

    // Step 7: Verify Tokenized Attestations
    console.log("\n7. Verifying Tokenized Attestations...");

    const verification = await verifyTokenAttestation({
        client,
        schemaPda,
        userAddress: testUser.address,
        authorizedSigner: authorizedSigner1,
        litDecryptionParams: {
            litNodeClient,
            litPayerEthersWallet,
        },
    });

    console.log(`    - Test User is ${verification.isVerified ? 'verified' : 'not verified'}`);
    console.log(`    - Test User's token is ${verification.isVerified ? 'valid' : 'invalid'}`);
    if (verification.decryptedAttestationData) {
        console.log(`    - Decrypted Attestation Data: ${verification.decryptedAttestationData}`);
    }

    const randomUser = await generateKeyPairSigner();
    const randomVerification = await verifyTokenAttestation({
        client,
        schemaPda,
        userAddress: randomUser.address,
        authorizedSigner: authorizedSigner2,
        litDecryptionParams: {
            litNodeClient,
            litPayerEthersWallet,
        },
    });
    console.log(`    - Random User is ${randomVerification.isVerified ? 'verified' : 'not verified'}`);

    // Test with unauthorized signer (should fail)
    console.log("\n    Testing with unauthorized signer (should fail)...");
    const unauthorizedSigner = await generateKeyPairSigner();
    console.log(`    - Unauthorized signer address: ${unauthorizedSigner.address}`);

    const unauthorizedResult = await verifyTokenAttestation({
        client,
        schemaPda,
        userAddress: testUser.address,
        authorizedSigner: unauthorizedSigner,
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

    // Step 8: Close Tokenized Attestation
    console.log("\n8. Closing Tokenized Attestation...");
    const eventAuthority = await deriveEventAuthorityAddress();

    const closeTokenizedAttestationInstruction = getCloseTokenizedAttestationInstruction({
        payer,
        authority: authorizedSigner1,
        credential: credentialPda,
        attestation: attestationPda,
        eventAuthority,
        attestationProgram: SOLANA_ATTESTATION_SERVICE_PROGRAM_ADDRESS,
        attestationMint,
        sasPda,
        attestationTokenAccount: recipientTokenAccount,
        tokenProgram: TOKEN_2022_PROGRAM_ADDRESS
    });

    await sendAndConfirmInstructions(client, payer, [closeTokenizedAttestationInstruction], 'Tokenized Attestation closed');

    // Return summary data for pretty printing
    return {
        addresses: {
            credentialPda,
            schemaPda,
            attestationPda,
            schemaMint,
            attestationMint,
            testUserAddress: testUser.address
        },
        verification,
        randomVerification,
        unauthorizedResult,
        config: SAS_TOKENIZED_CONFIG,
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
        console.log(`   Token: ${results.config.TOKEN_NAME} (${results.config.TOKEN_SYMBOL})`);

        console.log("\n🔑 CREATED ACCOUNTS:");
        console.log(`   Credential PDA:    ${results.addresses.credentialPda}`);
        console.log(`   Schema PDA:        ${results.addresses.schemaPda}`);
        console.log(`   Schema Mint:       ${results.addresses.schemaMint}`);
        console.log(`   Attestation PDA:   ${results.addresses.attestationPda}`);
        console.log(`   Attestation Mint:  ${results.addresses.attestationMint}`);
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
            console.log("   ❌  Some tests failed. Please review the results above.");
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