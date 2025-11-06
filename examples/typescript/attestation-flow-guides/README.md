# Solana Attestation Service TypeScript Examples

This repository contains the companion code for Solana Attestation Service (SAS) implementation guides to create, manage, verify, and close digital credentials on Solana. For more detailed explanations and step-by-step walkthroughs, see the comprehensive guides:

| Title                      | File                                  | Guide                                                                                                                                           | Description                                       |
| -------------------------- | ------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------- |
| Standard Attestation Demo  | `src/gill/sas-tokenized-gill-demo.ts` | [How to Build Digital Credentials using Solana Attestation Service](https://attest.solana.com/docs/guides/ts/how-to-create-digital-credentials) | Basic credential and attestation workflow         |
| Tokenized Attestation Demo | `src/gill/sas-standard-gill-demo.ts`  | [How to Create Tokenized Credentials using Solana Attestation Service](https://attest.solana.com/docs/guides/ts/tokenized-attestations)         | Create credentials as SPL tokens using Token-2022 |

Additionally, Solana Kit examples are provided in `src/kit`, and an example of using [Lit Protocol](https://developer.litprotocol.com/what-is-lit) to encrypt the attestation data is provided in `src/lit`.

## Requirements

- [Node.js](https://nodejs.org/) (v22 or later)
- [Solana CLI](https://solana.com/docs/intro/installation) (v2.2.x or greater)
- Package manager (e.g., [pnpm](https://pnpm.io/), [npm](https://docs.npmjs.com/downloading-and-installing-node-js-and-npm), or [yarn](https://classic.yarnpkg.com/en/docs/install))

## Installation

Clone the repository and install dependencies:

```bash
git clone https://github.com/solana-foundation/solana-attestation-service
cd examples/typescript/attestation-flow-guides
pnpm install
```

## Usage

### Run on Devnet

The simplest way to get started is using Solana devnet:

```bash
# Run the standard attestation demo with Gill
pnpm gill:standard

# Run the tokenized attestation demo will Gill
pnpm gill:tokenized

# Run the standard attestation demo with Kit
pnpm kit:standard

# Run the tokenized attestation demo will Kit
pnpm kit:tokenized
```

All of these scripts will automatically:
- Create test wallets
- Request devnet SOL airdrops
- Execute the full attestation workflow

### Local Development

For local development and testing:

1. **Download the SAS program:**
   ```bash
   pnpm dump
   ```

2. **Start local validator** (in a separate terminal):
   ```bash
   pnpm start-local
   ```

3. **Update configuration** in the demo files:
   ```typescript
   const CONFIG = {
       CLUSTER_OR_RPC: 'localnet', // Change from 'devnet'
       // ... rest of config
   };
   ```

4. **Run the demos:**
   ```bash
    # Run the standard attestation demo with Gill
    pnpm gill:standard

    # Run the tokenized attestation demo will Gill
    pnpm gill:tokenized

    # Run the standard attestation demo with Kit
    pnpm kit:standard

    # Run the tokenized attestation demo will Kit
    pnpm kit:tokenized
   ```

### Running the Lit Protocol examples

#### Prerequisites

##### Generate the Credential PDA

Unlike the Gill and Kit examples, the Lit examples require a known Credential PDA that used to query who are the authorized signers for a given attestation to determine who is allowed to decrypt the attestation data.

In order to know the Credential PDA, we'll need to generate the key pair used for the Attestation Issuer. Running the following command will generate and save the key pair, and use the key pair to create the Credential PDA.

There are two commands available depending on which Lit demo you'll be executing:

**NOTE**: By default, the key pairs generated for the Issuer (and Authorized Signers for the Credential) are saved in the `key-pairs/` directory.

- For running the standard attestation demo with Lit encryption:

   ```bash
   pnpm generate:credential-pda:standard
   ```

- For running the tokenized attestation demo with Lit encryption:

   ```bash
   pnpm generate:credential-pda:tokenized
   ```

After running either of the above commands, you will see output similar to the following:

```bash
Starting credential PDA generator

1. Getting Issuer keypair...
issuer keypair does not exist at path: key-pairs/issuer.json. Generating it...
Got Issuer keypair with address: DfcegVdXLB7wXEVjbEoYuXi6w9MWmMd8UmcGmMH2CdNZ

1. Creating Credential...
Credential PDA: HhU8FzCALX7XThBoPUQnJWAZMJdDeXctdxbedbT4kshV

Credential PDA generated successfully!
```

You'll need to copy the `Credential PDA` and paste it over the value for `AUTHORIZED_CREDENTIAL_PDA` in the [`src/lit/litActionDecrypt.ts`](src/lit/litActionDecrypt.ts) file. This PDA is what is used to verify that the requester of the Lit decryption request is authorized to decrypt the attestation data.

**NOTE**: The Credential PDA will be different depending on which example (`standard` or `tokenized`) you're running. Make sure to update the `AUTHORIZED_CREDENTIAL_PDA` value in the `litActionDecrypt.ts` file with the correct Credential PDA.

##### Local vs Devnet

If you're running the Lit examples against the Solana Devnet, you don't need to do anything else, and can move onto the [Running the Examples](#running-the-examples) section.

For running the Lit examples against a localnet, you'll need to update the `BASE_CONFIG` object in the [`src/lit/constants.ts`](src/lit/constants.ts) file and change the `CLUSTER_OR_RPC` to `localnet`:

```typescript
const BASE_CONFIG = {
    // Network configuration 
    CLUSTER_OR_RPC: 'localnet', // Change from 'devnet'
    // ... rest of config
};
```

Next:

1. **Download the SAS program:**
   ```bash
   pnpm dump
   ```

2. **Start local validator** (in a separate terminal):
   ```bash
   pnpm start-local
   ```

3. **Create a publicly accessible proxy for the local validator** (in a separate terminal):
   
   When you make the decryption request to Lit, the Lit nodes will need to be able to access your local validator to verify the requester of the decryption request is authorized to decrypt the attestation data.

   To give the Lit nodes access to your local validator, you'll need to create a publicly accessible proxy for the local validator. You can do this using various methods, but an easy way is by using the [ngrok CLI](https://ngrok.com/docs/getting-started/).

   Once installed, you can run the following command to create a publicly accessible proxy for the local validator:

   ```bash
   ngrok http 8899
   ```

   This will generate some output with the following key line:

   ```bash
   Forwarding                    https://47279cfb9aa7.ngrok-free.app -> http://localhost:8899 
   ```

   What's generated for your `https://...` URL should be copied and pasted over the value for `AUTHORIZED_RPC_URL` in the [`src/lit/litActionDecrypt.ts`](src/lit/litActionDecrypt.ts) file.

#### Running the Examples

##### Standard Attestation Demo

Run the standard attestation demo with Lit encryption using the following command:

```bash
pnpm lit:standard
```

These commands will generate a lot of output, but at the end you should see a summary that looks like the following:

```bash
================================================================================
SOLANA ATTESTATION SERVICE WITH LIT PROTOCOL ENCRYPTED ATTESTATION DEMO
================================================================================

📋 DEMO CONFIGURATION:
   Network: localnet
   Organization: LIT-ENCRYPTED-ATTESTATIONS
   Schema: LIT-ENCRYPTED-METADATA (v1)

🔑 CREATED ACCOUNTS:
   Credential PDA:    HhU8FzCALX7XThBoPUQnJWAZMJdDeXctdxbedbT4kshV
   Schema PDA:        HvLgB2BjQN3BzjQszYw6xuCCMpWPEkfs4b1YMrRBEp7H
   Attestation PDA:   41Tx4apNq6tocjY4FK4QXpZoTeybaPo1hK328SwvpLRf
   Test User:         JCRGgpTgaun9581uounx61RSmiMxN8KazEjdCnJbBdwB

🧪 VERIFICATION TEST RESULTS:
   Test User Verification:     ✅ PASSED
   Encrypted Metadata:
     - Ciphertext: plYShVUjPEdl69jWh54tjutxcBEjx7GiCYH1KKfnowhqUWOPkH...
     - Data Hash: 4efa888818ad5ba81b0e459037eb8c62a3f0bdf401de5372e21d8aa132a0a808
   Decrypted Attestation Data: {"name":"test-user","age":100,"country":"usa"}
   Random User Verification:   ✅ PASSED (correctly rejected)
   Unauthorized Signer Test:   ✅ PASSED (correctly rejected)
   ✅ ALL TESTS PASSED! Demo completed successfully.

================================================================================
```

##### Tokenized Attestation Demo

Run the tokenized attestation demo with Lit encryption using the following command:

```bash
pnpm lit:tokenized
```

These commands will generate a lot of output, but at the end you should see a summary that looks like the following:

```bash
================================================================================
SOLANA ATTESTATION SERVICE WITH LIT PROTOCOL ENCRYPTED ATTESTATION DEMO
================================================================================

📋 DEMO CONFIGURATION:
   Network: localnet
   Organization: LIT-ENCRYPTED-TOKEN-ATTESTATIONS
   Schema: LIT-ENCRYPTED-TOKEN-METADATA (v1)
   Token: Encrypted Identity Token (EID)

🔑 CREATED ACCOUNTS:
   Credential PDA:    7TLLV8HtYyzZ2GUN8tMRFvD9MEuAGdPm3wkCvk7wSsMG
   Schema PDA:        AhVoNT188zCgSwctKW3mEPBYTu9tu6AbR5VLSJFRdnLc
   Schema Mint:       CNJNEhh1xquFG1wAdtVEf6JvyN9uv7mb8TxE1qvqbJoC
   Attestation PDA:   HY8hFVPSqZ1mPS6efbGxF6GCjyCbD36pqWb7xF3PDq6V
   Attestation Mint:  tdTU7U7zeNpYZWuiscrkY9snYDKcnhv7BL196eRjH15
   Test User:         BoDK61hY1hvdbctyBexc22QkVJfqm4uAsfvUzWiD1o6Q

🧪 VERIFICATION TEST RESULTS:
   Test User Verification:     ✅ PASSED
   Encrypted Metadata:
     - Ciphertext: sH46l+sNSXgeX9qJn40USAglj9i4KlXjwqBMXpKzh5X+zXJwqd...
     - Data Hash: 4efa888818ad5ba81b0e459037eb8c62a3f0bdf401de5372e21d8aa132a0a808
   Decrypted Attestation Data: {"name":"test-user","age":100,"country":"usa"}
   Random User Verification:   ✅ PASSED (correctly rejected)
   Unauthorized Signer Test:   ✅ PASSED (correctly rejected)
   ✅ ALL TESTS PASSED! Demo completed successfully.

================================================================================
```

##### Understanding the Test Results

Three different tests are ran by the Lit demos:

- `Test User Verification`: This test verifies that the `Test User` the attestation was created for, actually has a verifiable attestation written on-chain.
  - `Encrypted Metadata` is the encrypted attestation data that was written on-chain for the `Test User`.
  - `Decrypted Attestation Data` is the attestation data that was decrypted by one of the Authorized Signers for the Credential.
- `Random User Verification`: This test verifies that a random user does not have a verifiable attestation written on-chain.
- `Unauthorized Signer Test`: This test verifies that an unauthorized signer is not able to decrypt the attestation data.
