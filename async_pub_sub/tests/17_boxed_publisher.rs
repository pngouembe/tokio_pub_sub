use async_pub_sub::{Publisher, PublisherImpl, Result, SubscriberImpl};

#[tokio::test]
async fn test_pub_sub_i32() -> Result<()> {
    // -- Setup & Fixtures
    let mut subscriber = SubscriberImpl::new("subscriber");
    let mut publisher: Box<dyn Publisher<Message = i32>> =
        Box::new(PublisherImpl::new("publisher", 10));

    subscriber.subscribe_to(&mut publisher)?;

    // -- Exec
    publisher.publish(42).await?;
    let message = subscriber.receive().await;

    // -- Check
    assert_eq!(message, 42);

    Ok(())
}
