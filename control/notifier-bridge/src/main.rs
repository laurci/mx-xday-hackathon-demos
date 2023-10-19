use amqprs::{
    channel::{BasicAckArguments, BasicConsumeArguments, Channel},
    connection::{Connection, OpenConnectionArguments},
    consumer::AsyncConsumer,
    BasicProperties, Deliver,
};
use anyhow::Result;
use tokio::sync::Notify;

const ROBOT1_ADDRESS: &str = "erd1qqqqqqqqqqqqqpgqjaqsz988zaxsfrqcych02ww3ep7qqtslmjdqxwetqe";
const ROBOT2_ADDRESS: &str = "erd1qqqqqqqqqqqqqpgqsq74pwtygz92qc4lmu2fved3ztxkq9dqmjdqmsc352";

struct Consumer;

#[async_trait::async_trait]
impl AsyncConsumer for Consumer {
    async fn consume(
        &mut self,
        channel: &Channel,
        deliver: Deliver,
        _basic_properties: BasicProperties,
        content: Vec<u8>,
    ) {
        let content = String::from_utf8(content).unwrap();

        if content.contains(ROBOT1_ADDRESS) {
            println!("robot 1 transaction found: {:?}", content);
        }

        if content.contains(ROBOT2_ADDRESS) {
            println!("robot 2 transaction found: {:?}", content);
        }

        let args = BasicAckArguments::new(deliver.delivery_tag(), false);
        channel.basic_ack(args).await.unwrap();
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenv::dotenv().ok();

    println!("robot1 address: {}", ROBOT1_ADDRESS);
    println!("robot2 address: {}", ROBOT2_ADDRESS);

    let rabbit_host = std::env::var("RABBITMQ_HOST")?;
    let rabbit_port: u16 = std::env::var("RABBITMQ_PORT")?.parse()?;
    let rabbit_username = std::env::var("RABBITMQ_USERNAME")?;
    let rabbit_password = std::env::var("RABBITMQ_PASSWORD")?;
    let rabbit_queue = std::env::var("RABBITMQ_QUEUE")?;

    let connection = Connection::open(&OpenConnectionArguments::new(
        &rabbit_host,
        rabbit_port,
        &rabbit_username,
        &rabbit_password,
    ))
    .await?;

    let channel = connection.open_channel(None).await?;
    let consume_args = BasicConsumeArguments::new(&rabbit_queue, "notifier-bridge")
        .manual_ack(true)
        .finish();

    channel.basic_consume(Consumer, consume_args).await?;

    let guard = Notify::new();
    guard.notified().await;

    Ok(())
}
