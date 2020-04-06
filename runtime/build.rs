use std::env;
use std::ffi::OsStr;
use substrate_wasm_builder_runner::WasmBuilder;

const BUILD_WASM_BINARY_OUR_DIR_ENV: &str = "BUILD_WASM_BINARY_OUT_DIR";

fn main() {
    if let Some(wasm_binary_dir) = env::var_os(BUILD_WASM_BINARY_OUR_DIR_ENV) {
        build_wasm_runtime_in_dir(&wasm_binary_dir);
    }
}

fn build_wasm_runtime_in_dir(out_dir: impl AsRef<OsStr>) {
    env::set_var("WASM_TARGET_DIRECTORY", out_dir);
    env::set_var("WASM_BUILD_TYPE", "release");
    WasmBuilder::new()
        .with_current_project()
        .with_wasm_builder_from_crates("1.0.9")
        .export_heap_base()
        .import_memory()
        .build();
}
