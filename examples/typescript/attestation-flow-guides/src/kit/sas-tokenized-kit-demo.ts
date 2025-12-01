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
} from "@solana/kit";
import {
  updateOrAppendSetComputeUnitLimitInstruction,
  updateOrAppendSetComputeUnitPriceInstruction,
  MAX_COMPUTE_UNIT_LIMIT,
  estimateComputeUnitLimitFactory,
} from "@solana-program/compute-budget";
import {
  ASSOCIATED_TOKEN_PROGRAM_ADDRESS,
  fetchMint,
  findAssociatedTokenPda,
  getMintSize,
  TOKEN_2022_PROGRAM_ADDRESS,
} from "@solana-program/token-2022";
import {
  getCreateCredentialInstruction,
  getCreateSchemaInstruction,
  serializeAttestationData,
  fetchSchema,
  fetchAttestation,
  deserializeAttestationData,
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
} from "sas-lib";

const CONFIG = {
  HTTP_CONNECTION_URL:
    process.env.CI || process.env.LOCAL
      ? "http://127.0.0.1:8899"
      : "https://api.devnet.solana.com",
  WSS_CONNECTION_URL:
    process.env.CI || process.env.LOCAL
      ? "ws://127.0.0.1:8900"
      : "wss://api.devnet.solana.com",
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
  TOKEN_NAME: "Test Identity",
  TOKEN_METADATA: "https://example.com/metadata.json",
  TOKEN_SYMBOL: "TESTID",
};

interface Client {
  rpc: Rpc<SolanaRpcApi>;
  rpcSubscriptions: RpcSubscriptions<SolanaRpcSubscriptionsApi>;
}

async function setupWallets(client: Client) {
  try {
    const payer = await generateKeyPairSigner(); // or loadKeypairSignerFromFile(path.join(process.env.PAYER));
    const authorizedSigner1 = await generateKeyPairSigner();
    const authorizedSigner2 = await generateKeyPairSigner();
    const issuer = await generateKeyPairSigner();
    const testUser = await generateKeyPairSigner();

    // Only airdrop when running locally or in CI
    if (process.env.CI || process.env.LOCAL) {
      const airdrop = airdropFactory({
        rpc: client.rpc,
        rpcSubscriptions: client.rpcSubscriptions,
      });
      const airdropTx: Signature = await airdrop({
        commitment: "processed",
        lamports: lamports(BigInt(1_000_000_000)),
        recipientAddress: payer.address,
      });
      await airdrop({
        commitment: "processed",
        lamports: lamports(BigInt(1_000_000_000)),
        recipientAddress: issuer.address,
      });
      await airdrop({
        commitment: "processed",
        lamports: lamports(BigInt(1_000_000_000)),
        recipientAddress: testUser.address,
      });

      console.log(`    - Airdrop completed: ${airdropTx}`);
    }
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
): Promise<Signature> {
  try {
    const simulationTx = await pipe(
      await createDefaultTransaction(client, payer),
      (tx) => appendTransactionMessageInstructions(instructions, tx),
    );
    const estimateCompute = estimateComputeUnitLimitFactory({
      rpc: client.rpc,
    });
    const computeUnitLimit = await estimateCompute(simulationTx);
    const signature = await pipe(
      await createDefaultTransaction(client, payer, computeUnitLimit),
      (tx) => appendTransactionMessageInstructions(instructions, tx),
      (tx) => signAndSendTransaction(client, tx),
    );
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
  client: Client;
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

async function verifyTokenAttestation({
  client,
  schemaPda,
  userAddress,
}: {
  client: Client;
  schemaPda: Address;
  userAddress: Address;
}) {
  try {
    const schema = await fetchSchema(client.rpc, schemaPda);

    const [attestationPda] = await deriveAttestationPda({
      credential: schema.data.credential,
      schema: schemaPda,
      nonce: userAddress,
    });
    const [attestationMint] = await deriveAttestationMintPda({
      attestation: attestationPda,
    });
    const mintAccount = await fetchMint(client.rpc, attestationMint);
    if (!mintAccount) return false;
    if (mintAccount.data.extensions.__option === "None") {
      return false;
    }
    const { value: foundExtensions } = mintAccount.data.extensions;

    // Verify member of group
    const [schemaMint] = await deriveSchemaMintPda({
      schema: schemaPda,
    });
    const tokenGroupMember = foundExtensions.find(
      (ext) => ext.__kind === "TokenGroupMember",
    );
    if (!tokenGroupMember) return false;
    if (tokenGroupMember.group !== schemaMint) return false;

    // Verify token metadata
    const tokenMetadata = foundExtensions.find(
      (ext) => ext.__kind === "TokenMetadata",
    );
    if (!tokenMetadata) return false;

    // Verify attestation PDA matches
    const attestationInMetadata =
      tokenMetadata.additionalMetadata.get("attestation");
    if (attestationInMetadata !== attestationPda) return false;

    // Verify schema PDA matches
    const schemaInMetadata = tokenMetadata.additionalMetadata.get("schema");
    if (schemaInMetadata !== schemaPda) return false;

    return true;
  } catch {
    return false;
  }
}

async function main() {
  console.log("Starting Solana Attestation Service Demo\n");

  const client: Client = {
    rpc: createSolanaRpc(CONFIG.HTTP_CONNECTION_URL),
    rpcSubscriptions: createSolanaRpcSubscriptions(CONFIG.WSS_CONNECTION_URL),
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

  // Step 4: Tokenize Schema
  console.log("\n4. Tokenizing Schema...");
  const [schemaMint] = await deriveSchemaMintPda({
    schema: schemaPda,
  });
  const sasPda = await deriveSasAuthorityAddress();
  const schemaMintAccountSpace = getMintSize([
    {
      __kind: "GroupPointer",
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

  await sendAndConfirmInstructions(
    client,
    payer,
    [createTokenizeSchemaInstruction],
    "Schema tokenized",
  );
  console.log(`    - Schema Mint: ${schemaMint}`);

  // Step 5: Create Tokenized Attestation
  console.log("\n5. Creating Tokenized Attestation...");
  const [attestationPda] = await deriveAttestationPda({
    credential: credentialPda,
    schema: schemaPda,
    nonce: testUser.address,
  });
  const [attestationMint] = await deriveAttestationMintPda({
    attestation: attestationPda,
  });

  const schema = await fetchSchema(client.rpc, schemaPda);
  const expiryTimestamp =
    Math.floor(Date.now() / 1000) +
    CONFIG.ATTESTATION_EXPIRY_DAYS * 24 * 60 * 60;
  const [recipientTokenAccount] = await findAssociatedTokenPda({
    mint: attestationMint,
    owner: testUser.address,
    tokenProgram: TOKEN_2022_PROGRAM_ADDRESS,
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
      name: CONFIG.TOKEN_NAME,
      symbol: CONFIG.TOKEN_SYMBOL,
      uri: CONFIG.TOKEN_METADATA,
      additionalMetadata: new Map([
        ["attestation", attestationPda],
        ["schema", schemaPda],
      ]),
    },
    {
      __kind: "TokenGroupMember",
      group: schemaMint,
      mint: attestationMint,
      memberNumber: 1,
    },
  ]);

  const createTokenizedAttestationInstruction =
    await getCreateTokenizedAttestationInstruction({
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
      name: CONFIG.TOKEN_NAME,
      uri: CONFIG.TOKEN_METADATA,
      symbol: CONFIG.TOKEN_SYMBOL,
      mintAccountSpace: attestationMintAccountSpace,
      recipientTokenAccount: recipientTokenAccount,
      associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ADDRESS,
      tokenProgram: TOKEN_2022_PROGRAM_ADDRESS,
    });

  await sendAndConfirmInstructions(
    client,
    payer,
    [createTokenizedAttestationInstruction],
    "Tokenized attestation created",
  );
  console.log(`    - Attestation PDA: ${attestationPda}`);
  console.log(`    - Attestation Mint: ${attestationMint}`);

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

  // Step 7: Verify Attestation Token
  console.log("\n7. Verifying Attestation Token...");
  const isTokenVerified = await verifyTokenAttestation({
    client,
    schemaPda,
    userAddress: testUser.address,
  });
  console.log(
    `    - Test User's token is ${isTokenVerified ? "verified" : "not verified"}`,
  );

  // Step 8: Close Tokenized Attestation
  console.log("\n8. Closing Tokenized Attestations...");
  const eventAuthority = await deriveEventAuthorityAddress();

  const closeTokenizedAttestationInstruction =
    getCloseTokenizedAttestationInstruction({
      payer,
      authority: authorizedSigner1,
      credential: credentialPda,
      attestation: attestationPda,
      eventAuthority,
      attestationProgram: SOLANA_ATTESTATION_SERVICE_PROGRAM_ADDRESS,
      attestationMint,
      sasPda,
      attestationTokenAccount: recipientTokenAccount,
      tokenProgram: TOKEN_2022_PROGRAM_ADDRESS,
    });
  await sendAndConfirmInstructions(
    client,
    payer,
    [closeTokenizedAttestationInstruction],
    "Tokenized Attestation closed",
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
