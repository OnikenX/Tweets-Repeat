#[cfg(test)]
mod shared;

mod tests {
    use crate::*;

    #[test]
    fn works() {
        println!("test done");
    }

    #[test]
    fn test_enum() {
        assert_eq!(ReturnValues::Sucess as i8, 1i8);
    }

    #[test]
    fn test_connect() {
        let initer = std::thread::spawn(move || {
            if init_lib() != (ReturnValues::Sucess as i8) {
                eprintln!("Init lib did not succeed!");
                panic!();
            };
        });
        loop {
            eprintln!("inited loop");
            let raw_str = gettweet();
            eprint!("Testing raw string: ");
            assert!(!raw_str.is_null());
            eprint!("passed!\n");
            let c_str: &CStr = unsafe { CStr::from_ptr(raw_str) };
            let str_slice: &str = c_str.to_str().unwrap();
            println!("String received: {}", str_slice);
        };
        initer.join();
    }
}

// src/lib.rs
use libc;
use std::ffi::{CStr, CString, NulError};
use static_init::{dynamic};
use std::sync::{Mutex, Arc, PoisonError, MutexGuard};
use std::sync::mpsc::{channel, RecvError};

use tokio::net::{TcpListener, TcpStream};
use std::{io, thread, error};
use async_std::{task};
use egg_mode::tweet::Tweet;
use std::thread::{sleep, JoinHandle};
use egg_mode::stream::{FilterLevel, StreamMessage};
use futures::prelude::*;
use libp2p::gossipsub::MessageId;
use libp2p::gossipsub::{
    GossipsubEvent, GossipsubMessage, IdentTopic as Topic, MessageAuthenticity, ValidationMode,
};
use libp2p::{gossipsub, identity, PeerId};
use std::hash::{Hash, Hasher};
use std::time::Duration;
use std::{
    task::{Context, Poll},
};
use std::collections::hash_map::DefaultHasher;
use once_cell::sync::OnceCell;
use std::sync::mpsc::Sender;
use std::sync::mpsc::Receiver;
use std::ptr::null;
use std::error::Error;
use std::cell::Cell;
use tokio_rustls::{TlsConnector, TlsStream};
use std::io::BufReader;
use std::fs::{File, read_to_string};
use std::path::Path;
use tokio_rustls::webpki::DNSNameRef;
use tokio::io::{AsyncWriteExt, AsyncReadExt};

// ######### ERRORS STRUCT ########

#[repr(i8)]
#[derive(Debug, PartialEq)]
enum ReturnValues {
    Sucess = 1i8,
    ErrorOnceCellUsed = -1i8,
    ErrorInitLibUsed = -2i8,
    ErrorThreadCreation = -3i8,
    ErrorOther = -4i8,
}

// ########## shared  variables #######


static RECEIVER: OnceCell<Mutex<Receiver<String>>> = OnceCell::new();

// ########### shared functions #############


// general shared functions

#[no_mangle]
/// Requests tweets from server
///
/// This is a C like binding
pub extern "cdecl" fn get_tweets(n_tweets: libc::c_double) -> *const libc::c_char {
    get_tweets_assist(n_tweets as i32)
}

fn convert_string_to_c_char(source: String) -> Result<*const libc::c_char, Box<dyn Error>> {
    Ok(CString::new(RECEIVER.get().ok_or("None")?.lock()?.recv()?)?.into_raw())
}

fn convert_c_char_to_string(source: *const libc::c_char) -> String {
    let c_str: &CStr = unsafe { CStr::from_ptr(string_c) };
    //cloning the string just to be safe
    String::from(c_str.to_str()?.clone())
}

///
fn get_tweets_assist(n_tweets: i32) -> *const libc::c_char {
    init_connection(n_tweets);
    convert_string_to_c_char("".to_string()).unwrap_or_else(null())
}

// static configcell : OnceCell<Cell<tokio_rustls::rustls::ClientConfig>> = OnceCell::new();

fn init_connection(n_tweets : i32) -> Vec<String>{

    //address setting
    let addr = (shared::SERVER_ADDR, shared::LISTENING_ON_SERVER_PORT)
        .to_socket_addrs()?
        .next()
        .ok_or_else(|| io::Error::from(io::ErrorKind::NotFound))?;


    //configurations
    let mut config = tokio_rustls::rustls::ClientConfig::new();
    config
        .root_store
        .add_server_trust_anchors(&webpki_roots::TLS_SERVER_ROOTS);
    let mut pem = BufReader::new(File::open(Path::new("AC7ION_certificate.crt"))?);

    config
        .root_store
        .add_pem_file(&mut pem)
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "invalid cert"))?;

    let connector = TlsConnector::from(Arc::new(config));
    let stream = TcpStream::connect(&addr).await?;

    let domain = DNSNameRef::try_from_ascii_str(&domain)
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "invalid dnsname"))?;


    let mut stream = connector.connect(domain, stream).await?;

    stream.write_i32(n_tweets);
    stream.flush();

    let tweets: Vec<String> = vec![];
    let temp = String::new();

    tweets


}
