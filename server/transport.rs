use async_trait::async_trait;

#[async_trait]
pub trait Transport {
    type Request;
    type Response;
    type Error;

    async fn send(resp: Self::Response) -> Result<(), Self::Error>;
    async fn receive() -> Result<Self::Request, Self::Error>;
    async fn shutdown() -> Result<(),Self::Error>;
} 

