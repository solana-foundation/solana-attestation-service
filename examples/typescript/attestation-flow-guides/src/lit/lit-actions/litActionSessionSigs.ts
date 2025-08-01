// @ts-nocheck

const _litActionCode = async () => {
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
    LitActions.setResponse({
      response: JSON.stringify({
        success: false,
        message: "Error verifying signature.",
        error: error.toString(),
      }),
    });
  }

  try {
    const SIWS_AUTH_METHOD_TYPE = ethers.utils.keccak256(
      ethers.utils.toUtf8Bytes("Solana Attestation Service Lit Encryption Guide")
    );
    const usersAuthMethodId = ethers.utils.keccak256(
      ethers.utils.toUtf8Bytes(`siws:${siwsMessageJson.address}`)
    );

    const isPermitted = await Lit.Actions.isPermittedAuthMethod({
      tokenId: pkpTokenId,
      authMethodType: SIWS_AUTH_METHOD_TYPE,
      userId: ethers.utils.arrayify(usersAuthMethodId),
    });

    if (isPermitted) {
      console.log("Solana public key is authorized to use this PKP");
      return Lit.Actions.setResponse({ response: "true" });
    }

    console.log("Solana public key is not authorized to use this PKP");
    return Lit.Actions.setResponse({
      response: "false",
      reason: "Solana public key is not authorized to use this PKP",
    });
  } catch (error) {
    console.error("Error checking if authed sol pub key is permitted:", error);
    return LitActions.setResponse({
      response: JSON.stringify({
        success: false,
        message: "Error checking if authed sol pub key is permitted.",
        error: error.toString(),
      }),
    });
  }
};

export const litActionCode = `(${_litActionCode.toString()})()`;
