use std::{
    error::Error,
};
use std::collections::VecDeque;
use std::hash::Hasher;
use std::ops::Add;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use argh::FromArgs;
use egg_mode::stream::{FilterLevel, StreamMessage};
use egg_mode::tweet::Tweet;
use egg_mode::user::TwitterUser;
use futures::future::err;
use futures::prelude::*;
use futures::TryStreamExt;
use tokio::fs;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpSocket, TcpStream};
use tokio::sync::{mpsc, oneshot};
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::task::JoinHandle;
use tokio::time::sleep;

// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
use common_tls::acceptor_creation;
use shared::*;

mod common_twitter;
mod common_tls;
mod shared;

static CERTIFICATE: &str = "AC7ION_certificate.crt";
static PRIVATE_KEY: &str = "AC7ION_private.key";
static TWEETS_BUFFER_LIMIT: usize = 100;

// /// Tokio Rustls server example
// #[derive(FromArgs)]
// struct CmdLineOptions {
//     /// bind addr
//     #[argh(option
//     )]
//     addr: String,
//
//     /// cert file
//     #[argh(option, short = 'c')]
//     cert: PathBuf,
//
//     /// key file
//     #[argh(option, short = 'k')]
//     key: PathBuf,
// }

/// tuple to send for a request to a tweet
type TweetRequest = (i32, mpsc::Sender<String>);

#[cfg(test)]
mod test {
    use std::error::Error;
    use std::io::{BufReader, Read};
    use std::ops::Add;
    use std::path::Path;

    use crate::{CERTIFICATE, PRIVATE_KEY, shared};
    use crate::common_tls::acceptor_creation;
    use crate::shared::TWEETS_FOLDER;

    #[test]
    fn test_certification() -> Result<(), Box<dyn Error>> {
        let addr = shared::LISTENING_ON_SERVER_ADDR.to_string().add(":").add(shared::LISTENING_ON_SERVER_PORT);
        let certs = Path::new(CERTIFICATE);
        dbg!(&certs);
        let private_key = Path::new(PRIVATE_KEY);
        dbg!(&private_key);
        let acceptor = acceptor_creation(&addr, certs, private_key)?;
        Ok(())
    }

    #[test]
    fn buffer_read_keys() {
        let value = &mut BufReader::new(std::fs::File::open(Path::new(PRIVATE_KEY)).unwrap());
        let mut string: String = String::new();
        value.read_to_string(&mut string);
        dbg!(string);
    }
}

#[tokio::main]
async fn main() {
    // let options: CmdLineOptions = argh::from_env();
    let (tweet_request_sender, tweet_request_receiver) = mpsc::channel::<TweetRequest>(10);
    let (tweet_sender, tweet_receiver) = mpsc::channel::<Tweet>(10);

    let tweets_manager_task = tokio::spawn(async move {
        match tweets_manager(tweet_receiver, tweet_request_receiver).await {
            Ok(_) => { eprintln!("Exited normally from tweets_manager."); }
            Err(e) => {
                eprintln!("Returned from tweets_manager, returned error {}", e);
            }
        }
    });

    let networking_server_client_task = tokio::spawn(async move {
        //the loading of the keys is giving a empty vector for some reason, fot that i will use
        //a normal tcp connection TODO(make the tls connection work)
        //match send_messages_tls(tweet_request_sender).await {
        match send_messages_tcp(tweet_request_sender).await {
            Ok(_) => { eprintln!("Exited normally from send_messages_tls"); }
            Err(e) => {
                eprintln!("Returned from send_messages_tls, returned error: {}", e);
            }
        }
    });

    let receive_tweets_task = tokio::spawn(async move {
        match receive_tweets(tweet_sender).await {
            Ok(_) => { eprintln!("Exited normally from receive_tweets"); }
            Err(e) => {
                eprintln!("Returned from receive_tweets, returned error: {}", e);
            }
        }
    });
    tokio::try_join!(tweets_manager_task, networking_server_client_task, receive_tweets_task);
}

/// recives tweets from the twitter api
async fn receive_tweets(tweet_sender: Sender<Tweet>) -> Result<(), Box<dyn Error>> {
    let config = common_twitter::Config::load().await;

    let stream = egg_mode::stream::filter()
        .track(&["#DOGECOIN"])
        .filter_level(FilterLevel::Low)
        // .language(&["en", "pt", "pt-pt", "pt-br", ])
        .start(&config.token)
        .try_for_each(|m| {
            if let StreamMessage::Tweet(tweet) = m {
                println!("tweet received from twitter...");
                tweet_sender.send(tweet.clone());
            } else {
                println!("Not an tweet: {:?}", &m);
            }
            futures::future::ok(())
        });
    if let Err(e) = stream.await {
        println!("Stream error: {:?}", &e);
        println!("Reconnecting...")
    };
    Ok(())
}

/// manages tweets in the server
async fn tweets_manager(mut tweet_receiver: Receiver<Tweet>, mut tweet_request_receiver: Receiver<TweetRequest>) -> Result<(), Box<dyn Error>> {
    //get netadata
    let mut use_disk_space = false;
    let mut path = std::env::current_dir()?;
    path.push(TWEETS_FOLDER);
    match tokio::fs::metadata(path).await {
        Ok(metadata) => {
            if !metadata.is_dir() {
                eprintln!("Is {} exists but is not a folder, ignoring disk tweets...", TWEETS_FOLDER);
            } else {
                use_disk_space = true;
            }
        }
        Err(e) => {
            eprintln!("Dir does not exists, creating it.");
            match fs::create_dir(TWEETS_FOLDER).await {
                Ok(_) => { use_disk_space = true; }
                Err(e) => { eprintln!("Could not create dir: {}", e); }
            };
        }
    }


    // this buffer saves tweets on memory
    let vec: VecDeque<String> = VecDeque::new();
    let tweets_buffer = Arc::new(parking_lot::Mutex::new(vec));

    let tweets_buffer_cloned = tweets_buffer.clone();
    println!("spawning respond_to_requests...");
    let respond_to_requests = tokio::spawn(async move {
        println!("Inited respond_to_requests....");
        let tweets_buffer = tweets_buffer_cloned;
        loop {
            match tweet_request_receiver.recv().await {
                None => {
                    eprintln!("Channel tweet_request_receiver is closed for some reason...");
                    break;
                }
                Some((mut n_tweets, mut sender)) => {
                    if n_tweets > TWEETS_BUFFER_LIMIT as i32 {
                        n_tweets = TWEETS_BUFFER_LIMIT as i32;
                    };
                    {
                        let guard_vec = tweets_buffer.lock();
                        if guard_vec.len() < n_tweets as usize {
                            n_tweets = guard_vec.len() as i32;
                        }

                        for (n, item) in guard_vec.iter().enumerate() {
                            sender.send(String::from(item.clone()));
                            if n + 1 >= n_tweets as usize {
                                break;
                            };
                        };
                    };
                }
            }
        };
    });

    let tweets_buffer_cloned = tweets_buffer.clone();
    // recives tweets from **receive_tweets()** saves them
    println!("spawning process_tweets...");
    let process_tweets = tokio::spawn(async move {
        println!("Inited process_tweets!");
        let tweets_buffer = tweets_buffer_cloned;
        loop {
            match tweet_receiver.recv().await {
                None => {
                    eprintln!("tweet_receiver is closed...");
                    break;
                }
                Some(tweet) => {
                    println!("Tweet received to processor : ");
                    let tweet_json = serde_json::to_string(&TweetSerializable::from(tweet)).unwrap();
                    {//protects the lock
                        let mut guard_vec = tweets_buffer.lock();
                        dbg!(&tweet_json);
                        guard_vec.push_back(tweet_json);
                        if guard_vec.len() > TWEETS_BUFFER_LIMIT {
                            guard_vec.pop_front();
                        }
                    }
                }
            }
            println!("Saiu do receiver");
        }
    });
    tokio::join!(process_tweets, respond_to_requests);
    Ok(())
}

/// tweets_manager helper: saves tweets to disk manages them
async fn manages_tweets_on_disk() {}

/// tweets_manager helper: sends the tweets from the server to the message sender to be received by the client
async fn send_tweets_as_messages() {}

/// Function responsible for talking with the client lib
async fn send_messages_tls(tweet_requester: mpsc::Sender<TweetRequest>) -> Result<(), Box<dyn Error>> {
    // tls setup
    let addr = shared::LISTENING_ON_SERVER_ADDR.to_string().add(":").add(shared::LISTENING_ON_SERVER_PORT);

    let certs = Path::new(CERTIFICATE);
    dbg!(&certs);
    let private_key = Path::new(PRIVATE_KEY);
    dbg!(&private_key);
    let acceptor = acceptor_creation(&addr, certs, private_key)?;

    //start tcp connections
    let listener = TcpListener::bind(&addr).await?;

    // accept connections
    loop {
        let (stream, _) = listener.accept().await?;
        let mut stream = acceptor.accept(stream).await?;
        let tweet_requester = tweet_requester.clone();
        tokio::spawn(async move {
            send_messages_tls_response(stream, tweet_requester).await;
        });
    }
}

async fn send_messages_tcp(tweet_requester: mpsc::Sender<TweetRequest>) -> Result<(), Box<dyn Error>> {
    let tcp_stream = TcpListener::bind(LISTENING_ON_SERVER_ADDR.to_string().add(":").add(LISTENING_ON_SERVER_PORT)).await.unwrap();

    loop {
        let (connection, addr) = tcp_stream.accept().await?;
        let tweet_requester = tweet_requester.clone();
        tokio::spawn(async move {});
    }
}

async fn send_messages_tcp_response(mut stream: TcpStream, tweet_requester: mpsc::Sender<TweetRequest>)
                                    -> Result<(), Box<dyn std::error::Error>> {
    //read how many tweets the client wants
    let n_tweets = stream.read_i32().await?;

    //create a channel to receive tweets and ask for them
    let (mut sx_t, mut rx_t) = mpsc::channel::<String>(100);
    tweet_requester.send((n_tweets, sx_t)).await?;

    //receives tweets and sends them to the client
    while let Some(tweet_json) = rx_t.recv().await {
        stream.write(tweet_json.as_bytes());
    }
    Ok(())
}

/// assistent function for _send_mesages_tls_ to responde to messages
async fn send_messages_tls_response(mut stream: tokio_rustls::server::TlsStream<TcpStream>, tweet_requester: mpsc::Sender<TweetRequest>)
                                    -> Result<(), Box<dyn std::error::Error>> {
    //read how many tweets the client wants
    let n_tweets = stream.read_i32().await?;

    //create a channel to receive tweets and ask for them
    let (mut sx_t, mut rx_t) = mpsc::channel::<String>(100);
    tweet_requester.send((n_tweets, sx_t)).await?;

    //receives tweets and sends them to the client
    while let Some(tweet_json) = rx_t.recv().await {
        stream.write(tweet_json.as_bytes());
    }
    let _ = stream.flush().await;
    Ok(())
}



