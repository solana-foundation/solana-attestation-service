const codama = require("codama");
const anchorIdl = require("@codama/nodes-from-anchor");
const path = require("path");
const renderers = require("@codama/renderers");
const { renderVisitor: renderJavaScriptVisitor } = require("@codama/renderers-js");
const fs = require("fs");

const TOKEN_2022_PROGRAM_ID = 'TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb';
const SAS_PROGRAM_ID = '22zoJMtdu4tQc2PzL74ZUT7FrwgB1Udec8DdW4yw4BdG';
const ATA_PROGRAM_ID = 'ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL';
const EVENT_AUTHORITY_PDA = 'DzSpKpST2TSyrxokMXchFz3G2yn5WEGoxzpGEUDjCX4g';
// Light Protocol System Accounts
const LIGHT_SYSTEM_PROGRAM_ID = 'SySTEM1eSU2p4BGQfQpimFEWWSC1XDFeun3Nqzz3rT7';
const ACCOUNT_COMPRESSION_PROGRAM_ID = 'compr6CUsB5m2jS4Y3831ztGSTnDpnKJTKS95d64XVq';
const ACCOUNT_COMPRESSION_AUTHORITY_PDA = 'HwXnGK3tPkkVY6P439H2p68AxpeuWXd5PcrAxFpbmfbA';
const REGISTERED_PROGRAM_PDA = '35hkDgaAKwMCaxRz2ocSZ6NaUrtKkyNqU6c4RV3tYJRh';
const CPI_SIGNER_PDA = '4J8ZeHg2hqPTKv1ck9bqucEcbVDBS6fVZsNCuThmDHFC';
const ALLOWED_ADDRESS_TREE = 'amt2kaJA14v3urZbZvnc5v2np8jqvc4Z8zDep5wbtzx';

const projectRoot = path.join(__dirname, "..");
const idlDir = path.join(projectRoot, "idl");
const sasIdl = require(path.join(idlDir, "solana_attestation_service.json"));
const rustClientsDir = path.join(__dirname, "..", "clients", "rust");
const typescriptClientsDir = path.join(
  __dirname,
  "..",
  "clients",
  "typescript",
);

function preserveConfigFiles() {
  const filesToPreserve = ['package.json', 'tsconfig.json', '.npmignore', 'pnpm-lock.yaml', 'Cargo.toml'];
  const preservedFiles = new Map();

  filesToPreserve.forEach(filename => {
    const filePath = path.join(typescriptClientsDir, filename);
    const tempPath = path.join(typescriptClientsDir, `${filename}.temp`);

    if (fs.existsSync(filePath)) {
      fs.copyFileSync(filePath, tempPath);
      preservedFiles.set(filename, tempPath);
    }
  });

  return {
    restore: () => {
      preservedFiles.forEach((tempPath, filename) => {
        const filePath = path.join(typescriptClientsDir, filename);
        if (fs.existsSync(tempPath)) {
          fs.copyFileSync(tempPath, filePath);
          fs.unlinkSync(tempPath);
        }
      });
    }
  };
}

const sasCodama = codama.createFromRoot(anchorIdl.rootNodeFromAnchor(sasIdl));
sasCodama.update(
  codama.bottomUpTransformerVisitor([
    // add 1 byte discriminator
    {
      select: "[accountNode]",
      transform: (node) => {
        codama.assertIsNode(node, "accountNode");

        return {
          ...node,
          data: {
            ...node.data,
            fields: [
              codama.structFieldTypeNode({
                name: "discriminator",
                type: codama.numberTypeNode("u8"),
              }),
              ...node.data.fields,
            ],
          },
        };
      },
    },
  ]),
);

sasCodama.update(
  codama.setInstructionAccountDefaultValuesVisitor([
    {
      account: 'tokenProgram',
      defaultValue: codama.publicKeyValueNode(TOKEN_2022_PROGRAM_ID)
    },
    {
      account: 'attestationProgram',
      defaultValue: codama.publicKeyValueNode(SAS_PROGRAM_ID)
    },
    {
      account: 'associatedTokenProgram',
      defaultValue: codama.publicKeyValueNode(ATA_PROGRAM_ID)
    },
    {
      account: 'eventAuthority',
      defaultValue: codama.publicKeyValueNode(EVENT_AUTHORITY_PDA)
    },
    // Light Protocol System Accounts
    {
      account: 'lightSystemProgram',
      defaultValue: codama.publicKeyValueNode(LIGHT_SYSTEM_PROGRAM_ID)
    },
    {
      account: 'cpiSigner',
      defaultValue: codama.publicKeyValueNode(CPI_SIGNER_PDA)
    },
    {
      account: 'accountCompressionProgram',
      defaultValue: codama.publicKeyValueNode(ACCOUNT_COMPRESSION_PROGRAM_ID)
    },
    {
      account: 'accountCompressionAuthority',
      defaultValue: codama.publicKeyValueNode(ACCOUNT_COMPRESSION_AUTHORITY_PDA)
    },
    {
      account: 'registeredProgramPda',
      defaultValue: codama.publicKeyValueNode(REGISTERED_PROGRAM_PDA)
    },
    {
      account: 'addressMerkleTree',
      defaultValue: codama.publicKeyValueNode(ALLOWED_ADDRESS_TREE)
    }
  ]),
);

const configPreserver = preserveConfigFiles();

sasCodama.accept(
  renderers.renderRustVisitor(path.join(rustClientsDir, "src", "generated"), {
    formatCode: true,
    crateFolder: rustClientsDir,
    deleteFolderBeforeRendering: true,
  }),
);

sasCodama.accept(
  renderJavaScriptVisitor(
    path.join(typescriptClientsDir, "src", "generated"),
    {
      formatCode: true,
      crateFolder: typescriptClientsDir,
      deleteFolderBeforeRendering: true,
    },
  ),
);

// Restore configuration files after generation
configPreserver.restore();
