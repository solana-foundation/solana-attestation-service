export const CONFIG = {
    // CLUSTER_OR_RPC: 'devnet',
    CLUSTER_OR_RPC: 'localnet',
    CREDENTIAL_NAME: 'LIT-ENCRYPTED-ATTESTATIONS',
    SCHEMA_NAME: 'LIT-ENCRYPTED-METADATA',
    SCHEMA_LAYOUT: Buffer.from([12, 12, 12]),
    SCHEMA_FIELDS: ["ciphertext", "dataToEncryptHash", "accessControlConditions"],
    SCHEMA_VERSION: 1,
    SCHEMA_DESCRIPTION: 'Schema for Lit Protocol encrypted attestation metadata with access control conditions',
    ATTESTATION_EXPIRY_DAYS: 365
};

export const ORIGINAL_DATA = {
    name: "test-user",
    age: 100,
    country: "usa",
};