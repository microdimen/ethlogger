use web3::Transport;
use web3::api::BaseFilter;
use std::time::Duration;
use web3::helpers::CallResult;
use serde::de::DeserializeOwned;
use futures::Stream;
use errors::*;
use futures::Poll;
use tokio_timer::{Interval, Timer};
use futures::Future;
use alloc::vec;
use futures::sync::mpsc::Receiver;
use futures::Async;

#[derive(Debug)]
pub struct FilterStreamOnce<T: Transport, I> {
    base: BaseFilter<T, I>,
    interval: Interval,
    state: FilterStreamState<I, T::Out>,
    rx: Receiver<()>,
}

impl<T: Transport, I> FilterStreamOnce<T, I> {
    pub fn new(base: BaseFilter<T, I>, poll_interval: Duration, rx: Receiver<()>) -> Self {
        FilterStreamOnce {
            base,
            interval: Timer::default().interval(poll_interval),
            state: FilterStreamState::WaitForInterval,
            rx,
        }
    }

    pub fn transport(&self) -> &T {
        self.base.transport()
    }
}

impl<T: Transport, I> Drop for FilterStreamOnce<T, I> {
    fn drop(&mut self) {
        debug!("Dropping FilterStream");
    }
}

#[derive(Debug)]
enum FilterStreamState<I, O> {
    WaitForInterval,
    GetFilterChanges(CallResult<Option<Vec<I>>, O>),
    NextItem(vec::IntoIter<I>),
}

impl<T: Transport, I: DeserializeOwned> Stream for FilterStreamOnce<T, I> {
    type Item = I;
    type Error = Error;

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        if let Ok(Async::Ready(_)) = self.rx.poll() {
            debug!("ready to finish filter stream");
            return Ok(None.into());
        }
        loop {
            let next_state = match self.state {
                FilterStreamState::WaitForInterval => {
                    let _ready = try_ready!(
                        self.interval
                            .poll()
                            .map_err(|_| Error::from("wait for interval error"))
                    );
                    let future = self.base.poll();

                    FilterStreamState::GetFilterChanges(future)
                }
                FilterStreamState::GetFilterChanges(ref mut future) => {
                    let items = try_ready!(future.poll()).unwrap_or_default();
                    FilterStreamState::NextItem(items.into_iter())
                }
                FilterStreamState::NextItem(ref mut iter) => match iter.next() {
                    Some(item) => {
                        return Ok(Some(item).into());
                    }
                    None => FilterStreamState::WaitForInterval,
                },
            };
            self.state = next_state;
        }
    }
}