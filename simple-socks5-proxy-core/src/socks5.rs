use anyhow::Result;
use socks5_impl::{
    protocol::{Address, Reply},
    server::{auth, ClientConnection, IncomingConnection, Server},
};
use std::{net::SocketAddr, sync::Arc};

use crate::transport::Session;

pub async fn start_socks5_server<S: Session>(listen_addr: SocketAddr, session: S) -> Result<()> {
    let server = Server::bind(listen_addr, Arc::new(auth::NoAuth)).await?;

    while let Ok((conn, _)) = server.accept().await {
        let session_cl = session.clone();
        log::debug!("New connection from: {}", conn.peer_addr()?);
        tokio::spawn(async move {
            if let Err(err) = handle_stream(conn, session_cl.clone()).await {
                log::error!("{err}");
            }
        });
    }
    Ok(())
}

async fn handle_stream<S: Session>(conn: IncomingConnection<()>, mut session: S) -> Result<()> {
    let (conn, _) = conn.authenticate().await?;

    match conn.wait_request().await? {
        ClientConnection::UdpAssociate(associate, _) => {
            let mut conn = associate
                .reply(Reply::CommandNotSupported, Address::unspecified())
                .await?;
            conn.shutdown().await?;
        }
        ClientConnection::Bind(bind, _) => {
            let mut conn = bind
                .reply(Reply::CommandNotSupported, Address::unspecified())
                .await?;
            conn.shutdown().await?;
        }
        ClientConnection::Connect(connect, addr) => {
            let target = match addr.clone() {
                Address::DomainAddress(domain, port) => {
                    session
                        .create_stream(domain, port, "localhost".into(), 0)
                        .await
                }
                Address::SocketAddress(addr) => {
                    session
                        .create_stream(addr.ip().to_string(), addr.port(), "localhost".into(), 0)
                        .await
                }
            };

            if let Ok(mut target) = target {
                let mut conn = connect.reply(Reply::Succeeded, addr).await?;
                log::debug!("Forward from: {}", conn.peer_addr()?);
                tokio::io::copy_bidirectional(&mut target, &mut conn).await?;
            } else {
                let mut conn = connect
                    .reply(Reply::HostUnreachable, Address::unspecified())
                    .await?;
                conn.shutdown().await?;
            }
        }
    }
    Ok(())
}
