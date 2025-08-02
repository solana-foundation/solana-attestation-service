import { deriveCredentialPda } from "sas-lib";

import { SAS_STANDARD_CONFIG, SAS_TOKENIZED_CONFIG } from "./constants";
import { getIssuerKeypair } from "./get-keypair";

async function main() {
    console.log("Starting credential PDA generator\n");

    let config;
    if (process.env.SAS_TYPE === "standard") {
        config = SAS_STANDARD_CONFIG;
    } else if (process.env.SAS_TYPE === "tokenized") {
        config = SAS_TOKENIZED_CONFIG;
    } else {
        throw new Error("SAS_TYPE is not set. Please set the SAS_TYPE environment variable to 'standard' or 'tokenized'.");
    }

    // Step 1: Get issuer keypair
    const issuer = await getIssuerKeypair();

    // Step 2: Create Credential
    console.log("\n2. Creating Credential...");
    const [credentialPda] = await deriveCredentialPda({
        authority: issuer.address,
        name: config.CREDENTIAL_NAME
    });

    console.log(`Credential PDA: ${credentialPda}`);
}

main()
    .then(() => console.log("\nCredential PDA generated successfully!"))
    .catch((error) => {
        console.error("❌ Failed to generate credential PDA:", error);
        process.exit(1);
    });