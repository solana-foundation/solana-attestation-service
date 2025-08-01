import { deriveCredentialPda } from "sas-lib";

import { CONFIG } from "./constants";
import { getIssuerKeypair } from "./get-keypair";

async function main() {
    console.log("Starting credential PDA generator\n");

    // Step 1: Get issuer keypair
    const issuer = await getIssuerKeypair();

    // Step 2: Create Credential
    console.log("\n2. Creating Credential...");
    const [credentialPda] = await deriveCredentialPda({
        authority: issuer.address,
        name: CONFIG.CREDENTIAL_NAME
    });

    console.log(`Credential PDA: ${credentialPda}`);
}

main()
    .then(() => console.log("\nCredential PDA generated successfully!"))
    .catch((error) => {
        console.error("❌ Failed to generate credential PDA:", error);
        process.exit(1);
    });