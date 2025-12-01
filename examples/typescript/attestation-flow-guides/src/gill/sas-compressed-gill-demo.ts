import {
  getCreateCredentialInstruction,
  getCreateSchemaInstruction,
  serializeAttestationData,
  getCreateCompressedAttestationInstruction,
  fetchSchema,
  getChangeAuthorizedSignersInstruction,
  deserializeAttestationData,
  deriveAttestationPda,
  deriveCredentialPda,
  deriveSchemaPda,
  deriveEventAuthorityAddress,
  getCloseCompressedAttestationInstruction,
  ALLOWED_ADDRESS_TREE,
  deriveCompressedAttestationPda,
  fetchCompressedAttestation,
} from "sas-lib";
import {
  airdropFactory,
  generateKeyPairSigner,
  lamports,
  Signature,
  TransactionSigner,
  Instruction,
  Address,
  createSolanaClient,
  createTransaction,
  SolanaClient,
  ReadonlyUint8Array,
  getAddressEncoder,
} from "gill";
import { estimateComputeUnitLimitFactory } from "gill/programs";
import {
  createRpc,
  Rpc as LightRpc,
  bn,
  batchQueue,
  VERSION,
  featureFlags,
} from "@lightprotocol/stateless.js";

// Enable V2 for Light Protocol
featureFlags.version = VERSION.V2;
import { PublicKey } from "@solana/web3.js";

const CONFIG = {
  CLUSTER_OR_RPC: "http://127.0.0.1:8899",
  LIGHT_RPC: "http://127.0.0.1:8784",
  PROVER_URL: "http://127.0.0.1:3001",
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

// Helper functions for type conversions
function addressToPublicKey(address: Address): PublicKey {
  return new PublicKey(address as string);
}

function publicKeyToAddress(pubkey: PublicKey): Address {
  return pubkey.toBase58() as Address;
}

function addressToBytes(address: Address): ReadonlyUint8Array {
  const encoder = getAddressEncoder();
  return encoder.encode(address);
}

async function setupWallets(client: SolanaClient, lightRpc: LightRpc) {
  try {
    const payer = await generateKeyPairSigner();
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
  skipEstimate: boolean = false,
): Promise<Signature> {
  try {
    const { value: latestBlockhash } = await client.rpc
      .getLatestBlockhash()
      .send();
    let computeUnitLimit = 1_400_000;

    if (!skipEstimate) {
      try {
        const simulationTx = createTransaction({
          version: "legacy",
          feePayer: payer,
          instructions: instructions,
          latestBlockhash,
          computeUnitLimit: 1_400_000,
          computeUnitPrice: 1,
        });

        const estimateCompute = estimateComputeUnitLimitFactory({
          rpc: client.rpc,
        });
        computeUnitLimit = await estimateCompute(simulationTx);
      } catch (estimateError) {
        console.warn(
          `    - Failed to estimate compute units, using default: ${computeUnitLimit}`,
        );
      }
    }

    const tx = createTransaction({
      version: "legacy",
      feePayer: payer,
      instructions: instructions,
      latestBlockhash,
      computeUnitLimit,
      computeUnitPrice: 1,
    });

    const signature = await client.sendAndConfirmTransaction(tx, {
      commitment: "confirmed",
      skipPreflight: skipEstimate,
    });
    console.log(`    - ${description} - Signature: ${signature}`);
    return signature;
  } catch (error) {
    console.error(`\n❌ Error in ${description}:`);
    console.error("Error:", error);
    if (error && typeof error === "object" && "context" in error) {
      console.error("Error context:", error.context);
    }
    if (error && typeof error === "object" && "cause" in error) {
      console.error("Error cause:", error.cause);
    }
    // Try to extract signature for failed transaction
    if (error && typeof error === "object" && "signature" in error) {
      console.error("Transaction signature (failed):", error.signature);
    }
    throw new Error(
      `Failed to ${description.toLowerCase()}: ${error instanceof Error ? error.message : "Unknown error"}`,
    );
  }
}

function sleep(ms: number): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

async function main() {
  console.log("Starting Compressed Attestation Service Demo\n");

  const client: SolanaClient = createSolanaClient({
    urlOrMoniker: CONFIG.CLUSTER_OR_RPC,
  });
  const lightRpc: LightRpc = createRpc(
    CONFIG.CLUSTER_OR_RPC,
    CONFIG.LIGHT_RPC,
    CONFIG.PROVER_URL,
    { commitment: "confirmed" },
  );

  // Step 1: Setup wallets and fund payer
  console.log("1. Setting up wallets and funding payer...");
  const { payer, authorizedSigner1, authorizedSigner2, issuer, testUser } =
    await setupWallets(client, lightRpc);

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
  console.log("\n3. Creating Schema...");
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

  // Step 4: Create Compressed Attestation
  console.log("\n4. Creating Compressed Attestation...");

  // Derive attestation PDA
  const [attestationPda] = await deriveAttestationPda({
    credential: credentialPda,
    schema: schemaPda,
    nonce: testUser.address,
  });

  // Use V2 address derivation
  const compressedAddress = await deriveCompressedAttestationPda({
    credential: credentialPda,
    schema: schemaPda,
    nonce: testUser.address,
  });
  const compressedAddressPubkey = addressToPublicKey(compressedAddress);
  const addressTreePubkey = addressToPublicKey(ALLOWED_ADDRESS_TREE as Address);

  // Get validity proof for new address (non-inclusion proof)
  const createProofResult = await lightRpc.getValidityProofV0(
    [], // no input accounts
    [
      {
        tree: addressTreePubkey,
        queue: addressTreePubkey, // For V2, tree and queue are the same for address tree
        address: bn(compressedAddressPubkey.toBytes()),
      },
    ],
  );

  // Use batchQueue as the output queue
  const outputQueue = batchQueue as Address;

  const schema = await fetchSchema(client.rpc, schemaPda);
  const expiryTimestamp =
    Math.floor(Date.now() / 1000) +
    CONFIG.ATTESTATION_EXPIRY_DAYS * 24 * 60 * 60;

  // Extract proof bytes from ValidityProof and concatenate a, b, c arrays
  const proofBytes = createProofResult.compressedProof
    ? (new Uint8Array([
        ...createProofResult.compressedProof.a,
        ...createProofResult.compressedProof.b,
        ...createProofResult.compressedProof.c,
      ]) as ReadonlyUint8Array)
    : (new Uint8Array(128) as ReadonlyUint8Array);

  const createCompressedAttestationInstruction =
    getCreateCompressedAttestationInstruction({
      payer,
      authority: authorizedSigner1,
      credential: credentialPda,
      schema: schemaPda,
      outputQueue,
      proof: proofBytes,
      nonce: testUser.address,
      data: serializeAttestationData(schema.data, CONFIG.ATTESTATION_DATA),
      expiry: expiryTimestamp,
      addressRootIndex: createProofResult.rootIndices[0],
    });

  await sendAndConfirmInstructions(
    client,
    payer,
    [createCompressedAttestationInstruction],
    "Compressed attestation created",
  );
  console.log(`    - Attestation PDA: ${attestationPda}`);
  console.log(
    `    - Compressed Address: 0x${Buffer.from(compressedAddressPubkey.toBytes()).toString("hex")}`,
  );

  // Wait for indexer to sync
  await sleep(2000);

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

  // Step 6: Verify Compressed Attestations
  console.log("\n6. Verifying Compressed Attestations...");

  // Verify test user
  const testUserResult = await fetchCompressedAttestation(
    lightRpc,
    compressedAddress,
  );
  if (testUserResult) {
    const { attestation } = testUserResult;
    const attestationData = deserializeAttestationData(
      schema.data,
      Uint8Array.from(attestation.data),
    );
    console.log(`    - Test User is verified`);
    console.log(`    - Attestation data:`, attestationData);
  } else {
    console.log(`    - Test User is not verified`);
  }

  // Verify random user (should fail)
  const randomUser = await generateKeyPairSigner();
  const randomCompressedAddress = await deriveCompressedAttestationPda({
    credential: credentialPda,
    schema: schemaPda,
    nonce: randomUser.address,
  });
  const randomResult = await fetchCompressedAttestation(
    lightRpc,
    randomCompressedAddress,
  );
  if (randomResult) {
    console.log(`    - Random User is verified`);
  } else {
    console.log(`    - Random User is not verified`);
  }

  // Step 7: Close Compressed Attestation
  console.log("\n7. Closing Compressed Attestation...");

  // Wait for indexer to sync
  await sleep(2000);

  // Fetch compressed account
  const attestationResult = await fetchCompressedAttestation(
    lightRpc,
    compressedAddress,
  );
  if (!attestationResult) {
    throw new Error("Compressed account not found");
  }

  const { compressedAccount, attestation: attestationForClose } =
    attestationResult;

  // Get validity proof for closing (inclusion proof)
  const closeProofResult = await lightRpc.getValidityProofV0(
    [
      {
        hash: compressedAccount.hash,
        tree: compressedAccount.treeInfo.tree,
        queue: compressedAccount.treeInfo.queue,
      },
    ],
    [], // no new addresses
  );

  // Extract proof bytes from ValidityProof and concatenate a, b, c arrays
  const closeProofBytes = closeProofResult.compressedProof
    ? (new Uint8Array([
        ...closeProofResult.compressedProof.a,
        ...closeProofResult.compressedProof.b,
        ...closeProofResult.compressedProof.c,
      ]) as ReadonlyUint8Array)
    : null;

  // Convert tree and queue PublicKeys to Addresses
  const stateMerkleTree = publicKeyToAddress(compressedAccount.treeInfo.tree);
  const closeOutputQueue = publicKeyToAddress(compressedAccount.treeInfo.queue);

  const eventAuthority = await deriveEventAuthorityAddress();
  const closeCompressedAttestationInstruction =
    getCloseCompressedAttestationInstruction({
      payer,
      authority: authorizedSigner1,
      credential: attestationForClose.credential,
      eventAuthority,
      stateMerkleTree,
      outputQueue: closeOutputQueue,
      proof: closeProofBytes,
      nonce: attestationForClose.nonce,
      schema: attestationForClose.schema,
      signer: attestationForClose.signer,
      expiry: Number(attestationForClose.expiry),
      data: attestationForClose.data,
      rootIndex: closeProofResult.rootIndices[0],
      leafIndex: compressedAccount.leafIndex,
      address: compressedAddressPubkey.toBytes(),
    });

  await sendAndConfirmInstructions(
    client,
    payer,
    [closeCompressedAttestationInstruction],
    "Closed compressed attestation",
    true,
  );

  // Wait for indexer to sync
  await sleep(2000);

  // Verify account is nullified
  const deletedAttestation = await fetchCompressedAttestation(
    lightRpc,
    compressedAddress,
  );
  if (!deletedAttestation) {
    console.log(`    - Compressed attestation successfully nullified`);
  } else {
    console.log(`    - WARNING: Compressed attestation still exists`);
  }
}

main()
  .then(() =>
    console.log(
      "\nCompressed Attestation Service demo completed successfully!",
    ),
  )
  .catch((error) => {
    console.error("❌ Demo failed:", error);
    process.exit(1);
  });
