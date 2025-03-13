use std::{fmt::Display, pin::Pin};

use futures::{future::BoxFuture, FutureExt, Stream};

use crate::Result;

use super::publisher_types::Request;

pub trait Publisher<Message>
where
    Message: Send + 'static,
{
    fn get_name(&self) -> &'static str;

    fn publish_event(&self, message: Message) -> BoxFuture<Result<()>>;

    fn publish_request<Req, Rsp>(&self, request: Req) -> BoxFuture<Result<Rsp>>
    where
        Req: Display + Send + 'static,
        Rsp: Display + Send + 'static,
        Message: From<Request<Req, Rsp>>,
        Self: Sync,
    {
        async move {
            let (request, response) = Request::<Req, Rsp>::new(request);

            self.publish_event(request.into()).await?;

            let response = response.await?;
            Ok(response)
        }
        .boxed()
    }

    fn get_message_stream(
        &mut self,
        subscriber_name: &'static str,
    ) -> Result<Pin<Box<dyn Stream<Item = Message> + Send + Sync + 'static>>>;
}

pub trait PublisherLayer<InnerPublisherType, Message>
where
    InnerPublisherType: Publisher<Message>,
    Message: Send + 'static,
{
    type PublisherType: Publisher<Message>;
    fn layer(&self, publisher: InnerPublisherType) -> Self::PublisherType;
}
