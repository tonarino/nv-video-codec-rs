extern crate bindgen;

use std::{env, fs, path::PathBuf};

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
    std::env::var("OUT_DIR").expect("OUT_DIR environment var not set.").into()
}

fn main() {
    let cuda_include = find_cuda_dir("CUDA_INCLUDE_PATH").join("include");
    for p in COMMON_CUDA_PATHS {
        println!("cargo:rustc-link-search={}/lib64", p)
    }

    let nvcodec_dir = env::current_dir().unwrap().join("Video_Codec_SDK_11.0.10/Samples/NvCodec");

    let build_dir = out_dir().join("build");
    fs::create_dir_all(&build_dir).expect("couldn't create cmake build dir");

    println!("cargo:rerun-if-changed={}", nvcodec_dir.display());
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
        // .clang_arg("-IVideo_Codec_SDK_11.0.10/Samples/Utils")
        .clang_args(&["-x", "c++"])
        .clang_arg(format!("-I{}", cuda_include.to_string_lossy()))
        .constified_enum_module("CUvideopacketflags")
        .newtype_enum(".*")
        // TODO: enable again if we can avoid blacklisting GUID symbols.
        //.allowlist_var("(?i)(.*cu.*|.*nv.*)")
        //.allowlist_type("(?i)(.*cu.*|.*nv.*)")
        //.allowlist_function("(?i)(.*cu.*|.*nv.*)")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .derive_default(true)
        // NOTE: This produces warnings about function pointer comparisons.
        .derive_partialeq(true)
        .derive_debug(true)
        .generate()
        .expect("Unable to generate bindings");

    // min working version
    /*
    let bindings = bindgen::Builder::default()
        .header("wrapper.h")
        .clang_arg("-IVideo_Codec_SDK_11.0.10/Interface")
        .clang_arg(format!("-I{}", cuda_include.to_string_lossy()))
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .generate()
        .expect("Unable to generate bindings");
    */

    // Write the bindings to the $OUT_DIR/bindings.rs file.
    let out_path = out_dir();
    bindings.write_to_file(out_path.join("bindings.rs")).expect("Couldn't write bindings!");
}
