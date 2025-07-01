# Solana Attestation Service TypeScript Examples

This repository contains the companion code for Solana Attestation Service (SAS) implementation guides to create, manage, verify, and close digital credentials on Solana. For more detailed explanations and step-by-step walkthroughs, see the comprehensive guides:

| Title | File | Guide | Description |
|-------|------|-------|-------------|
| Standard Attestation Demo | `src/attestation-demo.ts` | [How to Build Digital Credentials using Solana Attestation Service](https://attest.solana.com/docs/guides/ts/how-to-create-digital-credentials) | Basic credential and attestation workflow |
| Tokenized Attestation Demo | `src/tokenized-attestation-demo.ts` | [How to Create Tokenized Credentials using Solana Attestation Service](https://attest.solana.com/docs/guides/ts/tokenized-attestations) | Create credentials as SPL tokens using Token-2022 |

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

### Run on Devnet (Recommended)

The simplest way to get started is using Solana devnet:

```bash
# Run the standard attestation demo
pnpm demo:standard

# Run the tokenized attestation demo  
pnpm demo:tokenized
```

Both scripts will automatically:
- Create test wallets
- Request devnet SOL airdrops
- Execute the full attestation workflow

### Option 2: Local Development

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
   pnpm demo:standard
   # or
   pnpm demo:tokenized
   ```

## Available Scripts

| Script | Description |
|--------|-------------|
| `pnpm demo:standard` | Run the basic attestation workflow demo |
| `pnpm demo:tokenized` | Run the tokenized attestation demo with Token-2022 |
| `pnpm dump` | Download SAS program for local development |
| `pnpm start-local` | Start local test validator with SAS program |
| `pnpm build` | Compile TypeScript to JavaScript |
