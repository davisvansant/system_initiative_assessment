use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::str::FromStr;
use tokio::sync::{mpsc, oneshot};

use channel::{StateChannelSender, StateRequest, StateResponse};
use server::Server;
use state::State;

mod channel;
mod server;
mod state;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let address = IpAddr::V4(Ipv4Addr::from_str("0.0.0.0")?);
    let port = 5555;
    let socket_address = SocketAddr::new(address, port);

    let (state_sender, state_receiver) =
        mpsc::channel::<(StateRequest, oneshot::Sender<StateResponse>)>(64);

    let mut state = State::init(state_receiver).await?;

    let state_task = tokio::spawn(async move {
        if let Err(error) = state.run().await {
            println!("error with state -> {:?}", error);
        }
    });

    let mut server = Server::init(socket_address, state_sender).await?;

    let server_task = tokio::spawn(async move {
        if let Err(error) = server.run().await {
            println!("error with server -> {:?}", error);
        }
    });

    tokio::try_join!(state_task, server_task)?;

    Ok(())
}
