fn main() {
    println!("cargo:rustc-link-search=native=src/bpf");
    println!("cargo:rustc-link-lib=loader"); // provided in project
    println!("cargo:rustc-link-lib=bpf");  // provided by OS
}