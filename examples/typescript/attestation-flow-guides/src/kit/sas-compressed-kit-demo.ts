import {
  airdropFactory,
  createSolanaRpc,
  createSolanaRpcSubscriptions,
  generateKeyPairSigner,
  lamports,
  Rpc,
  sendAndConfirmTransactionFactory,
  pipe,
  createTransactionMessage,
  setTransactionMessageLifetimeUsingBlockhash,
  setTransactionMessageFeePayerSigner,
  appendTransactionMessageInstructions,
  SolanaRpcApi,
  SolanaRpcSubscriptionsApi,
  RpcSubscriptions,
  Signature,
  TransactionSigner,
  Instruction,
  Commitment,
  signTransactionMessageWithSigners,
  CompilableTransactionMessage,
  TransactionMessageWithBlockhashLifetime,
  getSignatureFromTransaction,
  Address,
  MicroLamports,
  ReadonlyUint8Array,
  getAddressEncoder,
} from "@solana/kit";
import {
  updateOrAppendSetComputeUnitLimitInstruction,
  updateOrAppendSetComputeUnitPriceInstruction,
  MAX_COMPUTE_UNIT_LIMIT,
  estimateComputeUnitLimitFactory,
} from "@solana-program/compute-budget";
import {
  getCreateCredentialInstruction,
  getCreateSchemaInstruction,
  serializeAttestationData,
  getCreateCompressedAttestationInstruction,
  fetchSchema,
  getChangeAuthorizedSignersInstruction,
  deserializeAttestationData,
  deriveCredentialPda,
  deriveSchemaPda,
  deriveAttestationPda,
  deriveEventAuthorityAddress,
  getCloseCompressedAttestationInstruction,
  SOLANA_ATTESTATION_SERVICE_PROGRAM_ADDRESS,
  ALLOWED_ADDRESS_TREE,
  deriveCompressedAttestationPda,
  fetchCompressedAttestation,
} from "sas-lib";
import {
  createRpc,
  Rpc as LightRpc,
  bn,
  batchQueue1,
  VERSION,
  featureFlags,
} from "@lightprotocol/stateless.js";

// Enable V2 for Light Protocol
featureFlags.version = VERSION.V2;
import { PublicKey } from "@solana/web3.js";

const CONFIG = {
  HTTP_CONNECTION_URL: "http://127.0.0.1:8899", // For devnet replace with helius rpc
  WSS_CONNECTION_URL: "ws://127.0.0.1:8900", // For devnet replace with helius rpc
  LIGHT_RPC: "http://127.0.0.1:8784", // For devnet replace with helius rpc
  PROVER_URL: "http://127.0.0.1:3001", // For devnet replace with helius rpc
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

interface Client {
  rpc: Rpc<SolanaRpcApi>;
  rpcSubscriptions: RpcSubscriptions<SolanaRpcSubscriptionsApi>;
  lightRpc: LightRpc;
}

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

async function setupWallets(client: Client) {
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

export const createDefaultTransaction = async (
  client: Client,
  feePayer: TransactionSigner,
  computeLimit: number = MAX_COMPUTE_UNIT_LIMIT,
  feeMicroLamports: MicroLamports = 1n as MicroLamports,
) => {
  const { value: latestBlockhash } = await client.rpc
    .getLatestBlockhash()
    .send();
  return pipe(
    createTransactionMessage({ version: 0 }),
    (tx) => setTransactionMessageFeePayerSigner(feePayer, tx),
    (tx) => setTransactionMessageLifetimeUsingBlockhash(latestBlockhash, tx),
    (tx) => updateOrAppendSetComputeUnitPriceInstruction(feeMicroLamports, tx),
    (tx) => updateOrAppendSetComputeUnitLimitInstruction(computeLimit, tx),
  );
};

export const signAndSendTransaction = async (
  client: Client,
  transactionMessage: CompilableTransactionMessage &
    TransactionMessageWithBlockhashLifetime,
  commitment: Commitment = "confirmed",
) => {
  const signedTransaction =
    await signTransactionMessageWithSigners(transactionMessage);
  const signature = getSignatureFromTransaction(signedTransaction);
  await sendAndConfirmTransactionFactory(client)(signedTransaction, {
    commitment,
  });
  return signature;
};

async function sendAndConfirmInstructions(
  client: Client,
  payer: TransactionSigner,
  instructions: Instruction[],
  description: string,
  skipEstimate: boolean = false,
  waitForIndexer: boolean = false,
): Promise<Signature> {
  try {
    let computeUnitLimit = 1_400_000;

    if (!skipEstimate) {
      try {
        const simulationTx = await pipe(
          await createDefaultTransaction(client, payer),
          (tx) => appendTransactionMessageInstructions(instructions, tx),
        );
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

    const signature = await pipe(
      await createDefaultTransaction(client, payer, computeUnitLimit),
      (tx) => appendTransactionMessageInstructions(instructions, tx),
      (tx) => signAndSendTransaction(client, tx),
    );
    console.log(`    - ${description} - Signature: ${signature}`);

    // Wait for indexer to sync if requested
    if (waitForIndexer) {
      const slot = await client.rpc.getSlot().send();
      await client.lightRpc.confirmTransactionIndexed(Number(slot));
    }

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
    throw new Error(
      `Failed to ${description.toLowerCase()}: ${error instanceof Error ? error.message : "Unknown error"}`,
    );
  }
}

async function verifyCompressedAttestation({
  client,
  schemaPda,
  userAddress,
  credentialPda,
}: {
  client: Client;
  schemaPda: Address;
  userAddress: Address;
  credentialPda: Address;
}): Promise<boolean> {
  try {
    const schema = await fetchSchema(client.rpc, schemaPda);
    if (schema.data.isPaused) {
      console.log(`    - Schema is paused`);
      return false;
    }

    const compressedAddress = await deriveCompressedAttestationPda({
      credential: credentialPda,
      schema: schemaPda,
      nonce: userAddress,
    });
    const result = await fetchCompressedAttestation(
      client.lightRpc,
      compressedAddress,
    );

    if (!result) {
      return false;
    }

    const { attestation } = result;
    const attestationData = deserializeAttestationData(
      schema.data,
      Uint8Array.from(attestation.data),
    );
    console.log(`    - Attestation data:`, attestationData);

    const currentTimestamp = BigInt(Math.floor(Date.now() / 1000));
    return currentTimestamp < attestation.expiry;
  } catch (error) {
    return false;
  }
}

async function main() {
  console.log("Starting Compressed Attestation Service Demo\n");

  const client: Client = {
    rpc: createSolanaRpc(CONFIG.HTTP_CONNECTION_URL),
    rpcSubscriptions: createSolanaRpcSubscriptions(CONFIG.WSS_CONNECTION_URL),
    lightRpc: createRpc(
      CONFIG.HTTP_CONNECTION_URL,
      CONFIG.LIGHT_RPC,
      CONFIG.PROVER_URL,
      { commitment: "confirmed" },
    ),
  };

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

  // Derive compressed address
  const compressedAddress = await deriveCompressedAttestationPda({
    credential: credentialPda,
    schema: schemaPda,
    nonce: testUser.address,
  });

  const [attestationPda] = await deriveAttestationPda({
    credential: credentialPda,
    schema: schemaPda,
    nonce: testUser.address,
  });

  const addressTree = addressToPublicKey(ALLOWED_ADDRESS_TREE as Address);
  const compressedAddressPubkey = addressToPublicKey(compressedAddress);

  // Get validity proof for new address (non-inclusion proof)
  const createProofResult = await client.lightRpc.getValidityProofV0(
    [], // no input accounts
    [
      {
        tree: addressTree,
        queue: addressTree, // For V2, tree and queue are the same for address tree
        address: bn(compressedAddressPubkey.toBytes()),
      },
    ],
  );

  // Extract proof bytes from ValidityProof and concatenate a, b, c arrays
  const proofBytes = createProofResult.compressedProof
    ? (new Uint8Array([
        ...createProofResult.compressedProof.a,
        ...createProofResult.compressedProof.b,
        ...createProofResult.compressedProof.c,
      ]) as ReadonlyUint8Array)
    : (new Uint8Array(128) as ReadonlyUint8Array);

  const schema = await fetchSchema(client.rpc, schemaPda);
  const expiryTimestamp =
    Math.floor(Date.now() / 1000) +
    CONFIG.ATTESTATION_EXPIRY_DAYS * 24 * 60 * 60;

  const createCompressedAttestationInstruction =
    getCreateCompressedAttestationInstruction({
      payer,
      authority: authorizedSigner1,
      credential: credentialPda,
      schema: schemaPda,
      outputQueue: batchQueue1 as Address,
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
    false,
    true,
  );
  console.log(`    - Attestation PDA: ${attestationPda}`);
  console.log(
    `    - Compressed Address: 0x${Buffer.from(compressedAddressPubkey.toBytes()).toString("hex")}`,
  );

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

  const isUserVerified = await verifyCompressedAttestation({
    client,
    schemaPda,
    userAddress: testUser.address,
    credentialPda,
  });
  console.log(
    `    - Test User is ${isUserVerified ? "verified" : "not verified"}`,
  );

  const randomUser = await generateKeyPairSigner();
  const isRandomVerified = await verifyCompressedAttestation({
    client,
    schemaPda,
    userAddress: randomUser.address,
    credentialPda,
  });
  console.log(
    `    - Random User is ${isRandomVerified ? "verified" : "not verified"}`,
  );

  // Step 7: Close Compressed Attestation
  console.log("\n7. Closing Compressed Attestation...");

  // Fetch compressed account
  const attestationResult = await fetchCompressedAttestation(
    client.lightRpc,
    compressedAddress,
  );
  if (!attestationResult) {
    throw new Error("Compressed account not found");
  }

  const { compressedAccount, attestation } = attestationResult;

  // Get validity proof for closing (inclusion proof)
  const closeProofResult = await client.lightRpc.getValidityProofV0(
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
      credential: attestation.credential,
      eventAuthority,
      stateMerkleTree,
      outputQueue: closeOutputQueue,
      proof: closeProofBytes,
      nonce: attestation.nonce,
      schema: attestation.schema,
      signer: attestation.signer,
      expiry: Number(attestation.expiry),
      data: attestation.data,
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
    true,
  );

  // Verify account is nullified
  const deletedAttestation = await fetchCompressedAttestation(
    client.lightRpc,
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
