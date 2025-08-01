import { existsSync, mkdirSync } from "fs";
import { generateExtractableKeyPairSigner } from "gill";
import { loadKeypairSignerFromFile, saveKeypairSignerToFile } from "gill/node";
import path from "path";

async function generateKeypair(outputPath: string) {
    console.log(`Generating Solana ${outputPath} keypair with Gill...`);

    const extractableSigner = await generateExtractableKeyPairSigner();
    await saveKeypairSignerToFile(extractableSigner, outputPath);

    console.log(`${outputPath} address: ${extractableSigner.address}`);
}

async function getKeyPair(keyPairName: string) {
    if (process.env.KEY_PAIR_DIR_PATH === undefined || process.env.KEY_PAIR_DIR_PATH === "") {
        throw new Error("KEY_PAIR_DIR_PATH is not set. Please set the KEY_PAIR_DIR_PATH environment variable to the path of where you'd like the issuer, authorized signer 1 and 2 to be stored.");
    }

    const keyPairDir = process.env.KEY_PAIR_DIR_PATH!;
    const keyPairPath = path.join(keyPairDir, `${keyPairName}.json`);
    if (!existsSync(keyPairPath)) {
        // Ensure the directory exists before creating the file
        if (!existsSync(keyPairDir)) {
            mkdirSync(keyPairDir, { recursive: true });
        }
        console.log(`${keyPairName} key file does not exist at path: ${keyPairPath}. Generating it...`);
        await generateKeypair(keyPairPath);
    }

    return loadKeypairSignerFromFile(keyPairPath);
}

export async function getIssuerKeypair() {
    return getKeyPair("issuer");
}

export async function getAuthorizedSigner1Keypair() {
    return getKeyPair("authorized-signer-1");
}

export async function getAuthorizedSigner2Keypair() {
    return getKeyPair("authorized-signer-2");
}
