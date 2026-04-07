use std::{env, fs, path::PathBuf};

const COMMON_CUDA_PATHS: &[&str] = &[
    "/opt/cuda",       // default Arch Linux location
    "/usr/local/cuda", // default Ubuntu location
];

const NV_CODEC_PATH: &str = "/usr/include/nvidia-sdk";

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

    let build_dir = out_dir().join("build");
    fs::create_dir_all(&build_dir).expect("couldn't create cmake build dir");

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

    let function_pointer_structs = [
        "_CUVIDPARSERPARAMS",
        "_CUVIDSOURCEPARAMS",
        "_NV_ENCODE_API_FUNCTION_LIST",
        "CUDA_HOST_NODE_PARAMS_st",
        "CUDA_HOST_NODE_PARAMS_v2_st",
    ];

    let bindings = bindgen::Builder::default()
        .header("wrapper.h")
        .clang_arg(format!("-I{NV_CODEC_PATH}"))
        .clang_args(&["-x", "c++"])
        .clang_arg(format!("-I{}", cuda_include.to_string_lossy()))
        .constified_enum_module("CUvideopacketflags")
        .newtype_enum(".*")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .derive_default(true)
        .derive_partialeq(true)
        // NOTE: These structs contain function pointers, comparisons are not meaningful.
        .no_partialeq(function_pointer_structs.join("|"))
        .derive_debug(true)
        // NOTE: `cuMemBatchDecompressAsync` in `cuda.h` has a comment that produces invalid docs.
        // TODO: disable only that one particular comment.
        .generate_comments(false)
        .generate()
        .expect("Unable to generate bindings");

    // Write the bindings to the $OUT_DIR/bindings.rs file.
    let out_path = out_dir();
    bindings.write_to_file(out_path.join("bindings.rs")).expect("Couldn't write bindings!");
}
