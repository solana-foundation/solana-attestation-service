// @ts-nocheck

const _litActionCode = async () => {
  // Hardcoded values for this specific attestation service instance
  const AUTHORIZED_CREDENTIAL_PDA = "HMDTWtveFZYK4mdTN4b1XhRWHQa1r3JsLm8YDKkWV4GT";
  const AUTHORIZED_RPC_URL = "https://api.devnet.solana.com";
  const AUTHORIZED_PROGRAM_ID = "22zoJMtdu4tQc2PzL74ZUT7FrwgB1Udec8DdW4yw4BdG";

  async function fetchAccountData(rpcUrl, address) {
    try {
      const response = await fetch(rpcUrl, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({
          jsonrpc: "2.0",
          id: 1,
          method: "getAccountInfo",
          params: [address, { encoding: "base64", commitment: "confirmed" }]
        })
      });

      const data = await response.json();
      if (data.error) {
        throw new Error(`RPC error: ${data.error.message}`);
      }

      if (!data.result || !data.result.value) {
        throw new Error("Account not found");
      }

      const accountInfo = data.result.value;
      return {
        data: ethers.utils.base64.decode(accountInfo.data[0]),
        owner: accountInfo.owner
      };
    } catch (error) {
      console.error("Error fetching account data:", error);
      throw error;
    }
  }

  function parseCredentialAccount(data) {
    let offset = 0;

    // Skip discriminator (1 byte)
    offset += 1;

    // Read authority (32 bytes)
    const authority = ethers.utils.base58.encode(data.slice(offset, offset + 32));
    offset += 32;

    // Read name length (4 bytes, little-endian)
    const nameLength = new DataView(data.buffer, offset, 4).getUint32(0, true);
    offset += 4;

    // Skip name bytes
    offset += nameLength;

    // Read authorized signers length (4 bytes, little-endian)
    const signersLength = new DataView(data.buffer, offset, 4).getUint32(0, true);
    offset += 4;

    // Read authorized signers
    const authorizedSigners = [];
    for (let i = 0; i < signersLength; i++) {
      const signer = ethers.utils.base58.encode(data.slice(offset, offset + 32));
      authorizedSigners.push(signer);
      offset += 32;
    }

    return { authority, authorizedSigners };
  }

  function getSiwsMessage(siwsInput) {
    console.log("Attempting to parse SIWS message: ", siwsInput);

    if (!siwsInput.domain || !siwsInput.address) {
      throw new Error("Domain and address are required for Siws message construction");
    }

    // Start with the mandatory domain and address line
    let message = `${siwsInput.domain} wants you to sign in with your Solana account:\n${siwsInput.address}`;

    // Add statement if provided (with double newline separator)
    if (siwsInput.statement) {
      message += `\n\n${siwsInput.statement}`;
    }

    // Collect advanced fields in the correct order as per ABNF spec
    const fields = [];

    if (siwsInput.uri) {
      fields.push(`URI: ${siwsInput.uri}`);
    }
    if (siwsInput.version) {
      fields.push(`Version: ${siwsInput.version}`);
    }
    if (siwsInput.chainId) {
      fields.push(`Chain ID: ${siwsInput.chainId}`);
    }
    if (siwsInput.nonce) {
      fields.push(`Nonce: ${siwsInput.nonce}`);
    }
    if (siwsInput.issuedAt) {
      fields.push(`Issued At: ${siwsInput.issuedAt}`);
    }
    if (siwsInput.expirationTime) {
      fields.push(`Expiration Time: ${siwsInput.expirationTime}`);
    }
    if (siwsInput.notBefore) {
      fields.push(`Not Before: ${siwsInput.notBefore}`);
    }
    if (siwsInput.requestId) {
      fields.push(`Request ID: ${siwsInput.requestId}`);
    }
    if (siwsInput.resources && siwsInput.resources.length > 0) {
      fields.push(`Resources:`);
      for (const resource of siwsInput.resources) {
        fields.push(`- ${resource}`);
      }
    }

    // Add advanced fields if any exist (with double newline separator)
    if (fields.length > 0) {
      message += `\n\n${fields.join('\n')}`;
    }

    return message;
  }

  function validateSiwsMessage(siwsInput) {
    const now = new Date();

    // Check if message has expired (expirationTime is in the past)
    if (siwsInput.expirationTime) {
      const expirationTime = new Date(siwsInput.expirationTime);
      if (now > expirationTime) {
        return {
          valid: false,
          error: `SIWS message has expired. Current time: ${now.toISOString()}, Expiration: ${siwsInput.expirationTime}`
        };
      }
    }

    // Check if message is not yet valid (notBefore is in the future)
    if (siwsInput.notBefore) {
      const notBefore = new Date(siwsInput.notBefore);
      if (now < notBefore) {
        return {
          valid: false,
          error: `SIWS message is not yet valid. Current time: ${now.toISOString()}, Not Before: ${siwsInput.notBefore}`
        };
      }
    }

    return { valid: true };
  }

  async function verifySiwsSignature(
    siwsMessage,
    signerAddress,
    siwsMessageSignature
  ) {
    try {
      const publicKey = await crypto.subtle.importKey(
        "raw",
        ethers.utils.base58.decode(signerAddress),
        {
          name: "Ed25519",
          namedCurve: "Ed25519",
        },
        false,
        ["verify"]
      );

      const isValid = await crypto.subtle.verify(
        "Ed25519",
        publicKey,
        ethers.utils.base58.decode(siwsMessageSignature),
        new TextEncoder().encode(siwsMessage)
      );

      return isValid;
    } catch (error) {
      console.error("Error in verifySiwsSignature:", error);
      throw error;
    }
  }

  const siwsMessageJson = JSON.parse(siwsMessage);
  const siwsMessageString = getSiwsMessage(siwsMessageJson);

  try {
    const siwsMessageValid = validateSiwsMessage(siwsMessageJson);
    if (!siwsMessageValid.valid) {
      console.log("SIWS message validation failed:", siwsMessageValid.error);
      return LitActions.setResponse({
        response: JSON.stringify({
          success: false,
          message: "SIWS message validation failed.",
          error: siwsMessageValid.error,
        }),
      });
    }
  } catch (error) {
    console.error("Error in validateSiwsMessage:", error);
    return LitActions.setResponse({
      response: JSON.stringify({
        success: false,
        message: "Error in validateSiwsMessage.",
        error: error.toString(),
      }),
    });
  }

  try {
    const siwsSignatureValid = await verifySiwsSignature(
      siwsMessageString,
      siwsMessageJson.address,
      siwsMessageSignature
    );

    if (!siwsSignatureValid) {
      console.log("Signature is invalid.");
      return LitActions.setResponse({
        response: JSON.stringify({
          success: false,
          message: "Signature is invalid.",
        }),
      });
    }

    console.log("Signature is valid.");
  } catch (error) {
    console.error("Error verifying signature:", error);
    return LitActions.setResponse({
      response: JSON.stringify({
        success: false,
        message: "Error verifying signature.",
        error: error.toString(),
      }),
    });
  }

  // Fetch and verify authorized signers from the hardcoded credential
  try {
    const accountInfo = await fetchAccountData(AUTHORIZED_RPC_URL, AUTHORIZED_CREDENTIAL_PDA);

    // Verify the credential account is owned by the correct program
    if (accountInfo.owner !== AUTHORIZED_PROGRAM_ID) {
      console.log(`Credential PDA owner mismatch. Expected: ${AUTHORIZED_PROGRAM_ID}, Got: ${accountInfo.owner}`);
      return LitActions.setResponse({
        response: JSON.stringify({
          success: false,
          message: "Credential PDA is not owned by the authorized program",
          expectedOwner: AUTHORIZED_PROGRAM_ID,
          actualOwner: accountInfo.owner
        }),
      });
    }

    const credential = parseCredentialAccount(accountInfo.data);

    // Check if the signer is authorized
    const signerAddress = siwsMessageJson.address;
    if (!credential.authorizedSigners.includes(signerAddress)) {
      console.log(`Signer ${signerAddress} is not in authorized signers list`);
      return LitActions.setResponse({
        response: JSON.stringify({
          success: false,
          message: "Signer is not authorized to decrypt",
          authorizedSigners: credential.authorizedSigners,
          requestingSigner: signerAddress
        }),
      });
    }

    console.log(`Signer ${signerAddress} is authorized to decrypt`);
  } catch (error) {
    console.error("Error checking authorized signers:", error);
    return LitActions.setResponse({
      response: JSON.stringify({
        success: false,
        message: "Error checking authorized signers",
        error: error.toString(),
      }),
    });
  }

  try {
    const decryptedData = await Lit.Actions.decryptAndCombine({
      accessControlConditions: solRpcConditions,
      ciphertext,
      dataToEncryptHash,
      authSig: {
        sig: ethers.utils
          .hexlify(ethers.utils.base58.decode(siwsMessageSignature))
          .slice(2),
        derivedVia: "solana.signMessage",
        signedMessage: siwsMessageString,
        address: siwsMessageJson.address,
      },
      chain: "solana",
    });
    return LitActions.setResponse({ response: decryptedData });
  } catch (error) {
    console.error("Error decrypting data:", error);
    return LitActions.setResponse({
      response: JSON.stringify({
        success: false,
        message: "Error decrypting data.",
        error: error.toString(),
      }),
    });
  }
}

export const litActionCode = `(${_litActionCode.toString()})()`;
