//! Codama IDL build script.
//!
//! Runs only when `GENERATE_IDL` is set, so ordinary builds do not rewrite the
//! committed IDL. `pnpm run generate-idl` sets it.

use {
    codama::Codama,
    std::{env, fs, path::Path},
};

const PROGRAM_NAME: &str = "solana_attestation_service";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("cargo:rerun-if-changed=src/");
    println!("cargo:rerun-if-env-changed=GENERATE_IDL");

    if env::var_os("GENERATE_IDL").is_none() {
        return Ok(());
    }

    generate_idl()
}

fn generate_idl() -> Result<(), Box<dyn std::error::Error>> {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR")?;
    let crate_path = Path::new(&manifest_dir).join("src");
    let codama = Codama::load(&crate_path)?;

    let mut idl: serde_json::Value = serde_json::from_str(&codama.get_json_idl()?)?;
    if let Some(program) = idl.get_mut("program").and_then(serde_json::Value::as_object_mut) {
        program.insert("name".to_string(), serde_json::Value::String(PROGRAM_NAME.to_string()));
    }
    let mut json = serde_json::to_string_pretty(&idl)?;
    json.push('\n');

    let idl_dir = Path::new(&manifest_dir).parent().unwrap().join("idl");
    fs::create_dir_all(&idl_dir)?;
    let idl_path = idl_dir.join(format!("{PROGRAM_NAME}.json"));
    fs::write(&idl_path, json)?;

    println!("cargo:warning=IDL written to: {}", idl_path.display());
    Ok(())
}
