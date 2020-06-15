use vergen::{generate_cargo_keys, ConstantsFlags};

fn main() {
    generate_cargo_keys(ConstantsFlags::all()).unwrap();
}
