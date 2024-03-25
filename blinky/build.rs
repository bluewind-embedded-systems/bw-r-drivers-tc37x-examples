use std::{env, error::Error, path::PathBuf};

fn main() -> Result<(), Box<dyn Error>> {
    let target_arch = env::var("CARGO_CFG_TARGET_ARCH").unwrap();

    if target_arch != "tricore" {
        panic!("Only tricore target is supported")
    }

    // Linker script directory
    {
        let ld_dir = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").unwrap())
            .join("ld")
            .canonicalize()?;

        println!("cargo:rustc-link-search={}", ld_dir.display());
    }

    // Linker scripts
    {
        println!("cargo:rustc-link-arg=-Tdefmt.x");
        println!("cargo:rustc-link-arg=-Ttc37xA_memory.ld");
    }

    // Do not page align sections, link against static libraries
    println!("cargo:rustc-link-arg=--nmagic");

    Ok(())
}
