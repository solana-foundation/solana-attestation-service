const codama = require("codama");
const anchorIdl = require("@codama/nodes-from-anchor");
const path = require("path");
const renderers = require("@codama/renderers");
const { renderVisitor: renderJavaScriptVisitor } = require("@codama/renderers-js");
const fs = require("fs");

const TOKEN_2022_PROGRAM_ID = 'TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb';
const SAS_PROGRAM_ID = '22zoJMtdu4tQc2PzL74ZUT7FrwgB1Udec8DdW4yw4BdG';
const ATA_PROGRAM_ID = 'ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL';
const EVENT_AUTHORITY_PDA = 'DzSpKpST2TSyrxokMXchFz3G2yn5WEGoxzpGEUDjCX4g';

const projectRoot = path.join(__dirname, "..");
const idlDir = path.join(projectRoot, "idl");
const sasIdl = require(path.join(idlDir, "solana_attestation_service.json"));
const rustClientsDir = path.join(__dirname, "..", "clients", "rust");
const typescriptClientsDir = path.join(
  __dirname,
  "..",
  "clients",
  "typescript",
);

function preserveConfigFiles() {
  const filesToPreserve = ['tsconfig.json', '.npmignore', 'pnpm-lock.yaml', 'Cargo.toml'];
  const preservedFiles = new Map();

  filesToPreserve.forEach(filename => {
    const filePath = path.join(typescriptClientsDir, filename);
    const tempPath = path.join(typescriptClientsDir, `${filename}.temp`);

    if (fs.existsSync(filePath)) {
      fs.copyFileSync(filePath, tempPath);
      preservedFiles.set(filename, tempPath);
    }
  });

  return {
    restore: () => {
      preservedFiles.forEach((tempPath, filename) => {
        const filePath = path.join(typescriptClientsDir, filename);
        if (fs.existsSync(tempPath)) {
          fs.copyFileSync(tempPath, filePath);
          fs.unlinkSync(tempPath);
        }
      });
    }
  };
}

const sasCodama = codama.createFromRoot(anchorIdl.rootNodeFromAnchor(sasIdl));
sasCodama.update(
  codama.bottomUpTransformerVisitor([
    // add 1 byte discriminator
    {
      select: "[accountNode]",
      transform: (node) => {
        codama.assertIsNode(node, "accountNode");

        return {
          ...node,
          data: {
            ...node.data,
            fields: [
              codama.structFieldTypeNode({
                name: "discriminator",
                type: codama.numberTypeNode("u8"),
              }),
              ...node.data.fields,
            ],
          },
        };
      },
    },
  ]),
);

// Seeds mirror `program/src/constants.rs`. Every schema created by the
// CreateSchema instruction is written at version 1; later versions are minted by
// ChangeSchemaVersion, which is why only CreateSchema can resolve its own PDA.
const SCHEMA_INITIAL_VERSION = 1;

const programName = sasCodama.getRoot().program.name;

sasCodama.update(
  codama.addPdasVisitor({
    [programName]: [
      codama.pdaNode({
        name: 'credential',
        seeds: [
          codama.constantPdaSeedNodeFromString('utf8', 'credential'),
          codama.variablePdaSeedNode('authority', codama.publicKeyTypeNode()),
          codama.variablePdaSeedNode('name', codama.stringTypeNode('utf8')),
        ],
      }),
      codama.pdaNode({
        name: 'schema',
        seeds: [
          codama.constantPdaSeedNodeFromString('utf8', 'schema'),
          codama.variablePdaSeedNode('credential', codama.publicKeyTypeNode()),
          codama.variablePdaSeedNode('name', codama.stringTypeNode('utf8')),
          codama.variablePdaSeedNode('version', codama.numberTypeNode('u8')),
        ],
      }),
      codama.pdaNode({
        name: 'attestation',
        seeds: [
          codama.constantPdaSeedNodeFromString('utf8', 'attestation'),
          codama.variablePdaSeedNode('credential', codama.publicKeyTypeNode()),
          codama.variablePdaSeedNode('schema', codama.publicKeyTypeNode()),
          codama.variablePdaSeedNode('nonce', codama.publicKeyTypeNode()),
        ],
      }),
      codama.pdaNode({
        name: 'schemaMint',
        seeds: [
          codama.constantPdaSeedNodeFromString('utf8', 'schemaMint'),
          codama.variablePdaSeedNode('schema', codama.publicKeyTypeNode()),
        ],
      }),
      codama.pdaNode({
        name: 'attestationMint',
        seeds: [
          codama.constantPdaSeedNodeFromString('utf8', 'attestationMint'),
          codama.variablePdaSeedNode('attestation', codama.publicKeyTypeNode()),
        ],
      }),
      codama.pdaNode({
        name: 'eventAuthority',
        seeds: [codama.constantPdaSeedNodeFromString('utf8', '__event_authority')],
      }),
      codama.pdaNode({
        name: 'sasAuthority',
        seeds: [codama.constantPdaSeedNodeFromString('utf8', 'sas')],
      }),
    ],
  }),
);

sasCodama.update(
  codama.setInstructionAccountDefaultValuesVisitor([
    {
      account: 'tokenProgram',
      defaultValue: codama.publicKeyValueNode(TOKEN_2022_PROGRAM_ID)
    },
    {
      account: 'attestationProgram',
      defaultValue: codama.publicKeyValueNode(SAS_PROGRAM_ID)
    },
    {
      account: 'associatedTokenProgram',
      defaultValue: codama.publicKeyValueNode(ATA_PROGRAM_ID)
    },
    {
      // The Rust renderer cannot resolve PDAs, so a pdaValueNode here would
      // leave Rust callers with no default at all. The literal address is the
      // `eventAuthority` PDA and is covered by a client test.
      account: 'eventAuthority',
      defaultValue: codama.publicKeyValueNode(EVENT_AUTHORITY_PDA)
    },
    {
      account: 'sasPda',
      defaultValue: codama.pdaValueNode('sasAuthority')
    },
    {
      instruction: 'createCredential',
      account: 'credential',
      defaultValue: codama.pdaValueNode('credential', [
        codama.pdaSeedValueNode('authority', codama.accountValueNode('authority')),
        codama.pdaSeedValueNode('name', codama.argumentValueNode('name')),
      ]),
    },
    {
      instruction: 'createSchema',
      account: 'schema',
      defaultValue: codama.pdaValueNode('schema', [
        codama.pdaSeedValueNode('credential', codama.accountValueNode('credential')),
        codama.pdaSeedValueNode('name', codama.argumentValueNode('name')),
        codama.pdaSeedValueNode('version', codama.numberValueNode(SCHEMA_INITIAL_VERSION)),
      ]),
    },
    {
      instruction: 'tokenizeSchema',
      account: 'mint',
      defaultValue: codama.pdaValueNode('schemaMint', [
        codama.pdaSeedValueNode('schema', codama.accountValueNode('schema')),
      ]),
    },
    {
      instruction: 'createAttestation',
      account: 'attestation',
      defaultValue: codama.pdaValueNode('attestation', [
        codama.pdaSeedValueNode('credential', codama.accountValueNode('credential')),
        codama.pdaSeedValueNode('schema', codama.accountValueNode('schema')),
        codama.pdaSeedValueNode('nonce', codama.argumentValueNode('nonce')),
      ]),
    },
    {
      instruction: 'createTokenizedAttestation',
      account: 'attestation',
      defaultValue: codama.pdaValueNode('attestation', [
        codama.pdaSeedValueNode('credential', codama.accountValueNode('credential')),
        codama.pdaSeedValueNode('schema', codama.accountValueNode('schema')),
        codama.pdaSeedValueNode('nonce', codama.argumentValueNode('nonce')),
      ]),
    },
    {
      instruction: 'createTokenizedAttestation',
      account: 'schemaMint',
      defaultValue: codama.pdaValueNode('schemaMint', [
        codama.pdaSeedValueNode('schema', codama.accountValueNode('schema')),
      ]),
    },
    {
      account: 'attestationMint',
      defaultValue: codama.pdaValueNode('attestationMint', [
        codama.pdaSeedValueNode('attestation', codama.accountValueNode('attestation')),
      ]),
    },
  ]),
);

const u32 = codama.numberTypeNode("u32");
const prefixedStringType = codama.sizePrefixTypeNode(
  codama.stringTypeNode("utf8"),
  u32,
);
const schemaDataTypeLink = codama.definedTypeLinkNode("schemaDataType");

/** A byte blob holding a run of items that fills the blob exactly. */
const joinedRunType = (item) =>
  codama.sizePrefixTypeNode(
    codama.arrayTypeNode(item, codama.remainderCountNode()),
    u32,
  );

// Variant order defines the discriminants and must match the `SchemaDataTypes`
// enum in `program/src/state/schema.rs`.
const SCHEMA_DATA_TYPE_VARIANTS = [
  "u8", "u16", "u32", "u64", "u128",
  "i8", "i16", "i32", "i64", "i128",
  "bool", "char", "string",
  "vecU8", "vecU16", "vecU32", "vecU64", "vecU128",
  "vecI8", "vecI16", "vecI32", "vecI64", "vecI128",
  "vecBool", "vecChar", "vecString",
];

const INSTRUCTIONS_TAKING_LAYOUT = ["createSchema", "changeSchemaVersion"];

const configPreserver = preserveConfigFiles();

sasCodama.accept(
  renderers.renderRustVisitor(path.join(rustClientsDir, "src", "generated"), {
    formatCode: true,
    crateFolder: rustClientsDir,
    deleteFolderBeforeRendering: true,
  }),
);

// Everything below shapes the TypeScript client only, and so runs after the Rust
// render. Two reasons it stays out of the Rust client: the Rust renderer discards
// the size prefix wrapping a remainder-count array and emits `RemainderVec`, which
// reads to the end of the buffer and so cannot represent an interior blob; and the
// Rust caller's layouts come from `SchemaStructSerialize` as raw bytes, so typing
// them as an enum would only add conversions. The Rust client keeps `Vec<u8>`.
//
// The Schema account stores `name`, `description`, `layout` and `field_names` as
// opaque length-prefixed byte blobs (see `program/src/state/schema.rs`), but the
// blobs have known internal structure: the two text fields are UTF-8, `layout` is
// a run of SchemaDataTypes discriminants, and `field_names` is a run of
// u32-length-prefixed strings. Describing that structure makes the generated
// codecs decode straight to `string`, `SchemaDataType[]` and `string[]`.
const SCHEMA_FIELD_TYPES = {
  description: prefixedStringType,
  fieldNames: joinedRunType(prefixedStringType),
  layout: joinedRunType(schemaDataTypeLink),
  name: prefixedStringType,
};

sasCodama.update(
  codama.bottomUpTransformerVisitor([
    {
      select: "[programNode]",
      transform: (node) => ({
        ...node,
        definedTypes: [
          ...node.definedTypes,
          codama.definedTypeNode({
            name: "schemaDataType",
            type: codama.enumTypeNode(
              SCHEMA_DATA_TYPE_VARIANTS.map((variant) =>
                codama.enumEmptyVariantTypeNode(variant),
              ),
            ),
          }),
        ],
      }),
    },
    // A layout argument is typed as a byte blob, which leaves callers writing raw
    // SchemaDataTypes discriminants. A u32-prefixed array of the named enum has
    // the same wire format and matches what reading a Schema back yields.
    ...INSTRUCTIONS_TAKING_LAYOUT.map((instruction) => ({
      select: `[instructionNode]${instruction}.[instructionArgumentNode]layout`,
      transform: (node) => {
        codama.assertIsNode(node, "instructionArgumentNode");

        return {
          ...node,
          type: codama.arrayTypeNode(
            schemaDataTypeLink,
            codama.prefixedCountNode(u32),
          ),
        };
      },
    })),
    {
      select: "[accountNode]schema",
      transform: (node) => {
        codama.assertIsNode(node, "accountNode");

        return {
          ...node,
          data: {
            ...node.data,
            fields: node.data.fields.map((field) =>
              field.name in SCHEMA_FIELD_TYPES
                ? { ...field, type: SCHEMA_FIELD_TYPES[field.name] }
                : field,
            ),
          },
        };
      },
    },
  ]),
);

// The renderer takes the package folder, writes to its src/generated, and syncs
// the dependency ranges below into clients/typescript/package.json on every run —
// so bumping kit means editing them here rather than in the manifest.
sasCodama.accept(
  renderJavaScriptVisitor(typescriptClientsDir, {
    formatCode: true,
    deleteFolderBeforeRendering: true,
    dependencyVersions: {
      "@solana/kit": "^7.0.0",
    },
    // `@solana/kit` re-exports the program client core helpers on a subpath.
    // Importing them from there keeps kit as the client's only external
    // package, matching the @solana-program clients.
    dependencyMap: {
      solanaProgramClientCore: "@solana/kit/program-client-core",
    },
    prettierOptions: {
      arrowParens: "always",
      printWidth: 80,
      semi: true,
      singleQuote: true,
      tabWidth: 2,
      trailingComma: "es5",
      useTabs: false,
    },
  }),
);

// Restore configuration files after generation
configPreserver.restore();
