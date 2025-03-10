use std::fmt::Display;

use futures::{
    stream::{self, BoxStream, SelectAll},
    FutureExt, StreamExt,
};
use tokio_pub_sub::{LoggingPublisher, Publisher, Request, Result, SimpleSubscriber, Subscriber};

// todo: fix the request response logging in the forwarder
struct LoggingForwarder<Message>
where
    Message: Display + Send + 'static,
{
    name: &'static str,
    messages: Option<SelectAll<BoxStream<'static, Message>>>,
}

impl<Message> LoggingForwarder<Message>
where
    Message: Display + Send + 'static,
{
    pub fn new(name: &'static str) -> Self {
        Self {
            name,
            messages: Some(SelectAll::new()),
        }
    }
}

impl<Message> Subscriber<Message> for LoggingForwarder<Message>
where
    Message: Display + Send + 'static,
{
    fn get_name(&self) -> &'static str {
        self.name
    }

    fn subscribe_to(&mut self, publisher: &mut impl Publisher<Message>) -> Result<()> {
        let stream = publisher.get_message_stream(self.name)?;

        // todo: Fix the unwrap
        self.messages.as_mut().unwrap().push(stream);

        Ok(())
    }

    fn receive(&mut self) -> impl std::future::Future<Output = Message> {
        async move { panic!("LoggingForwarder does not implement receive method") }
    }
}

impl<Message> Publisher<Message> for LoggingForwarder<Message>
where
    Message: Display + Send + 'static,
{
    fn get_name(&self) -> &'static str {
        self.name
    }

    fn publish_event(&self, _message: Message) -> futures::future::BoxFuture<Result<()>> {
        async move { panic!("LoggingForwarder does not implement publish method") }.boxed()
    }

    fn get_message_stream(
        &mut self,
        subscriber_name: &'static str,
    ) -> Result<BoxStream<'static, Message>> {
        // todo: Fix the unwrap
        let messages = self.messages.take().unwrap();
        let name = self.name;

        let stream = stream::unfold(messages, move |mut messages| async move {
            let message = messages.select_next_some().await;
            log::info!("[{}] -> [{}]: {}", name, subscriber_name, message);
            Some((message, messages))
        })
        .boxed();

        log::info!(
            "({}) <-> ({}): {}",
            self.name,
            subscriber_name,
            std::any::type_name::<Message>()
        );

        Ok(stream)
    }
}

#[test_log::test(tokio::test)]
async fn test_message_forwarder() -> Result<()> {
    // -- Setup & Fixtures
    let mut subscriber = SimpleSubscriber::<Request<i32, i32>>::new("subscriber");
    let mut forwarder = LoggingForwarder::new("forwarder");
    let mut publisher = LoggingPublisher::new("publisher", 10);

    forwarder.subscribe_to(&mut publisher)?;
    subscriber.subscribe_to(&mut forwarder)?;

    // -- Exec
    let publisher_task = tokio::spawn(async move {
        let response = publisher.publish_request(42).await;
        assert_eq!(response.expect("request successul"), 43);
    });

    let subscriber_task = tokio::spawn(async move {
        let request = subscriber.receive().await;
        let response = request.content + 1;

        request.respond(response);
    });

    // -- Check
    tokio::try_join!(publisher_task, subscriber_task)?;

    Ok(())
}
