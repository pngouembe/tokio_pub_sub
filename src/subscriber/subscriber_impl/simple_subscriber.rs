use std::future::Future;

use futures::{
    stream::{BoxStream, SelectAll},
    StreamExt,
};

use crate::{Publisher, Result, Subscriber};

pub struct SimpleSubscriber<Message>
where
    Message: Send + 'static,
{
    name: &'static str,
    messages: SelectAll<BoxStream<'static, Message>>,
}

impl<Message> SimpleSubscriber<Message>
where
    Message: Send + 'static,
{
    pub fn new(name: &'static str) -> Self {
        let messages = SelectAll::new();
        Self { name, messages }
    }

    pub fn subscribe_to(&mut self, publisher: &mut impl Publisher<Message>) -> Result<()> {
        let stream = publisher.get_message_stream(self.name)?;
        self.messages.push(stream);
        Ok(())
    }

    pub async fn receive(&mut self) -> Message {
        self.messages.select_next_some().await
    }
}

impl<Message> Subscriber<Message> for SimpleSubscriber<Message>
where
    Message: Send + 'static,
{
    fn get_name(&self) -> &'static str {
        self.name
    }

    fn subscribe_to(&mut self, publisher: &mut impl Publisher<Message>) -> Result<()> {
        SimpleSubscriber::subscribe_to(self, publisher)
    }

    fn receive(&mut self) -> impl Future<Output = Message> {
        SimpleSubscriber::receive(self)
    }
}
