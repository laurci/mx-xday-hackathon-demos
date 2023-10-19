use std::time::Duration;

use anyhow::Result;
use futures::StreamExt;
use rabbitmq_stream_client::Environment;

const RABBITMQ_QUEUE: &str = "all_events";

const ROBOT1_ADDRESS: &str = "erd1e22refv0l35v58evpaysnvwjnuzn62stanetew0kexfgpqrg9f0s2krnek";
const ROBOT2_ADDRESS: &str = "erd1vc5769khq85nlh86hplp6esnpfx5dzgwmeh4dsrd09ce8qc4na3q882pk5";

// struct Consumer;
//
// #[async_trait::async_trait]
// impl AsyncConsumer for Consumer {
//     async fn consume(
//         &mut self,
//         channel: &Channel,
//         deliver: Deliver,
//         _basic_properties: BasicProperties,
//         content: Vec<u8>,
//     ) {
//         let content = String::from_utf8(content).unwrap();
//         if content.contains(ROBOT1_ADDRESS) {
//             println!("robot 1 transaction found: {:?}", content);
//         } else if content.contains(ROBOT2_ADDRESS) {
//             println!("robot 2 transaction found: {:?}", content);
//         }
//
//         let args = BasicAckArguments::new(deliver.delivery_tag(), false);
//         channel.basic_ack(args).await.unwrap();
//     }
// }

#[tokio::main]
async fn main() -> Result<()> {
    println!("robot1 address: {}", ROBOT1_ADDRESS);
    println!("robot2 address: {}", ROBOT2_ADDRESS);

    let environment = Environment::builder()
        .host("localhost")
        .port(5672)
        .username("guest")
        .password("guest")
        .build()
        .await?;

    println!("built environment");

    let mut consumer = environment.consumer().build(RABBITMQ_QUEUE).await?;
    println!("built consumer");

    let handle = consumer.handle();
    println!("got handle");

    tokio::spawn(async move {
        while let Some(delivery) = consumer.next().await {
            println!("Got message {:?}", delivery);
        }
    });

    println!("started consumer");

    tokio::time::sleep(Duration::from_secs(10)).await;
    println!("closing consumer");

    handle.close().await?;

    Ok(())
}

