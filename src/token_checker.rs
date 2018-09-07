use ethabi::Token;
use rustc_hex::FromHex;
use errors::*;
use ethereum_types::{Address, U256};
use std::str::FromStr;
use ethabi::ParamType;


pub fn remove_0x(input: &str) -> &str {
    if input.starts_with("0x") {
        &input[2..]
    } else {
        input
    }
}

pub fn check_type(param_type: &ParamType, value: &str) -> Result<Token> {
    match *param_type {
        ParamType::Address => {
            let v = Address::from_str(remove_0x(value))?;
            Ok(Token::Address(v))
        }
        ParamType::Bytes => {
            let v = remove_0x(value).from_hex()?;
            Ok(Token::Bytes(v))
        }
        ParamType::Int(_) => {
            let v = U256::from_str(remove_0x(value))?;
            Ok(Token::Int(v))
        }
        ParamType::Uint(_) => {
            let v = U256::from_str(remove_0x(value))?;
            Ok(Token::Uint(v))
        }
        ParamType::Bool => {
            let v = bool::from_str(value)?;
            Ok(Token::Bool(v))
        }
        ParamType::String => {
            Ok(Token::String(value.to_string()))
        }
        ParamType::FixedBytes(_) => {
            let v = remove_0x(value).from_hex()?;
            let size = v.len();
            if size == 1 ||
                size == 2 ||
                size == 3 ||
                size == 4 ||
                size == 5 ||
                size == 8 ||
                size == 16 ||
                size == 32 ||
                size == 64 ||
                size == 128 ||
                size == 256 ||
                size == 512 ||
                size == 1024
                {
                    Ok(Token::FixedBytes(v.to_vec()))
                } else {
                Err(Error::from("Invalid sized bytes"))
            }
        }
        _ => Err(Error::from("Invalid"))
    }
}

// TODO: Suppurt Array and FixedArray
// value is in hex format
pub fn check(param_type_str: &str, value: &str) -> Result<Token> {
    match param_type_str {
        "address" => {
            let v = Address::from_str(remove_0x(value))?;
            Ok(Token::Address(v))
        }
        "bytes" => {
            let v = remove_0x(value).from_hex()?;
            Ok(Token::Bytes(v))
        }
        "int" => {
            let v = U256::from_str(remove_0x(value))?;
            Ok(Token::Int(v))
        }
        "uint" => {
            let v = U256::from_str(remove_0x(value))?;
            Ok(Token::Uint(v))
        }
        "bool" => {
            let v = bool::from_str(value)?;
            Ok(Token::Bool(v))
        }
        "string" => {
            Ok(Token::String(value.to_string()))
        }
        "fixedbytes" => {
            let v = remove_0x(value).from_hex()?;
            let size = v.len();
            if size == 1 ||
                size == 2 ||
                size == 3 ||
                size == 4 ||
                size == 5 ||
                size == 8 ||
                size == 16 ||
                size == 32 ||
                size == 64 ||
                size == 128 ||
                size == 256 ||
                size == 512 ||
                size == 1024
                {
                    Ok(Token::FixedBytes(v.to_vec()))
                } else {
                Err(Error::from("Invalid sized bytes"))
            }
        }
        _ => Err(Error::from("Invalid"))
    }
}
