extern crate bindgen;

use std::env;
use std::path::PathBuf;

static COMMON_CUDA_PATHS: &[&str] = &[
    "/opt/cuda",       // default Arch Linux location
    "/usr/local/cuda", // default Ubuntu location
];

fn find_cuda_dir(env_key: &'static str) -> PathBuf {
    if let Some(val) = env::var_os(env_key) {
        return PathBuf::from(&val);
    }

    COMMON_CUDA_PATHS
        .iter()
        .find(|cuda_path| {
            matches!(std::fs::metadata(cuda_path), Ok(f) if f.is_dir())
        })
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            println!("cargo:warning=cuda path not found, please set with CUDA_INCLUDE_PATH if you installed in a non-standard location.");
            PathBuf::from(COMMON_CUDA_PATHS[0])
        })
}

fn out_dir() -> PathBuf {
    std::env::var("OUT_DIR")
        .expect("OUT_DIR environment var not set.")
        .into()
}

fn main() {
    let cuda_include = find_cuda_dir("CUDA_INCLUDE_PATH").join("include");
    for p in COMMON_CUDA_PATHS {
        println!("cargo:rustc-link-search={}/lib64", p)
    }

    println!("cargo:rustc-link-search=Video_Codec_SDK_11.0.10/Lib/linux/stubs/x86_64");
    println!("cargo:rustc-link-lib=dylib=cuda");
    println!("cargo:rustc-link-lib=dylib=cudart");
    println!("cargo:rustc-link-lib=dylib=nvcuvid");
    println!("cargo:rustc-link-lib=dylib=nvidia-encode");

    if cfg!(target_os = "macos") {
        println!("cargo:rustc-link-lib=dylib=c++");
    } else {
        println!("cargo:rustc-link-lib=dylib=stdc++");
        println!("cargo:rustc-link-lib=dylib=gcc");
    }

    // Tell cargo to invalidate the built crate whenever the wrapper changes
    println!("cargo:rerun-if-changed=wrapper.h");

    let bindings = bindgen::Builder::default()
        .header("wrapper.h")
        .clang_arg("-IVideo_Codec_SDK_11.0.10/Interface")
        .clang_arg(format!("-I{}", cuda_include.to_string_lossy()))
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .generate()
        .expect("Unable to generate bindings");

    // Write the bindings to the $OUT_DIR/bindings.rs file.
    let out_path = PathBuf::from(out_dir());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
