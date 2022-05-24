use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::response::IntoResponse;
use axum::routing::get;
use axum::Extension;
use axum::Router;
use futures::sink::SinkExt;
use futures::stream::{SplitSink, SplitStream, StreamExt};
use std::net::SocketAddr;
use tokio::sync::mpsc;
use tokio::sync::mpsc::Receiver;
use uuid::Uuid;

use crate::channel::{
    add_connection, add_message, get_connection, get_connections, remove_connection,
};
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

        let connection_uuid = Uuid::new_v4();

        if let Err(error) = add_connection(&state, connection_uuid.to_owned(), sink_sender).await {
            println!("error adding connection to state -> {:?}", error);
        }

        tokio::spawn(async move {
            if let Err(error) = Self::write_connection(&mut sink_receiver, &mut sender).await {
                println!("error writing connection -> {:?}", error);
            }
        });

        tokio::spawn(async move {
            if let Err(error) = Self::read_connection(connection_uuid, &state, receiver).await {
                println!("error reading connection! -> {:?}", error);
            }
        });
    }

    async fn read_connection(
        connection_uuid: Uuid,
        state: &StateChannelSender,
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

                    let connection = get_connection(state, connection_uuid).await?;

                    connection.send(Message::Close(Some(close))).await?;

                    remove_connection(state, connection_uuid).await?;
                }
                Ok(Message::Close(None)) => {
                    println!("received close!");

                    let connection = get_connection(state, connection_uuid).await?;

                    connection.send(Message::Close(None)).await?;

                    remove_connection(state, connection_uuid).await?;
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
                    sender.close().await?;

                    channel.close();
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
    use axum::http::version::Version;
    use axum::http::StatusCode;
    use std::str::FromStr;
    use tokio::sync::oneshot;
    use tokio_tungstenite::connect_async;
    use tokio_tungstenite::tungstenite::protocol::frame::coding::CloseCode;
    use tokio_tungstenite::tungstenite::protocol::frame::CloseFrame;
    use tokio_tungstenite::tungstenite::Message;

    #[tokio::test(flavor = "multi_thread")]
    async fn run() -> Result<(), Box<dyn std::error::Error>> {
        let test_address = SocketAddr::from_str("127.0.0.1:5555")?;
        let (test_state_sender, test_state_receiver) =
            mpsc::channel::<(StateRequest, oneshot::Sender<StateResponse>)>(64);

        let mut test_state = crate::State::init(test_state_receiver).await?;

        tokio::spawn(async move {
            if let Err(error) = test_state.run().await {
                println!("error with test state -> {:?}", error);
            }
        });

        let mut test_server = Server::init(test_address, test_state_sender).await?;

        tokio::spawn(async move {
            if let Err(error) = test_server.run().await {
                println!("error with test server -> {:?}", error);
            }
        });

        let test_websocket_address = "ws://127.0.0.1:5555/messages";

        let (test_websocket_stream, test_websocket_response) =
            connect_async(test_websocket_address).await?;

        assert_eq!(test_websocket_response.version(), Version::HTTP_11);
        assert_eq!(
            test_websocket_response.status(),
            StatusCode::SWITCHING_PROTOCOLS,
        );

        let test_websocket_response_headers = test_websocket_response.headers();

        assert_eq!(
            test_websocket_response_headers.get("connection").unwrap(),
            "upgrade",
        );
        assert_eq!(
            test_websocket_response_headers.get("upgrade").unwrap(),
            "websocket",
        );
        assert!(test_websocket_response_headers.contains_key("sec-websocket-accept"));
        assert!(test_websocket_response_headers
            .get("sec-websocket-protocol")
            .is_none());

        let (mut test_websocket_client_writer, mut test_websocket_client_reader) =
            test_websocket_stream.split();

        let test_websocket_sender = tokio::spawn(async move {
            tokio::time::sleep(core::time::Duration::from_millis(2000)).await;

            test_websocket_client_writer
                .send(Message::Text(String::from("test_message")))
                .await
                .unwrap();

            tokio::time::sleep(core::time::Duration::from_millis(2000)).await;

            test_websocket_client_writer
                .send(Message::Close(Some(CloseFrame {
                    code: CloseCode::Normal,
                    reason: std::borrow::Cow::Borrowed("test_goodbye!"),
                })))
                .await
                .unwrap();

            tokio::time::sleep(core::time::Duration::from_millis(2000)).await;
        });

        while let Some(test_incoming_message) = test_websocket_client_reader.next().await {
            match test_incoming_message {
                Ok(Message::Text(test_text_message)) => {
                    assert_eq!(test_text_message.as_str(), "test_message");
                }
                Ok(Message::Binary(_)) => unimplemented!(),
                Ok(Message::Ping(_)) => unimplemented!(),
                Ok(Message::Pong(_)) => unimplemented!(),
                Ok(Message::Close(test_close_message)) => {
                    assert!(test_close_message.is_some());
                    assert_eq!(
                        test_close_message
                            .as_ref()
                            .unwrap()
                            .code
                            .to_string()
                            .as_str(),
                        "1000",
                    );
                    assert_eq!(test_close_message.as_ref().unwrap().reason, "test_goodbye!");
                }
                Ok(Message::Frame(_)) => unimplemented!(),
                Err(_) => panic!("unexpected error in test websocket reader!"),
            }
        }

        assert!(test_websocket_sender.await.is_ok());

        Ok(())
    }
}
