fn main() {
    println!("cargo::rerun-if-changed=kernels/ker_x64_scalar.s");
    println!("cargo::rerun-if-changed=kernels/ker_x64_complex.s");
    // Use the `cc` crate to build a C file and statically link it.
    cc::Build::new()
        .file("kernels/ker_scalar.s")
        .file("kernels/ker_complex.s")
        .file("kernels/ker_f64x4_complex.s")
        .compile("x64");
}
