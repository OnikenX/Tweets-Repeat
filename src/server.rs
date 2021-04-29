// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

mod common_twitter;
mod common_tls;
mod shared;

static CERTIFICATE: &str = "AC7ION_certificate.crt";
static PRIVATE_KEY: &str = "AC7ION_private.key";

use common_tls::acceptor_creation;
use shared::*;

#[macro_use]
extern crate fstrings;


use egg_mode::stream::{FilterLevel, StreamMessage};
use futures::TryStreamExt;
use async_std::{task};
use env_logger::{Builder, Env};
use futures::prelude::*;
use libp2p::gossipsub::MessageId;
use libp2p::gossipsub::{
    GossipsubEvent, GossipsubMessage, IdentTopic as Topic, MessageAuthenticity, ValidationMode,
};
use libp2p::{gossipsub, identity, PeerId};
use std::hash::{Hash, Hasher};
use std::time::Duration;
use std::{
    error::Error,
    task::{Context, Poll},
};
use std::collections::hash_map::DefaultHasher;
use tokio::fs;
use tokio::sync::{mpsc, oneshot};
use tokio::sync::mpsc::{Sender, Receiver};
use crate::shared::{LISTENING_ON_CLIENT, LISTENING_ON_SERVER};
use egg_mode::tweet::Tweet;
use egg_mode::user::TwitterUser;
use tokio::io::AsyncReadExt;
use tokio_native_tls::TlsAcceptor;
use std::path::{PathBuf, Path};
use tokio::net::{TcpListener, TcpSocket, TcpStream};
use std::sync::Arc;


/// Tokio Rustls server example
#[derive(FromArgs)]
struct Options {
    /// bind addr
    #[argh(optional, positional)]
    addr: String,

    /// cert file
    #[argh(option, short = 'c')]
    cert: PathBuf,

    /// key file
    #[argh(option, short = 'k')]
    key: PathBuf,
}


#[tokio::main]
async fn main() {
    let options: Options = argh::from_env();


    let (sx, rx) = mpsc::channel::<Tweet>(100);


    let files_saving = tokio::spawn(async move {
        save_tweets(rx).await
    });

    let networking_server_client = tokio::spawn(async move {
        send_messages_tls()
    });

    let receive_tweets = tokio::spawn(async move {
        receive_tweets(sx).await
    });

    let (f, s, t) = tokio::try_join!(files_saving, networking_server_client, receive_tweets);
}

// recives tweets from the twitter api
async fn receive_tweets(sx: Sender<Tweet>) -> Result<(), Box<dyn std::error::Error>> {
    loop {
        let config = common_twitter::Config::load().await;
        let stream = egg_mode::stream::filter()
            .track(&["#DOGECOIN"])
            .filter_level(FilterLevel::Medium)
// .language(&["en", "pt", "pt-pt", "pt-br", ])
            .start(&config.token)
            .try_for_each(|m| {
                if let StreamMessage::Tweet(tweet) = m {
                    println!("{:#?}", &tweet);
                    sx.send(tweet);
                } else {
                    println!("{:?}", m);
                }
                futures::future::ok(())
            });
        if let Err(e) = stream.await {
            println!("Stream error: {:?}", &e);
            println!("Reconnecting...")
        };
    };
}

// recives tweets from Receiver and loads them to disk
async fn save_tweets(rx: Receiver<Tweet>) -> Result<(), Box<dyn Error>> {
    //get netadata
    let mut path = std::env::current_dir()?;
    path.push(TWEETS_FOLDER);
    let metadata = tokio::fs::metadata(path).await?;
    //verify if metadata is valid, if not create
    if !metadata.is_dir() {
        fs::create_dir(TWEETS_FOLDER).await?;
    }

    // create tweets buffer
    let twitter_buffer = Arc::new(parking_lot::Mutex::new(Vec::new()));

    let (mut sx, mut rx) = mpsc::channel(64);

    let f
    let tweets_writter = tokio::spawn(async move {
        while let Some(tweet) = rx.recv().await {
            let tweet_json = serde_json::to_string(&TweetSerializable::from(tweet)).unwrap();

        }
    });
    tokio::join!(tweets_writter);
    Ok(())
}

type MenSender = (Sender<i32>, mpsc::Sender<String>, mpsc::Receiver<String>);

async fn send_messages_tls() -> Result<(), Box<dyn std::error::Error>> {
    // tls setup
    let certs = Path::new(CERTIFICATE);
    let private_key = Path::new(PRIVATE_KEY);
    let acceptor = acceptor_creation(&addr, certs, private_key)?;

    //start tcp connections
    let listener = TcpListener::bind(LISTENING_ON_SERVER).await?;
    //accept connections
    loop {
        let (stream, _) = listener.accept().await?;
        let acceptor = acceptor.clone();
        tokio::spawn(async move {
            send_messages_tls_response(stream, acceptor).await;
        });
    }
}

async fn send_mesages_tls_response(stream: TcpStream, acceptor: TlsAcceptor, sx: mpsc::Sender<MenSender>)
                                   -> Result<(), Box<dyn std::error::Error>> {
    let mut stream = acceptor.accept(stream).await?;
    let n_tweets = stream.read_i32().await?;
    std::thread::spawn(async move{

    });


    //reciver and sender of tweets
    let (mut sx_t, mut rx_t) = mpsc::channel::<String>(100);
    sx.send(MenSender { n_tweets, sx_t, rx_t }).await?;
    while let Some(tweet_json) = rx_t.recv().await {}
    stream.write();
}

async fn send_messages_tcp() {
    let tcp_stream = tokio::net::TcpListener::bind(LISTENING_ON_SERVER).await.unwrap();

    loop {
        let (connection, addr) = tcp_stream.accept().await.unwrap();
        tokio::spawn(async move {
            // connection.read()
        });
    }
}
