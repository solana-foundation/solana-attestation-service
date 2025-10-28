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
} from "sas-lib";
import {
  airdropFactory,
  generateKeyPairSigner,
  lamports,
  Signature,
  TransactionSigner,
  Instruction,
  Address,
  Blockhash,
  createSolanaClient,
  createTransaction,
  SolanaClient,
} from "gill";
import { estimateComputeUnitLimitFactory } from "gill/programs";

const CONFIG = {
  CLUSTER_OR_RPC:
    process.env.CI || process.env.LOCAL
      ? "http://127.0.0.1:8899"
      : "https://api.devnet.solana.com",
  CREDENTIAL_NAME: "TEST-ORGANIZATION",
  SCHEMA_NAME: "THE-BASICS",
  SCHEMA_LAYOUT: Buffer.from([12, 0, 12]),
  SCHEMA_FIELDS: ["name", "age", "country"],
  SCHEMA_VERSION: 1,
  SCHEMA_DESCRIPTION: "Basic user information schema for testing",
  ATTESTATION_DATA: {
    name: "test-user",
    age: 100,
    country: "usa",
  },
  ATTESTATION_EXPIRY_DAYS: 365,
};

async function setupWallets(client: SolanaClient) {
  try {
    const payer = await generateKeyPairSigner(); // or loadKeypairSignerFromFile(path.join(process.env.PAYER));
    const authorizedSigner1 = await generateKeyPairSigner();
    const authorizedSigner2 = await generateKeyPairSigner();
    const issuer = await generateKeyPairSigner();
    const testUser = await generateKeyPairSigner();

    const airdrop = airdropFactory({
      rpc: client.rpc,
      rpcSubscriptions: client.rpcSubscriptions,
    });
    const airdropTx: Signature = await airdrop({
      commitment: "processed",
      lamports: lamports(BigInt(1_000_000_000)),
      recipientAddress: payer.address,
    });
    const airdropTx2: Signature = await airdrop({
      commitment: "processed",
      lamports: lamports(BigInt(1_000_000_000)),
      recipientAddress: issuer.address,
    });
    const airdropTx3: Signature = await airdrop({
      commitment: "processed",
      lamports: lamports(BigInt(1_000_000_000)),
      recipientAddress: testUser.address,
    });

    console.log(`    - Airdrop completed: ${airdropTx}`);
    return { payer, authorizedSigner1, authorizedSigner2, issuer, testUser };
  } catch (error) {
    throw new Error(
      `Failed to setup wallets: ${error instanceof Error ? error.message : "Unknown error"}`,
    );
  }
}

async function sendAndConfirmInstructions(
  client: SolanaClient,
  payer: TransactionSigner,
  instructions: Instruction[],
  description: string,
): Promise<Signature> {
  try {
    const simulationTx = createTransaction({
      version: "legacy",
      feePayer: payer,
      instructions: instructions,
      latestBlockhash: {
        blockhash: "11111111111111111111111111111111" as Blockhash,
        lastValidBlockHeight: 0n,
      },
      computeUnitLimit: 1_400_000,
      computeUnitPrice: 1,
    });

    const estimateCompute = estimateComputeUnitLimitFactory({
      rpc: client.rpc,
    });
    const computeUnitLimit = await estimateCompute(simulationTx);
    const { value: latestBlockhash } = await client.rpc
      .getLatestBlockhash()
      .send();
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
    throw new Error(
      `Failed to ${description.toLowerCase()}: ${error instanceof Error ? error.message : "Unknown error"}`,
    );
  }
}

async function verifyAttestation({
  client,
  schemaPda,
  userAddress,
}: {
  client: SolanaClient;
  schemaPda: Address;
  userAddress: Address;
}): Promise<boolean> {
  try {
    const schema = await fetchSchema(client.rpc, schemaPda);
    if (schema.data.isPaused) {
      console.log(`    -  Schema is paused`);
      return false;
    }
    const [attestationPda] = await deriveAttestationPda({
      credential: schema.data.credential,
      schema: schemaPda,
      nonce: userAddress,
    });
    const attestation = await fetchAttestation(client.rpc, attestationPda);
    const attestationData = deserializeAttestationData(
      schema.data,
      attestation.data.data as Uint8Array,
    );
    console.log(`    - Attestation data:`, attestationData);
    const currentTimestamp = BigInt(Math.floor(Date.now() / 1000));
    return currentTimestamp < attestation.data.expiry;
  } catch (error) {
    return false;
  }
}

async function main() {
  console.log("Starting Solana Attestation Service Demo\n");

  const client: SolanaClient = createSolanaClient({
    urlOrMoniker: CONFIG.CLUSTER_OR_RPC,
  });

  // Step 1: Setup wallets and fund payer
  console.log("1. Setting up wallets and funding payer...");
  const { payer, authorizedSigner1, authorizedSigner2, issuer, testUser } =
    await setupWallets(client);

  // Step 2: Create Credential
  console.log("\n2. Creating Credential...");
  const [credentialPda] = await deriveCredentialPda({
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

  await sendAndConfirmInstructions(
    client,
    payer,
    [createCredentialInstruction],
    "Credential created",
  );
  console.log(`    - Credential PDA: ${credentialPda}`);

  // Step 3: Create Schema
  console.log("\n3.  Creating Schema...");
  const [schemaPda] = await deriveSchemaPda({
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

  await sendAndConfirmInstructions(
    client,
    payer,
    [createSchemaInstruction],
    "Schema created",
  );
  console.log(`    - Schema PDA: ${schemaPda}`);

  // Step 4: Create Attestation
  console.log("\n4. Creating Attestation...");
  const [attestationPda] = await deriveAttestationPda({
    credential: credentialPda,
    schema: schemaPda,
    nonce: testUser.address,
  });

  const schema = await fetchSchema(client.rpc, schemaPda);
  const expiryTimestamp =
    Math.floor(Date.now() / 1000) +
    CONFIG.ATTESTATION_EXPIRY_DAYS * 24 * 60 * 60;

  const createAttestationInstruction = await getCreateAttestationInstruction({
    payer,
    authority: authorizedSigner1,
    credential: credentialPda,
    schema: schemaPda,
    attestation: attestationPda,
    nonce: testUser.address,
    expiry: expiryTimestamp,
    data: serializeAttestationData(schema.data, CONFIG.ATTESTATION_DATA),
  });

  await sendAndConfirmInstructions(
    client,
    payer,
    [createAttestationInstruction],
    "Attestation created",
  );
  console.log(`    - Attestation PDA: ${attestationPda}`);

  // Step 5: Update Authorized Signers
  console.log("\n5. Updating Authorized Signers...");
  const changeAuthSignersInstruction =
    await getChangeAuthorizedSignersInstruction({
      payer,
      authority: issuer,
      credential: credentialPda,
      signers: [authorizedSigner1.address, authorizedSigner2.address],
    });

  await sendAndConfirmInstructions(
    client,
    payer,
    [changeAuthSignersInstruction],
    "Authorized signers updated",
  );

  // Step 6: Verify Attestations
  console.log("\n6. Verifying Attestations...");

  const isUserVerified = await verifyAttestation({
    client,
    schemaPda,
    userAddress: testUser.address,
  });
  console.log(
    `    - Test User is ${isUserVerified ? "verified" : "not verified"}`,
  );

  const randomUser = await generateKeyPairSigner();
  const isRandomVerified = await verifyAttestation({
    client,
    schemaPda,
    userAddress: randomUser.address,
  });
  console.log(
    `    - Random User is ${isRandomVerified ? "verified" : "not verified"}`,
  );

  // Step 7. Close Attestation
  console.log("\n7. Closing Attestation...");

  const eventAuthority = await deriveEventAuthorityAddress();
  const closeAttestationInstruction = await getCloseAttestationInstruction({
    payer,
    attestation: attestationPda,
    authority: authorizedSigner1,
    credential: credentialPda,
    eventAuthority,
    attestationProgram: SOLANA_ATTESTATION_SERVICE_PROGRAM_ADDRESS,
  });
  await sendAndConfirmInstructions(
    client,
    payer,
    [closeAttestationInstruction],
    "Closed attestation",
  );
}

main()
  .then(() =>
    console.log("\nSolana Attestation Service demo completed successfully!"),
  )
  .catch((error) => {
    console.error("❌ Demo failed:", error);
    process.exit(1);
  });
