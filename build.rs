use std::env;
use std::path::PathBuf;

use copy_to_output::copy_to_output_path;

fn main() {
    let deps_path = PathBuf::from("deps")
        .canonicalize()
        .expect("Couldn't find deps directory");

    println!("cargo:rustc-link-search={}", deps_path.to_str().unwrap());
    println!("cargo:rustc-link-lib=dylib=SG_Com");

    std::env::set_var(
        "LIBCLANG_PATH",
        "C:\\Program Files\\Microsoft Visual Studio\\2022\\Community\\VC\\Tools\\Llvm\\x64\\bin\\",
    );
    let bindings = bindgen::Builder::default()
        .header(deps_path.join("SG_Com.h").to_str().unwrap())
        .merge_extern_blocks(true)
        .rustified_non_exhaustive_enum(".*")
        .derive_default(true)
        .no_debug("SG_AnimationNodeInfo")
        .use_core()
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .generate()
        .expect("Unable to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");

    match copy_to_output_path(
        &deps_path.join("SG_Com.dll"),
        &std::env::var("PROFILE").unwrap(),
    ) {
        Ok(_) => (),
        Err(e) => eprintln!("Error copying SG_Com.dll: {}", e),
    }
}
