use errors::*;
use ethabi::Contract;
use ethabi::EventParam;
use ethabi::RawTopicFilter;
use ethabi::Topic;
use ethereum_types::Address;
use filter_stream_once::FilterStreamOnce;
use futures::Future;
use futures::Sink;
use futures::stream::Stream;
use futures::sync::mpsc;
use futures::sync::mpsc::Sender;
use futures::sync::oneshot;
use std::collections::HashMap;
use std::fs;
use std::str::FromStr;
use std::thread;
use std::time;
use token_checker::check_type;
use token_checker::remove_0x;
use tokio_core::reactor::Core;
use tokio_core::reactor::Remote;
use util::web3_filter;
use web3;
use web3::types::Log;

#[derive(Debug, Clone)]
pub struct NativeContractBuilder {
    abi_path: String,
    contract_address: String,
    host: String,
}

impl NativeContractBuilder {
    fn check_param(&self) -> Result<Address> {
        let _ = fs::File::open(&self.abi_path)?;
        Address::from_str(remove_0x(&self.contract_address)).map_err(Error::from)
        // TODO: check host
    }

    pub fn build(self) -> Result<NativeContract> {
        let addr = self.check_param()?;
        Ok(NativeContract {
            abi_path: self.abi_path,
            contract_address: addr,
            host: self.host,
            ..Default::default()
        })
    }

    pub fn new(abi_path: &str, contract_address: &str, host: &str) -> Self {
        NativeContractBuilder {
            abi_path: abi_path.to_string(),
            contract_address: contract_address.to_string(),
            host: host.to_string(),
        }
    }

    pub fn set_abi_path(&mut self, value: &str) {
        self.abi_path = value.to_string();
    }

    pub fn set_host(&mut self, value: &str) {
        self.host = value.to_string();
    }
    pub fn set_contract_address(&mut self, value: &str) {
        self.contract_address = value.to_string();
    }
}

const CORE_RUNNER_NAME: &str = "core_runner_name";

#[derive(Default)]
pub struct NativeContract {
    abi_path: String,
    contract_address: Address,
    host: String,

    contract: Option<Contract>,
    events: Vec<NativeEvent>,
    eloop: Option<Remote>,
    signal_map: HashMap<String, mpsc::Sender<()>>,
}

impl NativeContract {
    pub fn get_abi_path(&self) -> &str {
        &self.abi_path
    }

    pub fn get_contract_address(&self) -> String {
        format!("{}", self.contract_address)
    }

    pub fn get_host(&self) -> &str {
        &self.host
    }

    fn check_inited(&self) -> Result<()> {
        self.contract.as_ref().ok_or(ErrorKind::NotInited.into()).map(|_| ())
    }

    pub fn init(&mut self) -> Result<()> {
        if self.contract.is_some() {
            return Ok(());
        }

        let source_file = fs::File::open(&self.abi_path)?;
        let contract: Contract = Contract::load(source_file)?;
        self.events = contract.events
            .iter()
            .map(|(k, v)| {
                let params = v.inputs.iter()
                    .map(|p| {
                        NativeParam {
                            name: p.name.clone(),
                            kind: format!("{}", p.kind),
                            indexed: p.indexed,
                        }
                    })
                    .collect();
                NativeEvent {
                    name: k.clone(),
                    params,
                }
            })
            .collect();
        self.contract = Some(contract);
        Ok(())
    }

    pub fn get_all_events(&self) -> Result<Vec<NativeEvent>> {
        self.check_inited()?;

        Ok(self.events.clone())
    }

    pub fn get_event_by_id(&self, id: usize) -> Result<NativeEvent> {
        self.check_inited()?;

        Ok(self.events.get(id).map(|r| r.clone())?)
    }

    pub fn get_event_by_name(&self, name: &str) -> Result<NativeEvent> {
        self.check_inited()?;

        Ok(self.events.iter()
            .filter(|e| {
                e.name.eq(name)
            })
            .map(|e| {
                e.clone()
            })
            .nth(0)?)
    }

    fn start_looper(&mut self) -> Result<()> {
        if self.eloop.is_some() {
            return Ok(());
        }

        let (tx, rx) = oneshot::channel();
        let (stx, srx) = mpsc::channel(1);

        thread::spawn(move || {
            let mut core = Core::new().expect("Failed to create core");
            let remote = core.remote();

            tx.send(remote).unwrap();

            let _ = core.run(
                srx.into_future()
            ).unwrap();
        });
        self.eloop = Some(rx.wait()?);
        self.signal_map.insert(CORE_RUNNER_NAME.to_string(), stx);
        Ok(())
    }

    fn stop(tx: Sender<()>) -> Result<()> {
        let mut rwait = tx.wait();
        rwait.send(())?;
        rwait.flush()?;
        rwait.close()?;
        Ok(())
    }

    pub fn log_event_by_name<P>(&mut self, name: &str, params: Vec<String>, cb: Box<P>) -> Result<()>
        where P: INativeCallback + Send + 'static
    {
        self.check_inited()?;
        self.start_looper()?;

        if self.signal_map.contains_key(name) {
            return Err(ErrorKind::AlreadyLogging.into());
        }

        self.contract
            .as_ref()
            .and_then(|contract| {
                contract.events.get(name)
            })
            .ok_or(ErrorKind::InvalidEventName.into())
            .and_then(|event| {
                let indexed_param_types: Vec<&EventParam> = event.inputs.iter()
                    .filter(|p| p.indexed == true)
                    .collect();

                let mut raw_builder = RawTopicFilter::default();
                if indexed_param_types.len() > 0 && params.len() > 0 {
                    let param_0: &EventParam = indexed_param_types.get(0)?;
                    let t = check_type(&param_0.kind, &params.get(0)?)?;
                    if t.type_check(&param_0.kind) {
                        raw_builder.topic0 = Topic::This(t);
                    }

                    if indexed_param_types.len() > 1 && params.len() > 1 {
                        let param_1: &EventParam = indexed_param_types.get(1)?;
                        let t = check_type(&param_1.kind, &params.get(1)?)?;
                        if t.type_check(&param_1.kind) {
                            raw_builder.topic1 = Topic::This(t);
                        }

                        if indexed_param_types.len() > 2 && params.len() > 2 {
                            let param_2: &EventParam = indexed_param_types.get(2)?;
                            let t = check_type(&param_2.kind, &params.get(2)?)?;
                            if t.type_check(&param_2.kind) {
                                raw_builder.topic2 = Topic::This(t);
                            }
                        }
                    }
                }

                event.create_filter(raw_builder).map_err(Error::from)
            })
            .and_then(|topic_filter| {
                let host = self.host.clone();
                let addr = self.contract_address.clone();
                let n = name.to_string();
                let (tx, rx) = mpsc::channel(1);
                self.signal_map.insert(n.clone(), tx);
                self.eloop.as_ref()?.spawn(move |handle| {
                    let web3 = web3::Web3::new(web3::transports::Http::with_event_loop(&host, handle, 1).unwrap());

                    let filter = web3_filter(topic_filter, vec![addr]).build();
                    web3.eth_filter()
                        .create_logs_filter(filter)
                        .map_err(Error::from)
                        .and_then(|filter| {
                            FilterStreamOnce::new(filter, time::Duration::from_secs(2), rx)
                                .for_each(move |log| {
                                    cb.on_new_log(log);
                                    Ok(())
                                })
                        })
                        .map_err(|e| {
                            display_err(&e);
                            ()
                        })
                });
                Ok(())
            })
    }

    pub fn log_event_by_id<P>(&mut self, id: usize, params: Vec<String>, cb: Box<P>) -> Result<()>
        where P: INativeCallback + Send + 'static
    {
        self.check_inited()?;

        self.get_event_by_id(id)
            .and_then(|event| {
                self.log_event_by_name(&event.name, params, cb)
            })
    }

    pub fn stop_event_by_name(&mut self, name: &str) -> Result<()> {
        self.check_inited()?;

        Self::stop(self.signal_map.remove(name)?)
    }

    pub fn stop_event_by_id(&mut self, id: usize) -> Result<()> {
        self.check_inited()?;

        self.get_event_by_id(id)
            .and_then(|event| {
                self.stop_event_by_name(&event.name)
            })
    }

    // Block the current thread
    pub fn stop_all(&mut self) -> Result<()> {
        self.check_inited()?;

        if self.signal_map.drain()
            .map(|(_, v)| {
                Self::stop(v)
            })
            .all(|r: Result<()>| r.is_ok()) {
            Ok(())
        } else {
            Err(ErrorKind::StopAllError.into())
        }
    }

    pub fn get_logging_events(&self) -> Result<Vec<NativeEvent>> {
        self.check_inited()?;

        Ok(self.events.iter()
            .filter(|e| self.signal_map.contains_key(&e.name))
            .map(|e| e.clone())
            .collect())
    }
}

pub trait INativeCallback {
    fn on_new_log(&self, content: Log);
}

#[derive(Debug, Clone)]
pub struct NativeEvent {
    name: String,
    params: Vec<NativeParam>,
}

impl NativeEvent {
    pub fn get_name(&self) -> String {
        self.name.clone()
    }

    pub fn get_params(&self) -> Vec<NativeParam> {
        self.params.clone()
    }
}

#[derive(Debug, Clone)]
pub struct NativeParam {
    name: String,
    kind: String,
    indexed: bool,
}

impl NativeParam {
    pub fn get_name(&self) -> String {
        self.name.clone()
    }

    pub fn get_kind(&self) -> String {
        self.kind.clone()
    }

    pub fn is_indexed(&self) -> bool {
        self.indexed
    }
}
