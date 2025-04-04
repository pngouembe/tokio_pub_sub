use std::{fmt::Display, pin::Pin};

use async_pub_sub::{Publisher, PublisherImpl, PublisherLayer, Result, SubscriberImpl};
use futures::{future::BoxFuture, FutureExt, Stream};

struct LoggingPublisherLayer;

impl<P> PublisherLayer<P> for LoggingPublisherLayer
where
    P: Publisher + Send + Sync,
    P::Message: Display,
{
    type PublisherType = LoggingPublisher<P>;

    fn layer(&self, publisher: P) -> Self::PublisherType {
        LoggingPublisher {
            subscriber_name: None,
            publisher,
        }
    }
}

struct LoggingPublisher<P>
where
    P: Publisher,
{
    subscriber_name: Option<&'static str>,
    publisher: P,
}

impl<P> Publisher for LoggingPublisher<P>
where
    P: Publisher,
    P::Message: Display,
    Self: Sync,
{
    type Message = P::Message;
    fn get_name(&self) -> &'static str {
        self.publisher.get_name()
    }

    fn publish(&self, message: Self::Message) -> BoxFuture<async_pub_sub::Result<()>> {
        async move {
            let message_str = format!("{}", &message);
            let result = self.publisher.publish(message).await;
            log::info!(
                "[{}] -> [{}]: {}",
                self.publisher.get_name(),
                self.subscriber_name
                    .expect("subscriber name should be known"),
                message_str
            );
            result
        }
        .boxed()
    }

    fn get_message_stream(
        &mut self,
        subscriber_name: &'static str,
    ) -> Result<Pin<Box<dyn Stream<Item = Self::Message> + Send + Sync + 'static>>> {
        self.subscriber_name = Some(subscriber_name);
        self.publisher.get_message_stream(subscriber_name)
    }
}

#[test_log::test(tokio::test)]
async fn test_middleware_wrapping() -> Result<()> {
    // -- Setup & Fixtures
    let publisher = PublisherImpl::new("publisher", 10);
    let mut logging_publisher = LoggingPublisherLayer.layer(publisher);

    let mut subscriber = SubscriberImpl::new("subscriber");
    subscriber.subscribe_to(&mut logging_publisher)?;

    // -- Exercise
    let message = "Hello, World!";
    logging_publisher.publish(message.to_string()).await?;

    // -- Verification
    Ok(())
}
