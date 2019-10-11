use wasm_builder_runner::{build_current_project_with_rustflags, WasmBuilderSource};

const DUMMY_WASM_BINARY_ENV: &str = "BUILD_DUMMY_WASM_BINARY";

fn main() {
    match std::env::var(DUMMY_WASM_BINARY_ENV) {
        Ok(ref val) if val == "0" || val == "false" => {
            std::env::remove_var(DUMMY_WASM_BINARY_ENV);
        }
        _ => {
            std::env::set_var(DUMMY_WASM_BINARY_ENV, "1");
        }
    }
    build_current_project_with_rustflags(
        "wasm_binary.rs",
        WasmBuilderSource::Crates("1.0.7"),
        // This instructs LLD to export __heap_base as a global variable, which is used by the
        // external memory allocator.
        "-Clink-arg=--export=__heap_base",
    );
}
