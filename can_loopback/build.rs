use std::{env, error::Error, path::PathBuf};

fn main() -> Result<(), Box<dyn Error>> {
    let ld_dir = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").unwrap())
        .join("ld")
        .canonicalize()?;

    println!("cargo:rustc-link-search={}", ld_dir.display());

    println!("cargo:rustc-link-arg=--nmagic");
    println!("cargo:rustc-link-arg=-Tdefmt.x");
    println!("cargo:rustc-link-arg=-Ttc37xA_memory.ld");

    Ok(())
}
