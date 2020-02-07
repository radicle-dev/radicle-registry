use substrate_wasm_builder_runner::WasmBuilder;

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

    WasmBuilder::new()
        .with_current_project()
        .with_wasm_builder_from_crates("1.0.9")
        .export_heap_base()
        .import_memory()
        .build()
}
