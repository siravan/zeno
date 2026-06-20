fn main() {
    println!("cargo::rerun-if-changed=kernels/ker_x64_scalar.s");
    // Use the `cc` crate to build a C file and statically link it.
    cc::Build::new().file("kernels/ker_x64_scalar.s").compile("test");
}
