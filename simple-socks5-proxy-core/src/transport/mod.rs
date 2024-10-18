use tokio::io::{AsyncRead, AsyncWrite};

pub mod ssh;

pub trait Session {
    type Error;

    fn is_closed(&self) -> bool;

    async fn create_stream<A: Into<String>, B: Into<String>>(
        &mut self,
        host_to_connect: A,
        port_to_connect: u16,
        originator_address: B,
        originator_port: u16,
    ) -> Result<impl AsyncRead + AsyncWrite, Self::Error>;
}
