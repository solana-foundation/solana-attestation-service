import { existsSync } from "node:fs";
import { resolve } from "node:path";

import { assert } from "chai";
import {
  Address,
  createClient,
  generateKeyPairSigner,
  isSolanaError,
  KeyPairSigner,
  SOLANA_ERROR__INSTRUCTION_ERROR__CUSTOM,
} from "@solana/kit";
import { surfpool } from "@solana/surfpool/kit";

import {
  deserializeAttestationData,
  fetchAttestation,
  fetchSchema,
  findAttestationPda,
  findCredentialPda,
  findSchemaPda,
  getChangeSchemaVersionInstruction,
  getCloseAttestationInstruction,
  getCreateAttestationInstructionAsync,
  getCreateCredentialInstructionAsync,
  getCreateSchemaInstructionAsync,
  Schema,
  SchemaDataType,
  serializeAttestationData,
  SOLANA_ATTESTATION_SERVICE_PROGRAM_ADDRESS,
} from "../../src";

const PROGRAM_SO_PATH = resolve(
  __dirname,
  "../../../../target/deploy/solana_attestation_service.so"
);

/** `AttestationServiceError::InvalidAttestationData` in program/src/error.rs */
const INVALID_ATTESTATION_DATA_ERROR_CODE = 6;

/**
 * Transaction failures arrive wrapped in several layers of `SolanaError`, so
 * walk the cause chain for the program's own custom error code.
 */
const findCustomProgramErrorCode = (error: unknown): number | undefined => {
  const queue: unknown[] = [error];
  while (queue.length > 0) {
    const current = queue.shift();
    if (!(current instanceof Error)) {
      continue;
    }
    if (isSolanaError(current, SOLANA_ERROR__INSTRUCTION_ERROR__CUSTOM)) {
      return current.context.code;
    }
    const { cause, context } = current as {
      cause?: unknown;
      context?: { cause?: unknown };
    };
    queue.push(cause, context?.cause);
  }
  return undefined;
};

const CREDENTIAL_NAME = "surfpool-credential";
const SCHEMA_NAME = "surfpool-schema";
const SCHEMA_DESCRIPTION = "schema used by the surfpool integration test";

const SCHEMA_LAYOUT = [
  SchemaDataType.String,
  SchemaDataType.U8,
  SchemaDataType.Char,
  SchemaDataType.VecString,
];
const SCHEMA_FIELD_NAMES = ["name", "age", "grade", "tags"];

type Client = Awaited<ReturnType<typeof startClient>>;

const startClient = async () => {
  if (!existsSync(PROGRAM_SO_PATH)) {
    throw new Error(
      `Program binary not found at ${PROGRAM_SO_PATH}. Run \`cargo-build-sbf\` from the repository root first.`
    );
  }

  const client = await createClient().use(surfpool());
  client.surfnet.deploy({
    programId: SOLANA_ATTESTATION_SERVICE_PROGRAM_ADDRESS,
    soPath: PROGRAM_SO_PATH,
  });

  return client;
};

const createCredential = async (client: Client, authority: KeyPairSigner) => {
  const [credential] = await findCredentialPda({
    authority: authority.address,
    name: CREDENTIAL_NAME,
  });

  await client.sendTransaction(
    await getCreateCredentialInstructionAsync({
      authority,
      name: CREDENTIAL_NAME,
      payer: client.payer,
      signers: [authority.address],
    })
  );

  return credential;
};

const createSchema = async (
  client: Client,
  authority: KeyPairSigner,
  credential: Address,
  name: string,
  layout: SchemaDataType[],
  fieldNames: string[],
  description = SCHEMA_DESCRIPTION
) => {
  const [schema] = await findSchemaPda({ credential, name, version: 1 });

  await client.sendTransaction(
    await getCreateSchemaInstructionAsync({
      authority,
      credential,
      description,
      fieldNames,
      layout,
      name,
      payer: client.payer,
    })
  );

  return schema;
};

const createAttestation = async (
  client: Client,
  authority: KeyPairSigner,
  credential: Address,
  schema: Address,
  nonce: Address,
  data: Uint8Array
) =>
  client.sendTransaction(
    await getCreateAttestationInstructionAsync({
      authority,
      credential,
      data,
      expiry: 0,
      nonce,
      payer: client.payer,
      schema,
    })
  );

describe("Surfpool", () => {
  let client: Client;
  let authority: KeyPairSigner;
  let credential: Address;
  let schema: Address;
  let onchainSchema: Schema;

  before(async function () {
    this.timeout(120_000);

    client = await startClient();
    authority = await generateKeyPairSigner();

    credential = await createCredential(client, authority);
    schema = await createSchema(
      client,
      authority,
      credential,
      SCHEMA_NAME,
      SCHEMA_LAYOUT,
      SCHEMA_FIELD_NAMES
    );
    onchainSchema = (await fetchSchema(client.rpc, schema)).data;
  });

  after(() => {
    client?.surfnet.stop();
  });

  it("decodes the metadata, layout and field names the program wrote", async () => {
    assert.equal(onchainSchema.name, SCHEMA_NAME);
    assert.equal(onchainSchema.description, SCHEMA_DESCRIPTION);
    assert.deepEqual(onchainSchema.layout, SCHEMA_LAYOUT);
    assert.deepEqual(onchainSchema.fieldNames, SCHEMA_FIELD_NAMES);
  });

  it("round trips multi-byte text through the Schema metadata and field names", async () => {
    // Every text field the Schema stores is prefixed with a byte length rather
    // than a character count, so non-ASCII names must survive the round trip.
    const name = "surfpool-schéma-📋";
    const description = "descripción with ünicode";
    const fieldNames = ["名前", "âge", "país-🌍"];

    const unicodeSchema = await createSchema(
      client,
      authority,
      credential,
      name,
      [SchemaDataType.String, SchemaDataType.U8, SchemaDataType.String],
      fieldNames,
      description
    );
    const decoded = (await fetchSchema(client.rpc, unicodeSchema)).data;

    assert.equal(decoded.name, name);
    assert.equal(decoded.description, description);
    assert.deepEqual(decoded.fieldNames, fieldNames);

    const data = { "名前": "アダ", "âge": 36, "país-🌍": "españa" };
    assert.deepEqual(
      deserializeAttestationData(
        decoded,
        serializeAttestationData(decoded, data)
      ),
      data
    );
  });

  it("carries a new layout through ChangeSchemaVersion", async () => {
    const name = "surfpool-schema-versioned";
    const existingSchema = await createSchema(
      client,
      authority,
      credential,
      name,
      [SchemaDataType.String],
      ["name"]
    );

    const layout = [
      SchemaDataType.String,
      SchemaDataType.VecU64,
      SchemaDataType.Bool,
    ];
    const fieldNames = ["name", "scores", "active"];
    const [newSchema] = await findSchemaPda({ credential, name, version: 2 });

    await client.sendTransaction(
      getChangeSchemaVersionInstruction({
        authority,
        credential,
        existingSchema,
        fieldNames,
        layout,
        newSchema,
        payer: client.payer,
      })
    );

    const decoded = (await fetchSchema(client.rpc, newSchema)).data;
    assert.equal(decoded.version, 2);
    assert.deepEqual(decoded.layout, layout);
    assert.deepEqual(decoded.fieldNames, fieldNames);

    const data = { name: "Ada", scores: [1n, 2n, 3n], active: true };
    assert.deepEqual(
      deserializeAttestationData(
        decoded,
        serializeAttestationData(decoded, data)
      ),
      data
    );
  });

  it("creates an Attestation from data serialized against the onchain Schema", async () => {
    const nonce = (await generateKeyPairSigner()).address;
    const data = {
      name: "Ada Lovelace",
      age: 36,
      grade: "A",
      tags: ["engineer", "mathematician"],
    };

    await createAttestation(
      client,
      authority,
      credential,
      schema,
      nonce,
      serializeAttestationData(onchainSchema, data)
    );

    const [attestation] = await findAttestationPda({
      credential,
      nonce,
      schema,
    });
    const account = await fetchAttestation(client.rpc, attestation);

    assert.deepEqual(
      deserializeAttestationData(
        onchainSchema,
        Uint8Array.from(account.data.data)
      ),
      data
    );
  });

  it("round trips every supported layout type through the program", async () => {
    const layout = Array.from(
      { length: 26 },
      (_, index) => index as SchemaDataType
    );
    const fieldNames = Array.from({ length: 26 }, (_, index) => `f${index}`);
    const wideSchema = await createSchema(
      client,
      authority,
      credential,
      "surfpool-schema-wide",
      layout,
      fieldNames
    );
    const onchainWideSchema = (await fetchSchema(client.rpc, wideSchema)).data;

    const data = {
      f0: 1, f1: 2, f2: 3, f3: 4n, f4: 5n,
      f5: -1, f6: -2, f7: -3, f8: -4n, f9: -5n,
      f10: true, f11: "🔥", f12: "hello",
      f13: [1, 2], f14: [3, 4], f15: [5, 6], f16: [7n, 8n], f17: [9n, 10n],
      f18: [-1, -2], f19: [-3, -4], f20: [-5, -6], f21: [-7n, -8n],
      f22: [-9n, -10n], f23: [true, false], f24: ["a", "b"], f25: ["x", "y"],
    };

    const nonce = (await generateKeyPairSigner()).address;
    await createAttestation(
      client,
      authority,
      credential,
      wideSchema,
      nonce,
      serializeAttestationData(onchainWideSchema, data)
    );

    const [attestation] = await findAttestationPda({
      credential,
      nonce,
      schema: wideSchema,
    });
    const account = await fetchAttestation(client.rpc, attestation);

    assert.deepEqual(
      deserializeAttestationData(
        onchainWideSchema,
        Uint8Array.from(account.data.data)
      ),
      data
    );
  });

  it("rejects data that does not match the Schema layout", async () => {
    const nonce = (await generateKeyPairSigner()).address;
    const serialized = serializeAttestationData(onchainSchema, {
      name: "Ada Lovelace",
      age: 36,
      grade: "A",
      tags: ["engineer"],
    });
    const withTrailingByte = Uint8Array.from([...serialized, 0]);

    try {
      await createAttestation(
        client,
        authority,
        credential,
        schema,
        nonce,
        withTrailingByte
      );
      assert.fail("Expected the program to reject the Attestation data");
    } catch (error) {
      assert.equal(
        findCustomProgramErrorCode(error),
        INVALID_ATTESTATION_DATA_ERROR_CODE
      );
    }
  });

  it("closes an Attestation", async () => {
    const nonce = (await generateKeyPairSigner()).address;
    const data = { name: "Grace Hopper", age: 45, grade: "B", tags: ["navy"] };

    await createAttestation(
      client,
      authority,
      credential,
      schema,
      nonce,
      serializeAttestationData(onchainSchema, data)
    );

    const [attestation] = await findAttestationPda({
      credential,
      nonce,
      schema,
    });

    await client.sendTransaction(
      getCloseAttestationInstruction({
        attestation,
        authority,
        credential,
        payer: client.payer,
      })
    );

    const { value } = await client.rpc.getAccountInfo(attestation).send();
    assert.isNull(value);
  });
});
