use async_trait::async_trait;
use tokio::io::{AsyncRead, AsyncWrite};

pub mod ssh;

#[async_trait]
pub trait Session: Clone + Send + 'static{
    type Error: Send;
    type Stream: AsyncRead + AsyncWrite + Unpin + Send;

    fn is_closed(&self) -> bool;

    async fn create_stream(
        &mut self,
        host_to_connect: String,
        port_to_connect: u16,
        originator_address: String,
        originator_port: u16,
    ) -> Result<Self::Stream, Self::Error>;
}
