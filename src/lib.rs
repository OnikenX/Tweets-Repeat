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

use tokio::net::TcpListener;
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

// GAMEMAKER
#[no_mangle]
/// init_lib for gamemaker
///
/// This is a C like binding
pub extern "cdecl" fn init_lib_gamemaker() -> libc::c_double {
    f64::from(init_lib())
}

#[no_mangle]
/// Gets an **double** and returns the double of that **double**
///
/// This is a C like binding
pub extern "cdecl" fn foo(value: libc::c_double) -> libc::c_double {
    2.0 * value
}

#[no_mangle]
/// Gets a **string** and returns an *STRING **string** RECEIVED*
///
/// This is a C like binding
pub extern "cdecl" fn bar(string_c: *const libc::c_char) -> *const libc::c_char {
    match bar_assist(string_c) {
        Ok(string) => { string }
        Err(_) => { null() }
    }
}

// general shared functions

#[no_mangle]
/// This function needs to be called to initialize the functionality of the lib
///
/// This is a C like binding
pub extern "cdecl" fn init_lib() -> libc::c_char {
    if let None = RECEIVER.get() {
        let (sx, rx) = channel();
        RECEIVER.set(Mutex::new(rx));
        match futures::executor::block_on(receive_messages_gossip(sx)) {
            Ok(_) => { ReturnValues::Sucess as i8 }
            Err(_) => { ReturnValues::ErrorOther as i8 }
        }
    } else {
        eprintln!("Server can't be inited twice");
        ReturnValues::ErrorOnceCellUsed as i8
    }
}

#[no_mangle]
/// This is a C like binding
pub extern "cdecl" fn gettweet() -> *const libc::c_char {
    gettweet_asst_unwrapped()
    // match gettweet_asst(){
    //     Ok(ok) => {ok}
    //     Err(_) => {null()}
    // }
}


fn gettweet_asst_unwrapped() -> *const i8 {
    if let None = RECEIVER.get() {
        null() as *const i8
    } else {
        CString::new(RECEIVER.get().ok_or("RECEIVER returns None").unwrap().lock().unwrap().recv().unwrap()).unwrap().into_raw()
    }
}

fn gettweet_asst() -> Result<*mut i8, Box<dyn Error>> {
    Ok(CString::new(RECEIVER.get().ok_or("None")?.lock()?.recv()?)?.into_raw())
}

// ######## RUST FUNCTIONS / helper ##################

///  run this call to init the variables in the library, it will start a thread
///
/// # Returns
///
/// returns a value from the ReturnValues
///
/// Recives messages from a gossip instance of libp2p
async fn receive_messages_gossip(sx: Sender<String>) -> Result<(), Box<dyn std::error::Error>> {
    // Create a random PeerId
    let local_key = identity::Keypair::generate_ed25519();
    let local_peer_id = PeerId::from(local_key.public());
    println!("Local peer id: {:?}", local_peer_id);

    // Set up an encrypted TCP Transport over the Mplex and Yamux protocols
    let transport = libp2p::development_transport(local_key.clone()).await?;

    // Create a Gossipsub topic
    let topic = Topic::new("test-net");

    // Create a Swarm to manage peers and events
    let mut swarm = {
        // To content-address message, we can take the hash of message and use it as an ID.
        let message_id_fn = |message: &GossipsubMessage| {
            let mut s = DefaultHasher::new();
            message.data.hash(&mut s);
            MessageId::from(s.finish().to_string())
        };

        // Set a custom gossipsub
        let gossipsub_config = gossipsub::GossipsubConfigBuilder::default()
            .heartbeat_interval(Duration::from_secs(10)) // This is set to aid debugging by not cluttering the log space
            .validation_mode(ValidationMode::Strict) // This sets the kind of message validation. The default is Strict (enforce message signing)
            .message_id_fn(message_id_fn) // content-address messages. No two messages of the
            // same content will be propagated.
            .build()?;
        // build a gossipsub network behaviour
        let mut gossipsub: gossipsub::Gossipsub =
            gossipsub::Gossipsub::new(MessageAuthenticity::Signed(local_key), gossipsub_config)?;

        // subscribes to our topic
        match gossipsub.subscribe(&topic) {
            Ok(_) => {}
            Err(_) => { return Err(Box::from("gossip subscription error")); }
        }

        // add an explicit peer if one was provided
        if let Some(explicit) = std::env::args().nth(2) {
            let explicit = explicit.clone();
            match explicit.parse() {
                Ok(id) => gossipsub.add_explicit_peer(&id),
                Err(err) => println!("Failed to parse explicit peer id: {:?}", err),
            }
        }

        // build the swarm
        libp2p::Swarm::new(transport, gossipsub, local_peer_id)
    };

    // Listen on all interfaces and whatever port the OS assigns
    swarm.listen_on("/ip6/::/tcp/0".parse()?)?;

    // Reach out to another node if specified
    // if let Some(to_dial) = std::env::args().nth(1) {
    //     let dialing = to_dial.clone();
    //     match to_dial.parse() {
    //         Ok(to_dial) => match swarm.dial_addr(to_dial) {
    //             Ok(_) => println!("Dialed {:?}", dialing),
    //             Err(e) => println!("Dial {:?} failed: {:?}", dialing, e),
    //         },
    //         Err(err) => println!("Failed to parse address to dial: {:?}", err),
    //     }
    // }
    swarm.dial_addr(shared::SERVER_MULTIADDR.parse().unwrap()).unwrap();

    // Kick it off
    // let mut listening = false;
    task::block_on(future::poll_fn(move |cx: &mut Context<'_>| {
        loop {
            match swarm.poll_next_unpin(cx) {
                Poll::Ready(Some(gossip_event)) => match gossip_event {
                    GossipsubEvent::Message {
                        propagation_source: peer_id,
                        message_id: id,
                        message,
                    } => println!(
                        "Got message: {} with id: {} from peer: {:?}",
                        String::from_utf8_lossy(&message.data),
                        id,
                        peer_id
                    ),
                    _ => {}
                },
                Poll::Ready(None) | Poll::Pending => break,
            }
        }
        // if !listening {
        //     for addr in libp2p::Swarm::listeners(&swarm) {
        //         println!("Listening on {:?}", addr);
        //         listening = true;
        //     }
        // }
        Poll::Pending
    }))
}


fn bar_assist(string_c: *const libc::c_char) -> Result<*const libc::c_char, Box<dyn std::error::Error>> {
    let c_str: &CStr = unsafe { CStr::from_ptr(string_c) };
    let str_slice: &str = c_str.to_str()?;
    let return_string = format!("STRING \"{}\" RECEIVED", str_slice);
    println!("{}", &return_string);
    let c_str = CString::new(return_string)?;
    Ok(c_str.into_raw())
}

// #[no_mangle]
//  run this call to init the variables in the library, it will start a thread
//
// # Returns
//
// returns a value from the ReturnValues
// pub extern "cdecl" fn init_lib() -> libc::c_char {
//     static joinThread;
//     if !IS_THREAD_INICIALIZED.as_ref() {
//         if let None = THREAD_HANDLE.get() {
//             let (sx, rx) = channel::<String>();
//             match thread::Builder::new().spawn(move || receive_messages_gossip(sx)) {
//                 Ok(handle) => {
//                     joinThread = handle;
//                     if !(matches!(THREAD_HANDLE.set(handle), Ok(())) && matches!(RECEIVER.set(rx), Ok(()))) {
//                         return ReturnValues::ErrorOnceCellUsed as i8;
//                     }
//                 }
//                 Err(e) => {
//                     eprintln!("Error creating thread : {:?}", e);
//                     return ReturnValues::ErrorThreadCreation as i8;
//                 }
//             }
//         } else {
//             return ReturnValues::ErrorOnceCellUsed as i8;
//         }
//     } else {
//         return ReturnValues::ErrorInitLibUsed as i8;
//     }
//     ReturnValues::Sucess as i8
// }
