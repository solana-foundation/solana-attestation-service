import { renderVisitor as renderJavaScriptVisitor } from '@codama/renderers-js';
import { renderVisitor as renderRustVisitor } from '@codama/renderers-rust';
import {
    arrayTypeNode,
    assertIsNode,
    bottomUpTransformerVisitor,
    createFromJson,
    definedTypeLinkNode,
    definedTypeNode,
    enumEmptyVariantTypeNode,
    enumTypeNode,
    numberTypeNode,
    prefixedCountNode,
    remainderCountNode,
    sizePrefixTypeNode,
    stringTypeNode,
    structFieldTypeNode,
    type TypeNode,
} from 'codama';
import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';

type Codama = ReturnType<typeof createFromJson>;

const __dirname = path.dirname(fileURLToPath(import.meta.url));

const projectRoot = path.join(__dirname, '..');
const idlPath = path.join(projectRoot, 'idl', 'solana_attestation_service.json');
const rustClientsDir = path.join(projectRoot, 'clients', 'rust');
const typescriptClientsDir = path.join(projectRoot, 'clients', 'typescript');

const readIdl = () => createFromJson(fs.readFileSync(idlPath, 'utf-8'));

// Accounts are written with a one-byte discriminator that the Rust structs do not
// carry as a field (see `program/src/state/discriminator.rs`), so the clients only
// decode correctly once it is prepended.
const DATA_ACCOUNTS = ['attestation', 'credential', 'schema'];

// `SchemaMint`, `AttestationMint`, `EventAuthority` and `SasAuthority` are declared
// in `program/src/constants.rs` only to carry their PDA seeds, because Codama has no
// way to describe a PDA without an account. Their PDA helpers are worth generating;
// decoders for an empty account are not.
function shapeAccounts(codama: Codama) {
    codama.update(
        bottomUpTransformerVisitor([
            {
                select: '[programNode]',
                transform: node => ({
                    ...node,
                    accounts: node.accounts.filter(account => (account.data.fields ?? []).length > 0),
                }),
            },
        ]),
    );

    codama.update(
        bottomUpTransformerVisitor(
            DATA_ACCOUNTS.map(account => ({
                select: `[accountNode]${account}`,
                transform: node => {
                    assertIsNode(node, 'accountNode');

                    return {
                        ...node,
                        data: {
                            ...node.data,
                            fields: [
                                structFieldTypeNode({ name: 'discriminator', type: numberTypeNode('u8') }),
                                ...node.data.fields,
                            ],
                        },
                    };
                },
            })),
        ),
    );
}

// Two Rust-only adjustments. Events are mirrored into defined types because the
// Rust renderer has no event support, which is what keeps
// `types::CloseAttestationEvent` available to Rust callers; the event's one-byte
// type discriminator is already a field of the struct, so the mirrored type matches
// the wire format on its own. And the account-to-PDA links are dropped, because a
// generated `find_pda` for a PDA with an unprefixed string seed pulls in the
// `spl-collections` crate for its `TrailingStr` type. The PDA nodes themselves
// stay, so nothing is lost from the IDL.
const rustCodama = readIdl();
shapeAccounts(rustCodama);
rustCodama.update(
    bottomUpTransformerVisitor([
        {
            select: '[accountNode]',
            transform: node => {
                assertIsNode(node, 'accountNode');
                const { pda: _pda, ...rest } = node;

                return rest;
            },
        },
        {
            select: '[programNode]',
            transform: node => ({
                ...node,
                definedTypes: [
                    ...(node.definedTypes ?? []),
                    ...(node.events ?? []).map(event => definedTypeNode({ name: event.name, type: event.data })),
                ],
            }),
        },
    ]),
);

rustCodama.accept(
    renderRustVisitor(rustClientsDir, {
        anchorTraits: false,
        deleteFolderBeforeRendering: true,
        formatCode: true,
        generatedFolder: 'src/generated',
    }),
);

// The TypeScript client is shaped separately. The Rust renderer discards the size
// prefix wrapping a remainder-count array and emits `RemainderVec`, which reads to
// the end of the buffer and so cannot represent an interior blob; and Rust callers
// pass layouts as raw type-identifier bytes, so typing them as an enum would only
// add conversions. The Rust client keeps `Vec<u8>`.
const tsCodama = readIdl();
shapeAccounts(tsCodama);

const u32 = numberTypeNode('u32');
const prefixedStringType = sizePrefixTypeNode(stringTypeNode('utf8'), u32);
const schemaDataTypeLink = definedTypeLinkNode('schemaDataType');

/** A byte blob holding a run of items that fills the blob exactly. */
const joinedRunType = (item: TypeNode) => sizePrefixTypeNode(arrayTypeNode(item, remainderCountNode()), u32);

// Variant order defines the discriminants and must match the `SchemaDataTypes`
// enum in `program/src/state/schema.rs`.
const SCHEMA_DATA_TYPE_VARIANTS = [
    'u8',
    'u16',
    'u32',
    'u64',
    'u128',
    'i8',
    'i16',
    'i32',
    'i64',
    'i128',
    'bool',
    'char',
    'string',
    'vecU8',
    'vecU16',
    'vecU32',
    'vecU64',
    'vecU128',
    'vecI8',
    'vecI16',
    'vecI32',
    'vecI64',
    'vecI128',
    'vecBool',
    'vecChar',
    'vecString',
];

const INSTRUCTIONS_TAKING_LAYOUT = ['createSchema', 'changeSchemaVersion'];

// The Schema account stores `name`, `description`, `layout` and `field_names` as
// opaque length-prefixed byte blobs (see `program/src/state/schema.rs`), but the
// blobs have known internal structure: the two text fields are UTF-8, `layout` is
// a run of SchemaDataTypes discriminants, and `field_names` is a run of
// u32-length-prefixed strings. Describing that structure makes the generated
// codecs decode straight to `string`, `SchemaDataType[]` and `string[]`.
const SCHEMA_FIELD_TYPES: Record<string, TypeNode> = {
    description: prefixedStringType,
    fieldNames: joinedRunType(prefixedStringType),
    layout: joinedRunType(schemaDataTypeLink),
    name: prefixedStringType,
};

tsCodama.update(
    bottomUpTransformerVisitor([
        {
            select: '[programNode]',
            transform: node => ({
                ...node,
                definedTypes: [
                    ...(node.definedTypes ?? []),
                    definedTypeNode({
                        name: 'schemaDataType',
                        type: enumTypeNode(SCHEMA_DATA_TYPE_VARIANTS.map(variant => enumEmptyVariantTypeNode(variant))),
                    }),
                ],
            }),
        },
        // A layout argument is typed as a byte blob, which leaves callers writing raw
        // SchemaDataTypes discriminants. A u32-prefixed array of the named enum has
        // the same wire format and matches what reading a Schema back yields.
        ...INSTRUCTIONS_TAKING_LAYOUT.map(instruction => ({
            select: `[instructionNode]${instruction}.[instructionArgumentNode]layout`,
            transform: node => {
                assertIsNode(node, 'instructionArgumentNode');

                return {
                    ...node,
                    type: arrayTypeNode(schemaDataTypeLink, prefixedCountNode(u32)),
                };
            },
        })),
        {
            select: '[accountNode]schema',
            transform: node => {
                assertIsNode(node, 'accountNode');

                return {
                    ...node,
                    data: {
                        ...node.data,
                        fields: node.data.fields.map(field =>
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
tsCodama.accept(
    renderJavaScriptVisitor(typescriptClientsDir, {
        deleteFolderBeforeRendering: true,
        // `@solana/kit` re-exports the program client core helpers on a subpath.
        // Importing them from there keeps kit as the client's only external
        // package, matching the @solana-program clients.
        dependencyMap: {
            solanaProgramClientCore: '@solana/kit/program-client-core',
        },
        dependencyVersions: {
            '@solana/kit': '^8.0.0',
        },
        formatCode: true,
        prettierOptions: {
            arrowParens: 'always',
            printWidth: 80,
            semi: true,
            singleQuote: true,
            tabWidth: 2,
            trailingComma: 'es5',
            useTabs: false,
        },
    }),
);
