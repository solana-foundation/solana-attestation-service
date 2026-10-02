import {
    deserializeAttestationData,
    fetchMaybeAttestation,
    fetchSchema,
    findAttestationPda,
    SchemaDataType,
} from '@solana/attestation';
import { Address, createClient, generateKeyPairSigner, Instruction, lamports } from '@solana/kit';
import { solanaDevnetRpc } from '@solana/kit-plugin-rpc';
import { generatedPayer } from '@solana/kit-plugin-signer';
import { fetchSysvarClock } from '@solana/sysvars';

export const CONFIG = {
    HTTP_CONNECTION_URL: 'https://api.devnet.solana.com', // 'http://127.0.0.1:8899',
    WSS_CONNECTION_URL: 'wss://api.devnet.solana.com', // 'ws://127.0.0.1:8900',
    CREDENTIAL_NAME: 'TEST-ORGANIZATION',
    SCHEMA_NAME: 'THE-BASICS',
    SCHEMA_LAYOUT: [SchemaDataType.String, SchemaDataType.U8, SchemaDataType.String],
    SCHEMA_FIELDS: ['name', 'age', 'country'],
    SCHEMA_VERSION: 1,
    SCHEMA_DESCRIPTION: 'Basic user information schema for testing',
    ATTESTATION_DATA: {
        name: 'test-user',
        age: 100,
        country: 'usa',
    },
    ATTESTATION_EXPIRY_DAYS: 365,
};

export type Client = Awaited<ReturnType<typeof setupWallets>>['client'];

export async function setupWallets() {
    const client = await createClient()
        .use(generatedPayer())
        .use(solanaDevnetRpc({ rpcUrl: CONFIG.HTTP_CONNECTION_URL, rpcSubscriptionsUrl: CONFIG.WSS_CONNECTION_URL }));
    const authorizedSigner1 = await generateKeyPairSigner();
    const authorizedSigner2 = await generateKeyPairSigner();
    const issuer = await generateKeyPairSigner();
    const testUser = await generateKeyPairSigner();

    const airdropTx = await client.airdrop(client.payer.address, lamports(1_000_000_000n));
    console.log(`    - Airdrop completed: ${airdropTx}`);

    return { client, authorizedSigner1, authorizedSigner2, issuer, testUser };
}

export async function sendInstruction(client: Client, instruction: Instruction, description: string) {
    const { context } = await client.sendTransaction(instruction);
    console.log(`    - ${description} - Signature: ${context.signature}`);
}

export async function verifyAttestation({
    client,
    schemaPda,
    userAddress,
}: {
    client: Client;
    schemaPda: Address;
    userAddress: Address;
}): Promise<boolean> {
    const schema = await fetchSchema(client.rpc, schemaPda);
    if (schema.data.isPaused) {
        console.log(`    -  Schema is paused`);
        return false;
    }
    const [attestationPda] = await findAttestationPda({
        credential: schema.data.credential,
        schema: schemaPda,
        nonce: userAddress,
    });
    const attestation = await fetchMaybeAttestation(client.rpc, attestationPda);
    if (!attestation.exists) {
        return false;
    }
    const attestationData = deserializeAttestationData(schema.data, attestation.data.data);
    console.log(`    - Attestation data:`, attestationData);
    if (attestation.data.expiry === 0n) {
        return true;
    }
    const { unixTimestamp } = await fetchSysvarClock(client.rpc);
    return unixTimestamp < attestation.data.expiry;
}
