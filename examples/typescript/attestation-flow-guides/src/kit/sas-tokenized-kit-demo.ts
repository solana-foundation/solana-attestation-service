import {
    ASSOCIATED_TOKEN_PROGRAM_ADDRESS,
    fetchMaybeMint,
    findAssociatedTokenPda,
    getMintSize,
    TOKEN_2022_PROGRAM_ADDRESS,
} from '@solana-program/token-2022';
import {
    getCreateCredentialInstruction,
    getCreateSchemaInstruction,
    serializeAttestationData,
    fetchSchema,
    findAttestationPda,
    findCredentialPda,
    findSchemaPda,
    getTokenizeSchemaInstruction,
    findSchemaMintPda,
    findSasAuthorityPda,
    findAttestationMintPda,
    getCreateTokenizedAttestationInstruction,
    getCloseTokenizedAttestationInstructionAsync,
} from '@solana/attestation';
import { Address } from '@solana/kit';

import { Client, CONFIG, sendInstruction, setupWallets, verifyAttestation } from './shared';

const TOKEN_CONFIG = {
    TOKEN_NAME: 'Test Identity',
    TOKEN_METADATA: 'https://example.com/metadata.json',
    TOKEN_SYMBOL: 'TESTID',
};

async function verifyTokenAttestation({
    client,
    schemaPda,
    userAddress,
}: {
    client: Client;
    schemaPda: Address;
    userAddress: Address;
}): Promise<boolean> {
    if (!(await verifyAttestation({ client, schemaPda, userAddress }))) {
        return false;
    }
    const schema = await fetchSchema(client.rpc, schemaPda);

    const [attestationPda] = await findAttestationPda({
        credential: schema.data.credential,
        schema: schemaPda,
        nonce: userAddress,
    });
    const [attestationMint] = await findAttestationMintPda({
        attestation: attestationPda,
    });
    const mintAccount = await fetchMaybeMint(client.rpc, attestationMint);
    if (!mintAccount.exists || mintAccount.data.extensions.__option === 'None') {
        return false;
    }
    const { value: foundExtensions } = mintAccount.data.extensions;

    const [schemaMint] = await findSchemaMintPda({
        schema: schemaPda,
    });
    const tokenGroupMember = foundExtensions.find(ext => ext.__kind === 'TokenGroupMember');
    if (tokenGroupMember?.group !== schemaMint) return false;

    const tokenMetadata = foundExtensions.find(ext => ext.__kind === 'TokenMetadata');
    if (!tokenMetadata) return false;

    return (
        tokenMetadata.additionalMetadata.get('attestation') === attestationPda &&
        tokenMetadata.additionalMetadata.get('schema') === schemaPda
    );
}

async function main() {
    console.log('Starting Solana Attestation Service Demo\n');

    console.log('1. Setting up wallets and funding payer...');
    const { client, authorizedSigner1, issuer, testUser } = await setupWallets();
    const { payer } = client;

    console.log('\n2. Creating Credential...');
    const [credentialPda] = await findCredentialPda({
        authority: issuer.address,
        name: CONFIG.CREDENTIAL_NAME,
    });

    const createCredentialInstruction = getCreateCredentialInstruction({
        payer,
        credential: credentialPda,
        authority: issuer,
        name: CONFIG.CREDENTIAL_NAME,
        signers: [authorizedSigner1.address],
    });

    await sendInstruction(client, createCredentialInstruction, 'Credential created');
    console.log(`    - Credential PDA: ${credentialPda}`);

    console.log('\n3.  Creating Schema...');
    const [schemaPda] = await findSchemaPda({
        credential: credentialPda,
        name: CONFIG.SCHEMA_NAME,
        version: CONFIG.SCHEMA_VERSION,
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

    await sendInstruction(client, createSchemaInstruction, 'Schema created');
    console.log(`    - Schema PDA: ${schemaPda}`);

    console.log('\n4. Tokenizing Schema...');
    const [schemaMint] = await findSchemaMintPda({
        schema: schemaPda,
    });
    const [sasPda] = await findSasAuthorityPda();
    const schemaMintAccountSpace = getMintSize([
        {
            __kind: 'GroupPointer',
            authority: sasPda,
            groupAddress: schemaMint,
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

    await sendInstruction(client, createTokenizeSchemaInstruction, 'Schema tokenized');
    console.log(`    - Schema Mint: ${schemaMint}`);

    console.log('\n5. Creating Tokenized Attestation...');
    const [attestationPda] = await findAttestationPda({
        credential: credentialPda,
        schema: schemaPda,
        nonce: testUser.address,
    });
    const [attestationMint] = await findAttestationMintPda({
        attestation: attestationPda,
    });

    const schema = await fetchSchema(client.rpc, schemaPda);
    const expiryTimestamp = Math.floor(Date.now() / 1000) + CONFIG.ATTESTATION_EXPIRY_DAYS * 24 * 60 * 60;
    const [recipientTokenAccount] = await findAssociatedTokenPda({
        mint: attestationMint,
        owner: testUser.address,
        tokenProgram: TOKEN_2022_PROGRAM_ADDRESS,
    });

    const attestationMintAccountSpace = getMintSize([
        {
            __kind: 'GroupMemberPointer',
            authority: sasPda,
            memberAddress: attestationMint,
        },
        { __kind: 'NonTransferable' },
        {
            __kind: 'MetadataPointer',
            authority: sasPda,
            metadataAddress: attestationMint,
        },
        { __kind: 'PermanentDelegate', delegate: sasPda },
        { __kind: 'MintCloseAuthority', closeAuthority: sasPda },
        {
            __kind: 'TokenMetadata',
            updateAuthority: sasPda,
            mint: attestationMint,
            name: TOKEN_CONFIG.TOKEN_NAME,
            symbol: TOKEN_CONFIG.TOKEN_SYMBOL,
            uri: TOKEN_CONFIG.TOKEN_METADATA,
            additionalMetadata: new Map([
                ['attestation', attestationPda],
                ['schema', schemaPda],
            ]),
        },
        {
            __kind: 'TokenGroupMember',
            group: schemaMint,
            mint: attestationMint,
            memberNumber: 1,
        },
    ]);

    const createTokenizedAttestationInstruction = getCreateTokenizedAttestationInstruction({
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
        data: serializeAttestationData(schema.data, CONFIG.ATTESTATION_DATA),
        name: TOKEN_CONFIG.TOKEN_NAME,
        uri: TOKEN_CONFIG.TOKEN_METADATA,
        symbol: TOKEN_CONFIG.TOKEN_SYMBOL,
        mintAccountSpace: attestationMintAccountSpace,
        recipientTokenAccount: recipientTokenAccount,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ADDRESS,
        tokenProgram: TOKEN_2022_PROGRAM_ADDRESS,
    });

    await sendInstruction(client, createTokenizedAttestationInstruction, 'Tokenized attestation created');
    console.log(`    - Attestation PDA: ${attestationPda}`);
    console.log(`    - Attestation Mint: ${attestationMint}`);

    console.log('\n6. Verifying Attestations...');

    const isUserVerified = await verifyAttestation({
        client,
        schemaPda,
        userAddress: testUser.address,
    });
    console.log(`    - Test User is ${isUserVerified ? 'verified' : 'not verified'}`);

    console.log('\n7. Verifying Attestation Token...');
    const isTokenVerified = await verifyTokenAttestation({ client, schemaPda, userAddress: testUser.address });
    console.log(`    - Test User's token is ${isTokenVerified ? 'verified' : 'not verified'}`);

    console.log('\n8. Closing Tokenized Attestations...');
    const closeTokenizedAttestationInstruction = await getCloseTokenizedAttestationInstructionAsync({
        payer,
        authority: authorizedSigner1,
        credential: credentialPda,
        attestation: attestationPda,
        attestationTokenAccount: recipientTokenAccount,
        tokenProgram: TOKEN_2022_PROGRAM_ADDRESS,
    });
    await sendInstruction(client, closeTokenizedAttestationInstruction, 'Tokenized Attestation closed');
}

main()
    .then(() => console.log('\nSolana Attestation Service demo completed successfully!'))
    .catch(error => {
        console.error('❌ Demo failed:', error);
        process.exit(1);
    });
