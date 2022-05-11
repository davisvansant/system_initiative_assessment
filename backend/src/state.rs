use axum::extract::ws::Message;

use tokio::sync::mpsc::{Receiver, Sender};
use tokio::sync::oneshot;

pub type StateChannelReceiver = Receiver<(StateRequest, oneshot::Sender<StateResponse>)>;
pub type StateChannelSender = Sender<(StateRequest, oneshot::Sender<StateResponse>)>;

#[derive(Debug)]
pub enum StateRequest {
    AddConnection(Sender<Message>),
    AddMessage(Message),
    GetConnections,
}

#[derive(Debug)]
pub enum StateResponse {
    Connections(Vec<Sender<Message>>),
}

pub async fn add_connection(
    state: &StateChannelSender,
    connection: Sender<Message>,
) -> Result<(), Box<dyn std::error::Error>> {
    let (request, _response) = oneshot::channel();

    state
        .send((StateRequest::AddConnection(connection), request))
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

pub async fn get_connections(
    state: &StateChannelSender,
) -> Result<Vec<Sender<Message>>, Box<dyn std::error::Error>> {
    let (request, response) = oneshot::channel();

    state.send((StateRequest::GetConnections, request)).await?;

    match response.await {
        Ok(StateResponse::Connections(connections)) => Ok(connections),
        Err(error) => Err(Box::new(error)),
    }
}

pub struct State {
    messages: Vec<Message>,
    connections: Vec<Sender<Message>>,
    channel: StateChannelReceiver,
}

impl State {
    pub async fn init(channel: StateChannelReceiver) -> Result<State, Box<dyn std::error::Error>> {
        let messages = Vec::with_capacity(15);
        let connections = Vec::with_capacity(15);

        Ok(State {
            messages,
            connections,
            channel,
        })
    }

    pub async fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        while let Some((request, response)) = self.channel.recv().await {
            match request {
                StateRequest::AddConnection(connection) => {
                    println!("adding new websocket connection!");

                    self.connections.push(connection);
                }
                StateRequest::AddMessage(message) => {
                    println!("adding new message -> {:?}", &message);

                    self.messages.push(message);
                }
                StateRequest::GetConnections => {
                    println!("retreiving all active connections");

                    let connections = self.connections.to_vec();

                    if let Err(error) = response.send(StateResponse::Connections(connections)) {
                        println!("error sending get connections response -> {:?}", error);
                    }
                }
            }
        }

        Ok(())
    }
}
