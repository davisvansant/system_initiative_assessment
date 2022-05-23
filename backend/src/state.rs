use std::collections::HashMap;

use axum::extract::ws::Message;

use tokio::sync::mpsc::{Receiver, Sender};
use tokio::sync::oneshot;

use uuid::Uuid;

pub type StateChannelReceiver = Receiver<(StateRequest, oneshot::Sender<StateResponse>)>;
pub type StateChannelSender = Sender<(StateRequest, oneshot::Sender<StateResponse>)>;

#[derive(Debug)]
pub enum StateRequest {
    AddConnection((Uuid, Sender<Message>)),
    AddMessage(Message),
    GetConnection(Uuid),
    GetConnections,
    RemoveConnection(Uuid),
}

#[derive(Debug)]
pub enum StateResponse {
    Connection(Sender<Message>),
    Connections(Vec<Sender<Message>>),
}

pub async fn add_connection(
    state: &StateChannelSender,
    connection_uuid: Uuid,
    connection: Sender<Message>,
) -> Result<(), Box<dyn std::error::Error>> {
    let (request, _response) = oneshot::channel();

    state
        .send((
            StateRequest::AddConnection((connection_uuid, connection)),
            request,
        ))
        .await?;

    Ok(())
}

pub async fn add_message(
    state: &StateChannelSender,
    message: Message,
) -> Result<(), Box<dyn std::error::Error>> {
    let (request, _response) = oneshot::channel();

    state
        .send((StateRequest::AddMessage(message), request))
        .await?;

    Ok(())
}

pub async fn get_connection(
    state: &StateChannelSender,
    connection_uuid: Uuid,
) -> Result<Sender<Message>, Box<dyn std::error::Error>> {
    let (request, response) = oneshot::channel();

    state
        .send((StateRequest::GetConnection(connection_uuid), request))
        .await?;

    match response.await {
        Ok(StateResponse::Connection(connection)) => Ok(connection),
        Ok(StateResponse::Connections(_)) => panic!("should not be retreiving all connections!"),
        Err(error) => Err(Box::new(error)),
    }
}

pub async fn get_connections(
    state: &StateChannelSender,
) -> Result<Vec<Sender<Message>>, Box<dyn std::error::Error>> {
    let (request, response) = oneshot::channel();

    state.send((StateRequest::GetConnections, request)).await?;

    match response.await {
        Ok(StateResponse::Connection(_)) => panic!("should not be retreiving connection!"),
        Ok(StateResponse::Connections(connections)) => Ok(connections),
        Err(error) => Err(Box::new(error)),
    }
}

pub async fn remove_connection(
    state: &StateChannelSender,
    connection_uuid: Uuid,
) -> Result<(), Box<dyn std::error::Error>> {
    let (_request, _response) = oneshot::channel();

    state
        .send((StateRequest::RemoveConnection(connection_uuid), _request))
        .await?;

    Ok(())
}

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
