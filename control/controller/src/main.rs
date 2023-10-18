use anyhow::Result;
use futures_util::{future, pin_mut, StreamExt};
use gilrs::Gilrs;
use std::{env, thread};
use tokio_tungstenite::connect_async;
use url::Url;

fn get_ws_url() -> Url {
    let base = env::var("UNIT_NODE_ENDPOINT").unwrap();
    url::Url::parse(&format!("{base}/ws?app=robowars")).unwrap()
}

#[derive(Clone, Debug)]
enum BusMessage {
    Start,
    End { winner_id: Option<u8> },
}

fn start_controller_thread(bus: tokio::sync::broadcast::Sender<BusMessage>) {
    thread::spawn(move || {
        let mut gilrs = Gilrs::new().unwrap();

        for (_id, gamepad) in gilrs.gamepads() {
            println!(
                "{}({:?}) is {:?}",
                gamepad.name(),
                gamepad.os_name(),
                gamepad.power_info()
            );
        }

        loop {
            while let Some(event) = gilrs.next_event() {
                println!("{:?}", event);
                match event {
                    gilrs::Event {
                        id,
                        event: gilrs::EventType::ButtonPressed(button, _),
                        time: _,
                    } => {
                        println!("[{}] button pressed {:?}", id, button)
                    }
                    gilrs::Event {
                        id,
                        event: gilrs::EventType::ButtonReleased(button, _),
                        time: _,
                    } => {
                        println!("[{}] button released {:?}", id, button)
                    }
                    gilrs::Event {
                        id,
                        event: gilrs::EventType::AxisChanged(axis, value, _),
                        time: _,
                    } => {
                        println!("[{}] axis {:?} changed to {}", id, axis, value)
                    }
                    _ => {}
                }
            }
        }
    });
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenv::dotenv()?;

    let url = get_ws_url();
    let (ws_stream, _) = connect_async(url).await.expect("Failed to connect to unit");

    let (write, mut read) = ws_stream.split();

    let (mut bus, _) = tokio::sync::broadcast::channel::<BusMessage>(1024);

    let bus_tx = bus.clone();
    let ws_tx_task = tokio::spawn(async move {
        let mut read = bus_tx.subscribe();
        while let Ok(message) = read.recv().await {
            println!("message = {:?}", message);
        }
    });

    let bus_rx = bus.clone();
    let ws_rx_task = tokio::spawn(async move {
        println!("ws_rx_task started");
        while let Some(message) = read.next().await {
            println!("message = {:?}", message);
        }
    });

    start_controller_thread(bus.clone());

    future::select(ws_tx_task, ws_rx_task).await;

    Ok(())
}
