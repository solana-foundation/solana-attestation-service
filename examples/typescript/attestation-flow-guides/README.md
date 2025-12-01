# Solana Attestation Service TypeScript Examples

This repository contains the companion code for Solana Attestation Service (SAS) implementation guides to create, manage, verify, and close digital credentials on Solana. For more detailed explanations and step-by-step walkthroughs, see the comprehensive guides:

| Title | File | Guide | Description |
|-------|------|-------|-------------|
| Standard Attestation Demo | `src/gill/sas-standard-gill-demo.ts` | [How to Build Digital Credentials using Solana Attestation Service](https://attest.solana.com/docs/guides/ts/how-to-create-digital-credentials) | Basic credential and attestation workflow |
| Tokenized Attestation Demo | `src/gill/sas-tokenized-gill-demo.ts` | [How to Create Tokenized Credentials using Solana Attestation Service](https://attest.solana.com/docs/guides/ts/tokenized-attestations) | Create credentials as SPL tokens using Token-2022 |
| Compressed Attestation Demo | `src/gill/sas-compressed-gill-demo.ts` | - | Create compressed credentials using Light Protocol v2 |

Additionally, Solana Kit examples are provided in `src/kit`.

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

# Run the tokenized attestation demo with Gill
pnpm gill:tokenized

# Run the compressed attestation demo with Gill
pnpm gill:compressed

# Run the standard attestation demo with Kit
pnpm kit:standard

# Run the tokenized attestation demo with Kit
pnpm kit:tokenized

# Run the compressed attestation demo with Kit
pnpm kit:compressed
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

3. **Run the demos against local validator**:
   ```bash
   # Run the standard attestation demo with Gill (local)
   pnpm gill:standard:local

   # Run the tokenized attestation demo with Gill (local)
   pnpm gill:tokenized:local

   # Run the compressed attestation demo with Gill (local)
   pnpm gill:compressed:local

   # Run the standard attestation demo with Kit (local)
   pnpm kit:standard:local

   # Run the tokenized attestation demo with Kit (local)
   pnpm kit:tokenized:local

   # Run the compressed attestation demo with Kit (local)
   pnpm kit:compressed:local
   ```
