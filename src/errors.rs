use rustc_hex;
use serde_json;
use std;
use ethabi;
use futures;
use std::option::NoneError;
use web3;
#[cfg(feature = "backtrace")]
use error_chain::ChainedError;

error_chain! {
    foreign_links {
        ParseBookError(std::str::ParseBoolError);
        Io(std::io::Error);
        Utf8Error(std::str::Utf8Error);
        FormatError(std::fmt::Error);
        FromHexError(rustc_hex::FromHexError);
        FutureCanceled(futures::Canceled);
        MpscSendError(futures::sync::mpsc::SendError<()>);
        SerdError(serde_json::Error);
    }
    links {
        Eth(ethabi::Error, ethabi::ErrorKind);
        Web3(web3::Error,web3::ErrorKind );
    }
    errors {
         NotInited {
             description("Please init NativeContract before use")
             display("Not Inited")
        }
        InvalidEventName{
             description("Invalid Event Name")
             display("Invalid Event Name")
        }
        NoneToError {
              description("None Error")
             display("None Error")
        }
        PollError{
              description("Poll Error")
             display("Poll Error")
        }
        SendError{
              description("Send Error")
             display("Send Error")
        }
        StopAllError{
              description("Stop All Error")
             display("Stop All Error")
        }
        AlreadyLogging{
              description("Already Logging")
             display("Already logging")
        }
    }
}
impl From<NoneError> for Error {
    fn from(_: NoneError) -> Self {
        ErrorKind::NoneToError.into()
    }
}

#[cfg(feature = "backtrace")]
pub fn display_err(e: &Error) {
    error!("{}", e.display_chain().to_string());
}

#[cfg(not(feature = "backtrace"))]
pub fn display_err(e: &Error) {
    error!("{:?}", e);
}
