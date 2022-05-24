use axum::extract::ws::Message;
use std::collections::HashMap;
use tokio::sync::mpsc::Sender;
use uuid::Uuid;

use crate::channel::{StateChannelReceiver, StateRequest, StateResponse};

pub struct State {
    messages: Vec<Message>,
    connections: HashMap<Uuid, Sender<Message>>,
    channel: StateChannelReceiver,
}

impl State {
    pub async fn init(channel: StateChannelReceiver) -> Result<State, Box<dyn std::error::Error>> {
        let messages = Vec::with_capacity(15);
        let connections = HashMap::with_capacity(15);

        Ok(State {
            messages,
            connections,
            channel,
        })
    }

    pub async fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        while let Some((request, response)) = self.channel.recv().await {
            match request {
                StateRequest::AddConnection((connection_uuid, connection)) => {
                    println!("adding new websocket connection!");

                    match self.connections.insert(connection_uuid, connection) {
                        Some(updated_connection) => {
                            println!(
                                "backend state -> updated connection = {:?}",
                                updated_connection,
                            );
                        }
                        None => println!("backend state -> added new connection!"),
                    }
                }
                StateRequest::AddMessage(message) => {
                    println!("adding new message -> {:?}", &message);

                    self.messages.push(message);
                }
                StateRequest::GetConnection(connection_uuid) => {
                    match self.connections.get(&connection_uuid) {
                        Some(connection) => {
                            println!("backend state -> returning connection = {:?}", connection);

                            if let Err(error) =
                                response.send(StateResponse::Connection(connection.to_owned()))
                            {
                                println!("backend state -> error sending get connection response -> {:?}", error);
                            }
                        }
                        None => println!("backend state -> requested connection not found!"),
                    }
                }
                StateRequest::GetConnections => {
                    println!("retreiving all active connections");

                    let mut connections = Vec::with_capacity(self.connections.len());

                    for connection in self.connections.values() {
                        connections.push(connection.to_owned());
                    }

                    if let Err(error) = response.send(StateResponse::Connections(connections)) {
                        println!("error sending get connections response -> {:?}", error);
                    }
                }
                StateRequest::RemoveConnection(connection_uuid) => {
                    match self.connections.remove(&connection_uuid) {
                        Some(connection) => {
                            println!("backend state -> removing connection {:?}", connection);
                        }
                        None => println!("backend state -> connection not found for removal"),
                    }
                }
            }
        }

        Ok(())
    }
}
