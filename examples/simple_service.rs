use env_logger::Env;
use pion::*;
use rsbinder::{status::Result, *};
use std::path::Path;

// Include the code hello.rs generated from AIDL.
include!(concat!(env!("OUT_DIR"), "/echo_service.rs"));

// Set up to use the APIs provided in the code generated for Client and Service.
use crate::org::stardustxr::pion::IEchoService::*;

struct EchoService {}
impl Interface for EchoService {}
impl IEchoService for EchoService {
    fn echo(&self, text: &str) -> Result<String> {
        Ok(text.to_string())
    }
}

fn main() {
    env_logger::Builder::from_env(Env::default().default_filter_or("warn")).init();

    // Initialize ProcessState with the default binder path and the default max threads.
    let process = ProcessState::init_default();

    let file_path = Path::new("/tmp/binder_test.bind");
    let file = std::fs::File::create(file_path).unwrap();
    file.lock().unwrap();

    let pion = BpPion::from_binder(process.context_object().unwrap()).unwrap();

    let echo_service = BnEchoService::new_binder(EchoService {});

    let fd = ParcelFileDescriptor::new(file);

    IPion::register(&pion, &fd, &echo_service.as_binder()).unwrap();

    let raw_echo_service = IPion::exchange(&pion, &fd).unwrap();

    let echo = BpEchoService::from_binder(raw_echo_service).unwrap();

    assert_eq!(
        IEchoService::echo(&echo, "Hello, world!"),
        Ok("Hello, world!".to_string())
    );

    ProcessState::join_thread_pool().unwrap();
}
