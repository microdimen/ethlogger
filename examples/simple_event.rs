extern crate ethlogger;
extern crate web3;
#[macro_use]
extern crate log;
extern crate env_logger;

use ethlogger::native_contract::NativeContractBuilder;
use ethlogger::native_contract::INativeCallback;
use web3::types::Log;
use std::time;
use std::thread;
use log::LevelFilter;

pub fn main() {
    env_logger::Builder::default().filter_module("simple_event", LevelFilter::Debug).init();

    let abi_path = "./contract/simple.abi";
    let host = "http://127.0.0.1:8545";
    let contract_address = "f08c24beb893b56aaf55b60f665fbee1c97bc61c";
    let mut instance = NativeContractBuilder::new(
        abi_path,
        contract_address,
        host,
    ).build().unwrap();
    instance.init().unwrap();
    debug!("all events: {:?}", instance.get_all_events());

    struct CB {};
    impl INativeCallback for CB {
        fn on_new_log(&self, content: Log) {
            debug!("new log: {:?}", content);
        }
    }

    debug!("start logging");
    instance.log_event_by_name("Count", vec![], Box::new(CB {})).unwrap();

    thread::sleep(time::Duration::from_secs(3));

    debug!("stop all logging");
    instance.stop_all().unwrap();

    thread::sleep(time::Duration::from_secs(3));
}
