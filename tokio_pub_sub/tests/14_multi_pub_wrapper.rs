use tokio_pub_sub::{Result, SimplePublisher, SimpleSubscriber};
use tokio_pub_sub_macros::DerivePublisher;

#[derive(DerivePublisher)]
struct MultiPub {
    #[publisher(i32)]
    publisher_a: SimplePublisher<i32>,
    #[publisher(String)]
    publisher_b: SimplePublisher<String>,
}

impl MultiPub {
    fn new() -> Self {
        Self {
            publisher_a: SimplePublisher::new("publisher", 1),
            publisher_b: SimplePublisher::new("publisher", 1),
        }
    }
}

#[tokio::test]
async fn test_multi_pub() -> Result<()> {
    let mut subscriber1 = SimpleSubscriber::<i32>::new("subscriber1");
    let mut subscriber2 = SimpleSubscriber::<String>::new("subscriber2");

    let mut publisher = MultiPub::new();
    subscriber1.subscribe_to(&mut publisher)?;
    subscriber2.subscribe_to(&mut publisher)?;

    Ok(())
}
