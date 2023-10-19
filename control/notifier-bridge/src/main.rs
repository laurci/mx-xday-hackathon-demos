use amqprs::{
    channel::{BasicAckArguments, BasicConsumeArguments, Channel},
    connection::{Connection, OpenConnectionArguments},
    consumer::AsyncConsumer,
    BasicProperties, Deliver,
};
use anyhow::Result;
use serde::Deserialize;
use tokio::sync::Notify;

const ROBOT1_ADDRESS: &str = "erd1qqqqqqqqqqqqqpgqjaqsz988zaxsfrqcych02ww3ep7qqtslmjdqxwetqe";
const ROBOT2_ADDRESS: &str = "erd1qqqqqqqqqqqqqpgqsq74pwtygz92qc4lmu2fved3ztxkq9dqmjdqmsc352";

#[derive(Deserialize, Clone, Debug)]
struct ChainEvent {
    pub address: String,
    pub identifier: String,
    pub topics: Vec<String>,
    pub data: String,
    #[serde(rename = "txHash")]
    pub tx_hash: String,
}

fn try_parse_chain_event_for_address(text: &String, address: &str) -> Option<ChainEvent> {
    if let Some(start_idx) = text.find(address) {
        let after_bracket_idx = start_idx + text[start_idx..].find('}').unwrap();
        let before_bracket_idx = start_idx - "{\"address\":\"".len();
        let json = &text[before_bracket_idx..after_bracket_idx + 1];
        if let Ok(data) = serde_json::from_str::<ChainEvent>(json) {
            return Some(data);
        }
    }
    None
}

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

        if let Some(event) = try_parse_chain_event_for_address(&content, ROBOT1_ADDRESS) {
            if event.identifier == "join" {
                println!("robot 1 transaction found: {:?}", event);
            }
        }

        if let Some(event) = try_parse_chain_event_for_address(&content, ROBOT2_ADDRESS) {
            if event.identifier == "join" {
                println!("robot 2 transaction found: {:?}", event);
            }
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
