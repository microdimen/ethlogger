#![recursion_limit = "128"]
#![feature(alloc)]

#![feature(box_syntax)]
#![feature(try_trait)]
extern crate ethabi;
extern crate rustc_hex;
#[macro_use]
extern crate error_chain;
extern crate ethereum_types;
extern crate tokio_core;
#[macro_use]
extern crate futures;
extern crate tokio_timer;
extern crate serde;
extern crate alloc;
extern crate serde_json;
#[macro_use]
extern crate log;
extern crate web3;

pub mod token_checker;
pub mod errors;
pub mod native_contract;
pub mod util;
pub mod filter_stream_once;

