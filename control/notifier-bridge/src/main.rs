use amqprs::{
    channel::{BasicAckArguments, BasicConsumeArguments, Channel},
    connection::{Connection, OpenConnectionArguments},
    consumer::AsyncConsumer,
    BasicProperties, Deliver,
};
use anyhow::Result;
use tokio::sync::Notify;

const RABBITMQ_HOST: &str = "localhost";
const RABBITMQ_PORT: u16 = 5672;
const RABBITMQ_USERNAME: &str = "guest";
const RABBITMQ_PASSWORD: &str = "guest";

const RABBITMQ_QUEUE: &str = "all_events";

const ROBOT1_ADDRESS: &str = "erd1e22refv0l35v58evpaysnvwjnuzn62stanetew0kexfgpqrg9f0s2krnek";
const ROBOT2_ADDRESS: &str = "erd1vc5769khq85nlh86hplp6esnpfx5dzgwmeh4dsrd09ce8qc4na3q882pk5";

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
        } else if content.contains(ROBOT2_ADDRESS) {
            println!("robot 2 transaction found: {:?}", content);
        }

        let args = BasicAckArguments::new(deliver.delivery_tag(), false);
        channel.basic_ack(args).await.unwrap();
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    println!("robot1 address: {}", ROBOT1_ADDRESS);
    println!("robot2 address: {}", ROBOT2_ADDRESS);

    let connection = Connection::open(&OpenConnectionArguments::new(
        RABBITMQ_HOST,
        RABBITMQ_PORT,
        RABBITMQ_USERNAME,
        RABBITMQ_PASSWORD,
    ))
    .await?;

    let channel = connection.open_channel(None).await?;
    let consume_args = BasicConsumeArguments::new(RABBITMQ_QUEUE, "mainnet")
        .manual_ack(true)
        .finish();

    channel.basic_consume(Consumer, consume_args).await?;

    let guard = Notify::new();
    guard.notified().await;

    Ok(())
}

