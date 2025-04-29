const codama = require("codama");
const anchorIdl = require("@codama/nodes-from-anchor");
const path = require("path");
const renderers = require("@codama/renderers");
const fs = require("fs");

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
  const filesToPreserve = ['package.json', 'tsconfig.json', '.npmignore'];
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

const configPreserver = preserveConfigFiles();

sasCodama.accept(
  renderers.renderRustVisitor(path.join(rustClientsDir, "src", "generated"), {
    formatCode: true,
    crateFolder: rustClientsDir,
    deleteFolderBeforeRendering: true,
  }),
);

sasCodama.accept(
  renderers.renderJavaScriptVisitor(
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
