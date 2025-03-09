use std::fmt::Display;

use tokio_pub_sub::{LoggingPublisher, Publisher, Request, Result, SimpleSubscriber, Subscriber};

#[derive(Debug, PartialEq)]
struct Foo(i32);

#[derive(Debug, PartialEq)]
struct Bar(String);

#[derive(Debug)]
enum ServiceRequest {
    Foo(Request<Foo, i32>),
    Bar(Request<Bar, String>),
}

impl Display for ServiceRequest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

struct Service {
    subscriber: SimpleSubscriber<ServiceRequest>,
}

impl Service {
    pub fn new() -> Self {
        let subscriber = SimpleSubscriber::new("Service");

        Self { subscriber }
    }

    pub async fn run(&mut self) -> Result<()> {
        loop {
            let request = self.subscriber.receive().await;
            self.handle_request(request);
        }
    }

    fn handle_request(&mut self, request: ServiceRequest) {
        match request {
            ServiceRequest::Foo(request) => {
                let content = &request.content;
                request.respond(self.foo(&content));
            }
            ServiceRequest::Bar(ref request) => {
                todo!()
            }
        }
        {}
    }

    fn foo(&self, value: &Foo) -> i32 {
        let Foo(value) = value;
        value + 1
    }

    fn bar(&mut self, value: Bar) -> String {
        let Bar(value) = value;
        format!("bar: {}", value)
    }
}

impl Subscriber<ServiceRequest> for Service {
    fn get_name(&self) -> &'static str {
        self.subscriber.get_name()
    }

    fn subscribe_to(&mut self, publisher: &mut impl Publisher<ServiceRequest>) -> Result<()> {
        self.subscriber.subscribe_to(publisher)
    }

    fn receive(&mut self) -> impl std::future::Future<Output = ServiceRequest> {
        self.subscriber.receive()
    }
}

#[test_log::test(tokio::test)]
async fn test_multiple_publishers() -> Result<()> {
    // -- Setup & Fixtures
    let mut publisher = LoggingPublisher::new("publisher", 1);
    let mut subscriber = SimpleSubscriber::new("subscriber");
    let mut service = Service::new();

    service.subscribe_to(&mut publisher)?;
    subscriber.subscribe_to(&mut service)?;

    tokio::spawn(async move {
        service.run().await.unwrap();
    });

    // -- Exec
    publisher.publish_event(42).await?;

    let message = subscriber.receive().await;

    // -- Check
    assert_eq!(message, 42.to_string());

    Ok(())
}
