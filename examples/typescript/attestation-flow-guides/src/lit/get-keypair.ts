import { existsSync, mkdirSync } from "fs";
import { generateExtractableKeyPairSigner } from "gill";
import { loadKeypairSignerFromFile, saveKeypairSignerToFile } from "gill/node";
import path from "path";

async function generateKeypair(outputPath: string) {
    const extractableSigner = await generateExtractableKeyPairSigner();
    await saveKeypairSignerToFile(extractableSigner, outputPath);
}

async function getKeyPair(keyPairName: string) {
    const keyPairDir = 'key-pairs';
    const keyPairPath = path.join(keyPairDir, `${keyPairName}.keypair.json`);
    if (!existsSync(keyPairPath)) {
        // Ensure the directory exists before creating the file
        if (!existsSync(keyPairDir)) {
            mkdirSync(keyPairDir, { recursive: true });
        }
        console.log(`${keyPairName} keypair does not exist at path: ${keyPairPath}. Generating it...`);
        await generateKeypair(keyPairPath);
    }

    return loadKeypairSignerFromFile(keyPairPath);
}

export async function getIssuerKeypair() {
    const keypair = await getKeyPair("issuer");
    console.log(`Got Issuer keypair with address: ${keypair.address}`);
    return keypair;
}

export async function getAuthorizedSigner1Keypair() {
    const keypair = await getKeyPair("authorized-signer-1");
    console.log(`Got Authorized Signer 1 keypair with address: ${keypair.address}`);
    return keypair;
}

export async function getAuthorizedSigner2Keypair() {
    const keypair = await getKeyPair("authorized-signer-2");
    console.log(`Got Authorized Signer 2 keypair with address: ${keypair.address}`);
    return keypair;
}
