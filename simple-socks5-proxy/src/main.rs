use std::path::PathBuf;

use clap::Parser;
use simple_socks5_proxy_core::*;

#[tokio::main]
async fn main() {
    env_logger::init();
    let cli = Cli::parse();

    let mut config = ssh::SshSessionConfig::default();
    let parts = cli.host.split('@').collect::<Vec<&str>>();
    if parts.len() == 2 {
        config.username = parts[0].to_string();
        config.host = parts[1].to_string();
    } else {
        eprintln!("Invalid host format: {}", cli.host);
        std::process::exit(1);
    }

    config.port = cli.port;
    config.secret_key_path = cli.private_key;
    println!("Connecting to SSH server: {}", config.host);

    let session = ssh::SshSession::new(config).await.unwrap();
    println!("Created SSH session successfully");

    let listen_addr = cli.local_socks5_addr.parse().unwrap();
    println!("Starting socks5 server on: {:?}", listen_addr);
    socks5::start_socks5_server(listen_addr, session)
        .await
        .unwrap();
}

#[derive(clap::Parser)]
pub struct Cli {
    #[clap(index = 1)]
    host: String,

    #[clap(long, short, default_value_t = 22)]
    port: u16,

    #[clap(long, short = 'k')]
    private_key: Option<PathBuf>,

    #[clap(long, short = 'l', required = true)]
    local_socks5_addr: String,
}
