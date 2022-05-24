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
