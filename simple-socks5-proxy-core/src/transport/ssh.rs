use anyhow::{bail, Result};
use async_trait::async_trait;
use russh::{client, keys};
use std::{path::PathBuf, sync::Arc};

#[derive(Debug)]
pub struct SshSessionConfig {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: Option<String>,
    pub secret_key_path: Option<PathBuf>,
    pub secret_key_password: Option<String>,

    pub russh_config: russh::client::Config,
}

impl Default for SshSessionConfig {
    fn default() -> Self {
        Self {
            host: Default::default(),
            port: 22,
            username: "root".into(),
            password: Default::default(),
            secret_key_path: Default::default(),
            secret_key_password: Default::default(),
            russh_config: Default::default(),
        }
    }
}

#[derive(Clone)]
pub struct SshSession {
    client: Arc<client::Handle<SshEventHandler>>,
}

pub struct SshEventHandler {}

#[async_trait]
impl client::Handler for SshEventHandler {
    type Error = russh::Error;

    async fn check_server_key(
        &mut self,
        server_public_key: &keys::key::PublicKey,
    ) -> Result<bool, Self::Error> {
        // TODO
        println!(
            "You are connecting to a server with the following public key: {:?}",
            server_public_key.fingerprint()
        );
        Ok(true)
    }
}

impl SshSession {
    pub async fn new(config: SshSessionConfig) -> Result<Self> {
        let russh_config = Arc::new(config.russh_config);
        let client_handler = SshEventHandler {};

        let mut ssh_client =
            client::connect(russh_config, (config.host, config.port), client_handler).await?;

        let res = if let Some(key_path) = config.secret_key_path {
            let key_pair = keys::load_secret_key(&key_path, config.secret_key_password.as_deref())?;
            ssh_client
                .authenticate_publickey(config.username, Arc::new(key_pair))
                .await?
        } else {
            let Some(password) = config.password else {
                bail!("Password is required for password-based authentication");
            };
            ssh_client
                .authenticate_password(config.username, password)
                .await?
        };
        if !res {
            bail!("Authentication failed");
        }

        Ok(Self {
            client: Arc::new(ssh_client),
        })
    }
}

#[async_trait]
impl super::Session for SshSession {
    type Error = russh::Error;
    type Stream = russh::ChannelStream<client::Msg>;

    fn is_closed(&self) -> bool {
        self.client.is_closed()
    }

    async fn create_stream(
        &mut self,
        host_to_connect: String,
        port_to_connect: u16,
        originator_address: String,
        originator_port: u16,
    ) -> std::result::Result<Self::Stream, Self::Error> {
        let channel = self
            .client
            .channel_open_direct_tcpip(
                host_to_connect,
                port_to_connect as u32,
                originator_address,
                originator_port as u32,
            )
            .await?;
        log::debug!("Channel opened: {:?}", channel.id());
        let stream = channel.into_stream();
        Ok(stream)
    }
}
