use std::{env, path::PathBuf};

static COMMON_CUDA_PATHS: &[&str] = &[
    "/opt/cuda",                 // default Arch Linux location
    "/usr/lib/x86_64-linux-gnu", // default Ubuntu location
    "/usr/local/cuda",
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

    let sdk_samples_out_dir = cmake::Config::new("Video_Codec_SDK_11.0.10/Samples/NvCodec")
        .define("CMAKE_C_COMPILER", "gcc-8")
        .define("CMAKE_CXX_COMPILER", "g++-8")
        .build();

    println!(
        "cargo:rustc-link-search=native={}/lib",
        sdk_samples_out_dir.display()
    );
    println!("cargo:rustc-link-lib=static=NvCodec");

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

    let bindings = bindgen::Builder::default()
        .header("wrapper.h")
        .clang_arg("-IVideo_Codec_SDK_11.0.10/Interface")
        .clang_arg("-IVideo_Codec_SDK_11.0.10/Samples/NvCodec/NvDecoder")
        .clang_arg("-IVideo_Codec_SDK_11.0.10/Samples/NvCodec/NvEncoder")
        .clang_arg("-IVideo_Codec_SDK_11.0.10/Samples/Utils")
        .clang_args(&["-x", "c++"])
        .clang_arg(format!("-I{}", cuda_include.to_string_lossy()))
        .blocklist_item("std::basic_.*stream_sentry.*")
        .enable_cxx_namespaces()
        .respect_cxx_access_specs(true)
        .allowlist_type("Nv(En|De)coder")
        .constified_enum_module("cudaVideoCodec_enum.*")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .generate()
        .expect("Unable to generate bindings");

    // Write the bindings to the $OUT_DIR/bindings.rs file.
    let out_path = PathBuf::from(out_dir());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
