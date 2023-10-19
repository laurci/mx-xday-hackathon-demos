use anyhow::Result;
use futures_util::{future, SinkExt, StreamExt};
use gilrs::Gilrs;
use std::{env, io, str, thread};
use tokio::io::AsyncWriteExt;
use tokio_serial::SerialPortBuilderExt;
use tokio_tungstenite::connect_async;
use tokio_util::{
    bytes::BytesMut,
    codec::{Decoder, Encoder},
};
use url::Url;

fn get_ws_url() -> Url {
    let base = env::var("UNIT_NODE_ENDPOINT").unwrap();
    url::Url::parse(&format!("{base}/ws?app=robotwars")).unwrap()
}

#[derive(Clone, Debug)]
enum BusMessage {
    Start,
    Update { values: Vec<i8> },
    End { winner_id: Option<u8> },
}

fn start_controller_thread(bus: tokio::sync::broadcast::Sender<BusMessage>) {
    thread::spawn(move || {
        let mut gilrs = Gilrs::new().unwrap();

        let mut values = vec![0i8; 4];

        bus.send(BusMessage::Update {
            values: values.clone(),
        })
        .unwrap();

        loop {
            while let Some(event) = gilrs.next_event() {
                match event {
                    gilrs::Event {
                        id,
                        event: gilrs::EventType::AxisChanged(axis, value, _),
                        time: _,
                    } => {
                        let axis_id = match axis {
                            gilrs::Axis::LeftStickY => 0i8,
                            gilrs::Axis::RightStickX => 1i8,
                            _ => -1i8,
                        };

                        if axis_id < 0 {
                            continue;
                        }

                        let gamepad_id: i8 = format!("{}", id).parse().unwrap();

                        let index = gamepad_id * 2 + axis_id;

                        // map value from -1.0..1.0 to -100..100
                        let mut value = (value * 100.0) as i8;
                        if value.abs() < 5 {
                            value = 0;
                        }
                        let old_value = values[index as usize];
                        if old_value == value {
                            continue;
                        }

                        values[index as usize] = value;

                        bus.send(BusMessage::Update {
                            values: values.clone(),
                        })
                        .unwrap();
                    }
                    _ => {}
                }
            }
        }
    });
}

struct LineCodec;

impl Decoder for LineCodec {
    type Item = String;
    type Error = io::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        let newline = src.as_ref().iter().position(|b| *b == b'\n');
        if let Some(n) = newline {
            let line = src.split_to(n + 1);
            return match str::from_utf8(line.as_ref()) {
                Ok(s) => Ok(Some(s.to_string())),
                Err(_) => Err(io::Error::new(io::ErrorKind::Other, "Invalid String")),
            };
        }
        Ok(None)
    }
}

impl Encoder<String> for LineCodec {
    type Error = io::Error;

    fn encode(&mut self, _item: String, _dst: &mut BytesMut) -> Result<(), Self::Error> {
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenv::dotenv()?;

    let url = get_ws_url();
    let (ws_stream, _) = connect_async(url).await.expect("Failed to connect to unit");
    let (mut ws_tx, mut ws_rx) = ws_stream.split();

    let mut serial_rx = tokio_serial::new("/dev/cu.usbmodem1101", 9600).open_native_async()?;
    serial_rx
        .set_exclusive(false)
        .expect("Unable to set serial port exclusive to false");

    let mut serial_reader = LineCodec.framed(serial_rx);
    let mut serial_tx = tokio_serial::new("/dev/cu.usbmodem1101", 9600).open_native_async()?;

    let (bus, _) = tokio::sync::broadcast::channel::<BusMessage>(10_000);

    let bus_tx = bus.clone();
    let ws_tx_task = tokio::spawn(async move {
        let mut read = bus_tx.subscribe();
        while let Ok(message) = read.recv().await {
            match message {
                BusMessage::End { winner_id } => {
                    let message = format!("winner_{}", winner_id.unwrap_or(0));
                    ws_tx
                        .send(tokio_tungstenite::tungstenite::Message::Text(message))
                        .await
                        .unwrap();
                }
                _ => {}
            }
        }
    });

    let bus_rx = bus.clone();
    let ws_rx_task = tokio::spawn(async move {
        println!("ws_rx_task started");
        while let Some(message) = ws_rx.next().await {
            let message = message.unwrap();
            match message {
                tokio_tungstenite::tungstenite::Message::Text(text) => {
                    if text == "start" {
                        bus_rx.send(BusMessage::Start).unwrap();
                    }
                }
                _ => {}
            }
        }
    });

    let bus_serial_rx = bus.clone();
    tokio::spawn(async move {
        while let Some(line_result) = serial_reader.next().await {
            let line = line_result.expect("Failed to read line");
            let line = line.trim();
            if line.starts_with("lost_") {
                let loser_id: u8 = line[5..].parse().unwrap();
                match loser_id {
                    1 => bus_serial_rx
                        .send(BusMessage::End { winner_id: Some(2) })
                        .unwrap(),
                    2 => bus_serial_rx
                        .send(BusMessage::End { winner_id: Some(1) })
                        .unwrap(),
                    _ => 0,
                };
            }
        }
    });

    let bus_serial_tx = bus.clone();
    tokio::spawn(async move {
        let mut read = bus_serial_tx.subscribe();
        let mut active = true;
        while let Ok(message) = read.recv().await {
            match message {
                BusMessage::Update { values } => {
                    if active {
                        let message = format!(
                            "p {} {} {} {}\n",
                            values[0], values[1], values[2], values[3]
                        );
                        println!("pushing {}", message);
                        serial_tx.write_all(message.as_bytes()).await.unwrap();
                    }
                }
                BusMessage::Start => {
                    active = true;
                    println!("activate bots");
                }
                BusMessage::End { .. } => {
                    active = false;
                    println!("deactivate bots");

                    let message = "p 0 0 0 0\n";
                    println!("pushing {}", message);
                    serial_tx.write_all(message.as_bytes()).await.unwrap();
                }
            }
        }
    });

    start_controller_thread(bus.clone());

    future::select(ws_tx_task, ws_rx_task).await;

    Ok(())
}
