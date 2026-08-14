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
  const filesToPreserve = ['tsconfig.json', '.npmignore', 'pnpm-lock.yaml', 'Cargo.toml'];
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

// The renderer takes the package folder, writes to its src/generated, and syncs
// the dependency ranges below into clients/typescript/package.json on every run —
// so bumping kit means editing them here rather than in the manifest.
sasCodama.accept(
  renderJavaScriptVisitor(typescriptClientsDir, {
    formatCode: true,
    deleteFolderBeforeRendering: true,
    dependencyVersions: {
      "@solana/kit": "^7.0.0",
      "@solana/program-client-core": "^7.0.0",
    },
    prettierOptions: {
      arrowParens: "always",
      printWidth: 80,
      semi: true,
      singleQuote: true,
      tabWidth: 2,
      trailingComma: "es5",
      useTabs: false,
    },
  }),
);

// Restore configuration files after generation
configPreserver.restore();
