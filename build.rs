// In build.rs
use std::path::PathBuf;

fn main() {
    rsbinder_aidl::Builder::new()
        .source(PathBuf::from("aidl/pion.aidl"))
        .output(PathBuf::from("pion.rs"))
        .generate()
        .unwrap();

    rsbinder_aidl::Builder::new()
        .source(PathBuf::from("aidl/echo_service.aidl"))
        .output(PathBuf::from("echo_service.rs"))
        .generate()
        .unwrap();
}
