use anyhow::Result;
use futures_util::StreamExt;
use gilrs::Gilrs;
use std::{env, io, process::Command, str, thread};
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
    Join { robot_id: u8, address: String },
    Update { values: Vec<i8> },
    End { winner_id: Option<u8> },
}

fn send_nft(to: String) {
    println!("send nft to {}", to);

    let Ok(out) = Command::new("../send-nft.sh").arg(to).output() else {
        println!("failed to send nft");
        return;
    };
    if out.stderr.len() > 0 {
        println!("stderr: {}", str::from_utf8(&out.stderr).unwrap());
    }
}

fn start_controller_thread(bus: tokio::sync::broadcast::Sender<BusMessage>) {
    thread::spawn(move || {
        let mut gilrs = Gilrs::new().unwrap();

        let mut values = vec![0i8; 4];

        bus.send(BusMessage::Update {
            values: values.clone(),
        })
        .unwrap();

        let mut last_update = std::time::Instant::now();

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

                        if last_update.elapsed().as_millis() > 5 {
                            last_update = std::time::Instant::now();
                            bus.send(BusMessage::Update {
                                values: values.clone(),
                            })
                            .unwrap();
                        }
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
    let (_, mut ws_rx) = ws_stream.split();

    let mut serial_rx = tokio_serial::new("/dev/cu.usbmodem1101", 9600).open_native_async()?;
    serial_rx
        .set_exclusive(false)
        .expect("Unable to set serial port exclusive to false");

    let mut serial_reader = LineCodec.framed(serial_rx);
    let mut serial_tx = tokio_serial::new("/dev/cu.usbmodem1101", 9600).open_native_async()?;

    let (bus, _) = tokio::sync::broadcast::channel::<BusMessage>(10_000);

    let bus_rx = bus.clone();
    let ws_rx_task = tokio::spawn(async move {
        println!("ws_rx_task started");
        while let Some(message) = ws_rx.next().await {
            println!("ws_rx_task received message {:?}", message);

            let message = message.unwrap();
            match message {
                tokio_tungstenite::tungstenite::Message::Text(text) => {
                    if text.starts_with("join_") {
                        let data = text[5..].to_string();
                        let robot = if data.starts_with("robot1:") {
                            1
                        } else if data.starts_with("robot2:") {
                            2
                        } else {
                            continue;
                        };
                        let address = data[7..].to_string();

                        bus_rx
                            .send(BusMessage::Join {
                                robot_id: robot,
                                address,
                            })
                            .unwrap();
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
        let mut robot1_address: Option<String> = None;
        let mut robot2_address: Option<String> = None;

        while let Ok(message) = read.recv().await {
            match message {
                BusMessage::Update { values } => {
                    if active {
                        let value = format!(
                            "p {} {} {} {}\n",
                            values[0], values[1], values[2], values[3]
                        );

                        serial_tx.write_all(value.as_bytes()).await.unwrap();
                    }
                }
                BusMessage::Join { robot_id, address } => {
                    if robot_id == 1 {
                        robot1_address = Some(address);
                    } else if robot_id == 2 {
                        robot2_address = Some(address);
                    }

                    if robot1_address.is_some() && robot2_address.is_some() {
                        active = true;
                        println!("activate bots");
                    }
                }
                BusMessage::End { winner_id } => {
                    if !active {
                        continue;
                    }

                    println!("deactivate bots");

                    active = false;

                    let value = "p 0 0 0 0\n";
                    serial_tx.write_all(value.as_bytes()).await.unwrap();

                    if !winner_id.is_none() {
                        let winner_id = winner_id.unwrap();
                        let address = if winner_id == 1 {
                            robot1_address.unwrap()
                        } else {
                            robot2_address.unwrap()
                        };

                        send_nft(address);
                    }

                    robot1_address = None;
                    robot2_address = None;
                }
            }
        }
    });

    start_controller_thread(bus.clone());

    ws_rx_task.await?;

    Ok(())
}
