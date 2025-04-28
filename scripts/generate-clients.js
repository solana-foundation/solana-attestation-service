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

// Function to preserve package.json
function preservePackageJson() {
  const packageJsonPath = path.join(typescriptClientsDir, "package.json");
  const tempPackageJsonPath = path.join(typescriptClientsDir, "package.json.temp");
  
  if (fs.existsSync(packageJsonPath)) {
    fs.copyFileSync(packageJsonPath, tempPackageJsonPath);
  }
  
  return {
    restore: () => {
      if (fs.existsSync(tempPackageJsonPath)) {
        fs.copyFileSync(tempPackageJsonPath, packageJsonPath);
        fs.unlinkSync(tempPackageJsonPath);
      }
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

// Preserve package.json before Rust client generation
const packageJsonPreserver = preservePackageJson();

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

// Restore package.json after generation
packageJsonPreserver.restore();
