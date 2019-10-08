use wasm_builder_runner::{build_current_project_with_rustflags, WasmBuilderSource};

fn main() {
    std::env::set_var("BUILD_DUMMY_WASM_BINARY", "1");
    build_current_project_with_rustflags(
        "wasm_binary.rs",
        WasmBuilderSource::Crates("1.0.7"),
        // This instructs LLD to export __heap_base as a global variable, which is used by the
        // external memory allocator.
        "-Clink-arg=--export=__heap_base",
    );
}
