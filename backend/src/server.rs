use std::net::SocketAddr;

use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::response::IntoResponse;
use axum::routing::get;
use axum::Extension;
use axum::Router;

use futures::sink::SinkExt;
use futures::stream::{SplitSink, SplitStream, StreamExt};

use tokio::sync::mpsc;
use tokio::sync::mpsc::{Receiver, Sender};

use crate::state::{add_connection, add_message, get_connections};
use crate::StateChannelSender;

pub struct Server {
    socket_address: SocketAddr,
    state: StateChannelSender,
}

impl Server {
    pub async fn init(
        socket_address: SocketAddr,
        state: StateChannelSender,
    ) -> Result<Server, Box<dyn std::error::Error>> {
        Ok(Server {
            socket_address,
            state,
        })
    }

    pub async fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("running server -> {:?}", &self.socket_address);

        axum::Server::bind(&self.socket_address)
            .serve(self.router().await.into_make_service())
            .await?;

        Ok(())
    }

    async fn router(&self) -> Router {
        let state = self.state.to_owned();

        Router::new()
            .route("/messages", get(Server::connection))
            .layer(Extension(state))
    }

    async fn connection(
        connection: WebSocketUpgrade,
        Extension(state): Extension<StateChannelSender>,
    ) -> impl IntoResponse {
        println!("connection -> {:?}", &connection);

        connection.on_upgrade(|websocket| Server::websocket(websocket, state))
    }

    async fn websocket(websocket: WebSocket, state: StateChannelSender) {
        let (mut sender, receiver) = websocket.split();
        let (sink_sender, mut sink_receiver) = mpsc::channel::<Message>(16);

        if let Err(error) = add_connection(&state, sink_sender.to_owned()).await {
            println!("error adding connection to state -> {:?}", error);
        }

        tokio::spawn(async move {
            if let Err(error) = Self::write_connection(&mut sink_receiver, &mut sender).await {
                println!("error writing connection -> {:?}", error);
            }
        });

        tokio::spawn(async move {
            if let Err(error) = Self::read_connection(&state, sink_sender, receiver).await {
                println!("error reading connection! -> {:?}", error);
            }
        });
    }

    async fn read_connection(
        state: &StateChannelSender,
        channel: Sender<Message>,
        mut receiver: SplitStream<WebSocket>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        while let Some(incoming) = receiver.next().await {
            println!("incoming message! {:?}", &incoming);
            match incoming {
                Ok(Message::Text(text)) => {
                    println!("message text -> {:?}", text);

                    add_message(state, Message::Text(text.to_owned())).await?;

                    let connections = get_connections(state).await?;

                    for connection in connections {
                        connection.send(Message::Text(text.to_owned())).await?;
                    }
                }
                Ok(Message::Binary(binary)) => println!("message binary -> {:?}", binary),
                Ok(Message::Ping(ping)) => println!("received ping! -> {:?}", ping),
                Ok(Message::Pong(pong)) => println!("received pong! -> {:?}", pong),
                Ok(Message::Close(Some(close))) => {
                    println!("received close! -> {:?}", close);

                    channel.send(Message::Close(Some(close))).await?;
                }
                Ok(Message::Close(None)) => {
                    println!("received close!");

                    channel.send(Message::Close(None)).await?;
                }
                Err(error) => println!("error receiving message -> {:?}", error),
            }
        }

        Ok(())
    }
    async fn write_connection(
        channel: &mut Receiver<Message>,
        sender: &mut SplitSink<WebSocket, Message>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        while let Some(message) = channel.recv().await {
            match message {
                Message::Text(text) => sender.send(Message::Text(text)).await?,
                Message::Binary(binary) => sender.send(Message::Binary(binary)).await?,
                Message::Ping(ping) => sender.send(Message::Ping(ping)).await?,
                Message::Pong(pong) => sender.send(Message::Pong(pong)).await?,
                Message::Close(_close_frame) => {
                    channel.close();

                    sender.close().await?;
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{StateRequest, StateResponse};
    use axum::body::Body;
    use axum::http::version::Version;
    use axum::http::{Request, StatusCode};
    use std::str::FromStr;
    use tokio::sync::mpsc;
    use tokio::sync::oneshot;

    #[tokio::test(flavor = "multi_thread")]
    async fn run() -> Result<(), Box<dyn std::error::Error>> {
        let test_address = SocketAddr::from_str("127.0.0.1:5555")?;
        let (test_state_sender, _test_state_receiver) =
            mpsc::channel::<(StateRequest, oneshot::Sender<StateResponse>)>(64);
        let mut test_server = Server::init(test_address, test_state_sender).await?;

        tokio::spawn(async move {
            if let Err(error) = test_server.run().await {
                println!("error with test server -> {:?}", error);
            }
        });

        let test_client = hyper::Client::new();

        let test_request = Request::builder()
            .uri("http://127.0.0.1:5555/messages")
            .header("connection", "upgrade")
            .header("upgrade", "websocket")
            .header("sec-websocket-version", "13")
            .header("sec-websocket-key", "dGhlIHNhbXBsZSBub25jZQ==")
            .body(Body::empty())?;

        let test_response = test_client.request(test_request).await?;

        assert_eq!(test_response.version(), Version::HTTP_11);
        assert_eq!(test_response.status(), StatusCode::SWITCHING_PROTOCOLS);

        let test_response_headers = test_response.headers();

        assert_eq!(test_response_headers.get("connection").unwrap(), "upgrade");
        assert_eq!(test_response_headers.get("upgrade").unwrap(), "websocket");

        Ok(())
    }
}
