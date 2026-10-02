import { generateKeyPairSigner } from '@solana/kit';
import {
    getCreateCredentialInstruction,
    getCreateSchemaInstruction,
    serializeAttestationData,
    getCreateAttestationInstruction,
    fetchSchema,
    getChangeAuthorizedSignersInstruction,
    findCredentialPda,
    findSchemaPda,
    findAttestationPda,
    getCloseAttestationInstruction,
} from '@solana/attestation';

import { CONFIG, sendInstruction, setupWallets, verifyAttestation } from './shared';

async function main() {
    console.log('Starting Solana Attestation Service Demo\n');

    console.log('1. Setting up wallets and funding payer...');
    const { client, authorizedSigner1, authorizedSigner2, issuer, testUser } = await setupWallets();
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

    console.log('\n4. Creating Attestation...');
    const [attestationPda] = await findAttestationPda({
        credential: credentialPda,
        schema: schemaPda,
        nonce: testUser.address,
    });

    const schema = await fetchSchema(client.rpc, schemaPda);
    const expiryTimestamp = Math.floor(Date.now() / 1000) + CONFIG.ATTESTATION_EXPIRY_DAYS * 24 * 60 * 60;

    const createAttestationInstruction = getCreateAttestationInstruction({
        payer,
        authority: authorizedSigner1,
        credential: credentialPda,
        schema: schemaPda,
        attestation: attestationPda,
        nonce: testUser.address,
        expiry: expiryTimestamp,
        data: serializeAttestationData(schema.data, CONFIG.ATTESTATION_DATA),
    });

    await sendInstruction(client, createAttestationInstruction, 'Attestation created');
    console.log(`    - Attestation PDA: ${attestationPda}`);

    console.log('\n5. Updating Authorized Signers...');
    const changeAuthSignersInstruction = getChangeAuthorizedSignersInstruction({
        payer,
        authority: issuer,
        credential: credentialPda,
        signers: [authorizedSigner1.address, authorizedSigner2.address],
    });

    await sendInstruction(client, changeAuthSignersInstruction, 'Authorized signers updated');

    console.log('\n6. Verifying Attestations...');

    const isUserVerified = await verifyAttestation({
        client,
        schemaPda,
        userAddress: testUser.address,
    });
    console.log(`    - Test User is ${isUserVerified ? 'verified' : 'not verified'}`);

    const randomUser = await generateKeyPairSigner();
    const isRandomVerified = await verifyAttestation({
        client,
        schemaPda,
        userAddress: randomUser.address,
    });
    console.log(`    - Random User is ${isRandomVerified ? 'verified' : 'not verified'}`);

    console.log('\n7. Closing Attestation...');

    const closeAttestationInstruction = getCloseAttestationInstruction({
        payer,
        attestation: attestationPda,
        authority: authorizedSigner1,
        credential: credentialPda,
    });
    await sendInstruction(client, closeAttestationInstruction, 'Closed attestation');
}

main()
    .then(() => console.log('\nSolana Attestation Service demo completed successfully!'))
    .catch(error => {
        console.error('❌ Demo failed:', error);
        process.exit(1);
    });
