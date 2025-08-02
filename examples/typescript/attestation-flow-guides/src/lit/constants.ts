const BASE_CONFIG = {
    // Network configuration 
    CLUSTER_OR_RPC: 'localnet',

    // Basic SAS Information
    SCHEMA_LAYOUT: Buffer.from([12, 12]),
    SCHEMA_FIELDS: ["ciphertext", "dataToEncryptHash"],
    SCHEMA_VERSION: 1,
}

export const SAS_STANDARD_CONFIG = {
    ...BASE_CONFIG,

    CREDENTIAL_NAME: 'LIT-ENCRYPTED-ATTESTATIONS',
    SCHEMA_NAME: 'LIT-ENCRYPTED-METADATA',
    SCHEMA_DESCRIPTION: 'Schema for Lit Protocol encrypted attestation metadata with access control conditions',
    ATTESTATION_EXPIRY_DAYS: 365
};

export const SAS_TOKENIZED_CONFIG = {
    ...BASE_CONFIG,

    CREDENTIAL_NAME: 'LIT-ENCRYPTED-TOKEN-ATTESTATIONS',
    SCHEMA_NAME: 'LIT-ENCRYPTED-TOKEN-METADATA',
    SCHEMA_DESCRIPTION: 'Schema for Lit Protocol encrypted tokenized attestation metadata',
    ATTESTATION_EXPIRY_DAYS: 365,

    // Token Metadata
    TOKEN_NAME: "Encrypted Identity Token",
    TOKEN_METADATA: "https://example.com/encrypted-metadata.json",
    TOKEN_SYMBOL: "EID",
};

export const ORIGINAL_DATA = {
    name: "test-user",
    age: 100,
    country: "usa",
};