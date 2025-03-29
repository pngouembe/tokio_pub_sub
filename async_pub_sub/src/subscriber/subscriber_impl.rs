use std::{future::Future, pin::Pin};

use futures::{stream::SelectAll, Stream, StreamExt};

use crate::{PublisherWrapper, Result, Subscriber};

pub struct SubscriberImpl<Message>
where
    Message: Send + 'static,
{
    name: &'static str,
    messages: SelectAll<Pin<Box<dyn Stream<Item = Message> + Send + Sync + 'static>>>,
}

impl<Message> SubscriberImpl<Message>
where
    Message: Send + 'static,
{
    pub fn new(name: &'static str) -> Self {
        let messages = SelectAll::new();
        Self { name, messages }
    }

    pub fn subscribe_to(&mut self, publisher: &mut impl PublisherWrapper<Message>) -> Result<()> {
        let stream = publisher.get_message_stream(self.name)?;
        self.messages.push(stream);
        Ok(())
    }

    pub async fn receive(&mut self) -> Message {
        self.messages.select_next_some().await
    }
}

impl<Message> Subscriber for SubscriberImpl<Message>
where
    Message: Send + 'static,
{
    type Message = Message;

    fn get_name(&self) -> &'static str {
        self.name
    }

    fn subscribe_to(&mut self, publisher: &mut impl PublisherWrapper<Self::Message>) -> Result<()> {
        SubscriberImpl::subscribe_to(self, publisher)
    }

    fn receive(&mut self) -> impl Future<Output = Message> {
        SubscriberImpl::receive(self)
    }
}
