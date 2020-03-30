use vergen::{generate_cargo_keys, ConstantsFlags};

fn main() {
    generate_cargo_keys(ConstantsFlags::SHA_SHORT).unwrap();
}
