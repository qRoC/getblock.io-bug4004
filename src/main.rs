use rand::Rng;
use std::pin::pin;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::time::sleep;
use tokio::time::Duration;
use tokio::time::Instant;

mod ws {
    use std::{cmp, mem, pin::pin, time::Duration};

    use futures_util::{SinkExt, StreamExt};
    use tokio::{select, sync::mpsc, time::sleep};

    pub use tokio_tungstenite::tungstenite::{
        protocol::{frame::coding::CloseCode, CloseFrame},
        Message, Result,
    };

    pub type Stream = tokio_tungstenite::WebSocketStream<
        tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
    >;

    type Sender = mpsc::UnboundedSender<Box<str>>;
    type Receiver = mpsc::UnboundedReceiver<Box<str>>;

    pub struct Transport {
        pub sender: Sender,
        pub receiver: Receiver,

        endpoint: &'static str,
    }

    impl Transport {
        pub async fn connect(endpoint: &'static str) -> Result<Self> {
            let (sender, receiver) = Self::open(endpoint).await?;

            Ok(Self {
                sender,
                receiver,
                endpoint,
            })
        }

        pub async fn reconnect(&mut self) -> (Sender, Receiver) {
            let mut attempt = 0;
            loop {
                match Self::open(self.endpoint).await {
                    Ok((sender, receiver)) => {
                        return (
                            mem::replace(&mut self.sender, sender),
                            mem::replace(&mut self.receiver, receiver),
                        );
                    }
                    Err(error) => {
                        let sleep_ms = cmp::max(100 * attempt, 3000);

                        sleep(Duration::from_millis(sleep_ms)).await;

                        attempt += 1;
                    }
                }
            }
        }
    }

    impl Transport {
        async fn open(endpoint: &str) -> Result<(Sender, Receiver)> {
            let stream = tokio_tungstenite::connect_async(endpoint)
                .await
                .map(|(ws_stream, _)| ws_stream)?;

            let (sender_tx, sender_rx) = mpsc::unbounded_channel();
            let (receiver_tx, receiver_rx) = mpsc::unbounded_channel();

            tokio::spawn(Self::worker(stream, sender_rx, receiver_tx));

            Ok((sender_tx, receiver_rx))
        }

        async fn worker(mut stream: Stream, mut sender: Receiver, receiver: Sender) {
            const KEEPALIVE: Duration = Duration::from_secs(10);

            let mut keepalive = pin!(sleep(KEEPALIVE));

            loop {
                select! {
                    biased;

                    data = sender.recv() => {
                        match data {
                            Some(data) => {
                                keepalive.set(sleep(KEEPALIVE));

                                let message = Message::Text(data.into_string());

                                if let Err(error) = stream.send(message).await {
                                    break
                                }
                            },
                            None => {

                                break
                            },
                        }
                    }
                    _ = &mut keepalive => {
                        keepalive.set(sleep(KEEPALIVE));

                        if let Err(error) = stream.send(Message::Ping(vec![])).await {
                            break
                        }
                    }
                    data = stream.next() => {
                        match data {
                            Some(Ok(Message::Text(payload))) => {
                                receiver.send(payload.into_boxed_str()).expect("'sender' channel closed unexpectedly");
                            }
                            Some(Ok(Message::Binary(payload))) => {}
                            Some(Ok(Message::Ping(_))) => {
                            }
                            Some(Ok(Message::Pong(_))) => {}
                            Some(Ok(Message::Frame(_))) => {}
                            Some(Ok(Message::Close(frame))) => {
                                break
                            }
                            Some(Err(error)) => {
                                break
                            },
                            None => {
                                break
                            },
                        }
                    }
                }
            }
        }
    }
}

async fn worker(mut transport: ws::Transport, mut request_stream: mpsc::UnboundedReceiver<String>) {
    const SEND_DELAY: Duration = Duration::from_millis(200);

    let mut buffer = Vec::with_capacity(50);
    let mut send_trigger = pin!(sleep(SEND_DELAY));

    loop {
        tokio::select! {
            biased;

            response = transport.receiver.recv() => {
                match response {
                    Some(response) => {
                        if response.as_ref().contains("gbiid") {
                            println!("BUG FOUND! Response: {}", response);
                            std::process::abort();
                        } else if response.as_ref().contains("error") {
                            println!("ERROR FOUND! Response: {}", response);
                            std::process::abort();
                        }
                    },
                    None => {
                        let (sender, mut receiver) = transport.reconnect().await;
                        //
                    }
                }
            }
            _ = &mut send_trigger => {
                if !buffer.is_empty() {
                    println!("Sending {} requests", buffer.len());
                    let payload = format!("[{}]", buffer.join(",")).into();
                    transport.sender.send(payload).expect("request stream closed unexpectedly");
                    buffer.clear();
                }
                send_trigger.as_mut().reset(Instant::now() + SEND_DELAY);
            }
            request = request_stream.recv() => {
                match request {
                    Some(request) => {
                        buffer.push(request);
                    },
                    None => {
                        break
                    },
                }
            }
        }
    }
}

#[tokio::main]
async fn main() {
    let transport = ws::Transport::connect("wss://go.getblock.io/KEY")
        .await
        .unwrap();

    let (sender, receiver) = mpsc::unbounded_channel();

    tokio::spawn(worker(transport, receiver));

    let shutdown = Arc::new(AtomicBool::new(false));

    let shutdown_signal = shutdown.clone();
    tokio::spawn(async move {
        tokio::signal::ctrl_c().await.unwrap();
        shutdown_signal.store(true, std::sync::atomic::Ordering::Relaxed);
    });

    let mut id = 1;

    // some subscriptions
    sender
        .send(format!(
            r#"{{"jsonrpc":"2.0", "id":{},"method":"eth_subscribe","params":["newHeads"]}}"#,
            id
        ))
        .unwrap();
    id += 1;

    sender
        .send(format!(
            r#"{{"jsonrpc":"2.0", "id":{},"method":"eth_subscribe","params":["newPendingTransactions"]}}"#,
            id
        ))
        .unwrap();
    id += 1;

    let mut rng = rand::thread_rng();

    // random requests
    loop {
        if shutdown.load(std::sync::atomic::Ordering::Relaxed) {
            break;
        }

        if rng.gen_bool(0.45) {
            sender
            .send(format!(
                r#"{{"jsonrpc":"2.0", "id":{},"method":"eth_getBalance","params":["0xc6e2459991BfE27cca6d86722F35da23A1E4Cb97", "latest"]}}"#,
                id
            ))
            .unwrap();
            id += 1;
        }

        if rng.gen_bool(0.45) {
            sender
            .send(format!(
                r#"{{"jsonrpc":"2.0", "id":{},"method":"eth_getBlockByHash","params":["0x9833a0c0755f264c10b1bf7bfcd2eb48ed04ce2df60c037bb6a3ce823895fcec", false]}}"#,
                id
            ))
            .unwrap();
            id += 1;
        }

        if rng.gen_bool(0.45) {
            sender
            .send(format!(
                r#"{{"jsonrpc":"2.0", "id":{},"method":"eth_getStorageAt","params":["0xc6e2459991BfE27cca6d86722F35da23A1E4Cb97", "0x6661e9d6d8b923d5bbaab1b96e1dd51ff6ea2a93520fdc9eb75d059238b8c5e9", "latest"]}}"#,
                id
            ))
            .unwrap();
            id += 1;
        }

        sleep(Duration::from_millis(rng.gen_range(15..60))).await;
    }
}
