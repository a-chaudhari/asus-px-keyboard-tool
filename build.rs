use std::env;
use std::path::PathBuf;
use libbpf_cargo::SkeletonBuilder;
// use libbpf_cargo::util::CargoWarningFormatter;

fn main() {
    // let () = tracing_subscriber::fmt()
    //     .event_format(CargoWarningFormatter)
    //     .init();

    let out = PathBuf::from(
        env::var_os("CARGO_MANIFEST_DIR")
            .expect("CARGO_MANIFEST_DIR must be set in build script"),
    )
        .join("src")
        .join("bpf")
        .join("hid_modify.skel.rs");

    SkeletonBuilder::new()
        .source("src/bpf/hid_modify.bpf.c")
        .build_and_generate(&out)
        .unwrap();

    println!("cargo:rerun-if-changed=src/bpf/hid_modify.bpf.c");
    println!("cargo:rerun-if-changed=src/bpf/hid_modify.bpf.h");
}