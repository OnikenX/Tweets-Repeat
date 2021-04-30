use argh::FromArgs;
use std::fs::File;
use std::io::{self, BufReader};
use std::net::ToSocketAddrs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::io::{copy, sink, split, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio_rustls::rustls::internal::pemfile::{certs, rsa_private_keys};
use tokio_rustls::rustls::{Certificate, NoClientAuth, PrivateKey, ServerConfig};
use tokio_rustls::TlsAcceptor;
use std::error::Error;


pub fn acceptor_creation(addr : &String, cert : &Path, key : &Path, ) -> Result<TlsAcceptor, Box<dyn Error>>{
    //setup dos argumentos recebidos
    let addr = addr
        .to_socket_addrs()?
        .next()
        .ok_or_else(|| io::Error::from(io::ErrorKind::AddrNotAvailable))?;
    let certs = load_certs(&cert)?;
    let mut keys = load_keys(&key)?;
    assert!(keys.len() > 0);

    //setup da configuração tls
    let mut config = ServerConfig::new(NoClientAuth::new());

    config.set_single_cert(certs, keys.pop().ok_or("None")?)
        .map_err(|err| io::Error::new(io::ErrorKind::InvalidInput, err))?;

    Ok(TlsAcceptor::from(Arc::new(config)))
}

fn load_certs(path: &Path) -> io::Result<Vec<Certificate>> {
    certs(&mut BufReader::new(File::open(path)?))
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "invalid cert"))
}

fn load_keys(path: &Path) -> io::Result<Vec<PrivateKey>> {
    let  rd = &mut BufReader::new(File::open(path)?);
    rsa_private_keys(rd)
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "invalid key"))
}

