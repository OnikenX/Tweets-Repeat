use std::{error, io, thread};
use std::error::Error;
use std::ffi::{CString, NulError};
use std::fs::{File, read_to_string};
use std::ops::Add;
use std::ptr::null;

use libc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

use shared::*;

mod shared;


#[cfg(test)]
mod tests {
    use crate::*;

    #[test]
    fn connection_test() {
        let con = tokio::runtime::Runtime::new().unwrap().block_on(async { connection_tcp(5).await });
        con.unwrap();
    }
}
// general shared functions

#[no_mangle]
/// Requests tweets from server
///
/// This is a C like binding
pub extern "cdecl" fn get_tweets(n_tweets: libc::c_double) -> *const libc::c_char {
    match get_tweets_helper(n_tweets as i32) {
        Ok(json) => { json }
        Err(_) => { null() }
    }
}

/// does the same as get_tweets but with the tweak that is wrapped
pub extern "cdecl" fn get_tweets_wrapped(n_tweets: libc::c_double) -> *const libc::c_char {
    match get_tweets_wrapped_helper(n_tweets as i32) {
        Ok(json) => { json }
        Err(_) => {null()}
    }
}



fn get_tweets_helper(n_tweets: i32) -> Result<*const libc::c_char, Box<dyn Error>> {
    Ok(CString::new(tokio::runtime::Runtime::new()?.block_on(async move { connection_tcp(n_tweets).await })?)?.into_raw())
}

fn get_tweets_wrapped_helper(n_tweets: i32) -> Result<*const libc::c_char, Box<dyn Error>> {
    let json = tokio::runtime::Runtime::new()?.block_on(async move { connection_tcp(n_tweets).await })?;
    let mut wrapped_json = String::from("{\"tweets\":");
    wrapped_json += json.as_ref();
    wrapped_json += "}";
    Ok(CString::new(json)?.into_raw())
}


async fn connection_tcp_unwrapped(n_tweets: i32) -> Result<String, Box<dyn Error>> {
    let mut stream = tokio::net::TcpStream::connect(SERVER_ADDR.to_string().add(":").add(LISTENING_ON_SERVER_PORT)).await.unwrap();
    let _ = stream.write_i32(n_tweets).await.unwrap();
    let mut json_tweets = String::new();
    let _ = stream.read_to_string(&mut json_tweets).await.unwrap();
    dbg!(&json_tweets);
    let json: Vec<TweetSerializable> = serde_json::from_str(&json_tweets[..]).unwrap();
    dbg!(&json);
    Ok(json_tweets)
}



async fn connection_tcp(n_tweets: i32) -> Result<String, Box<dyn Error>> {
    let mut stream = tokio::net::TcpStream::connect(SERVER_ADDR.to_string().add(":").add(LISTENING_ON_SERVER_PORT)).await?;
    let _ = stream.write_i32(n_tweets).await?;
    let mut json_tweets = String::new();
    let _ = stream.read_to_string(&mut json_tweets).await?;
    Ok(json_tweets)
}
