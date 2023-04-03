use async_std::channel::{unbounded, Receiver, Sender};
use async_std::net::{SocketAddr, TcpListener, TcpStream};
use async_tungstenite::{
    accept_async,
    tungstenite::{Error, Result},
    WebSocketStream,
};
use futures::{prelude::*, stream::SplitSink};
use log::*;
use tungstenite::Message;

enum NewNotification {
    NewClient((SocketAddr, SplitSink<WebSocketStream<TcpStream>, Message>)),
    NewMessage((SocketAddr, String)),
}

async fn accept_connection(
    peer: SocketAddr,
    stream: TcpStream,
    new_server_request_data_tx: Sender<NewNotification>,
) {
    if let Err(e) = handle_connection(peer, stream, new_server_request_data_tx).await {
        match e {
            Error::ConnectionClosed | Error::Protocol(_) | Error::Utf8 => (),
            err => error!("Error processing connection: {}", err),
        }
    }
}

async fn handle_connection(
    peer: SocketAddr,
    stream: TcpStream,
    new_server_request_data_tx: Sender<NewNotification>,
) -> Result<()> {
    let ws_stream = accept_async(stream).await.expect("Failed to accept");
    let (sender, mut receiver) = ws_stream.split();
    new_server_request_data_tx
        .send(NewNotification::NewClient((peer, sender)))
        .await
        .expect("to be able to send to the channel");

    info!("New WebSocket connection: {}", peer);
    while let Some(msg) = receiver.next().await {
        let msg = msg?;
        if msg.is_text() || msg.is_binary() {
            // let response = sender.send(Message::Text(msg.to_string())).await;
            new_server_request_data_tx
                .send(NewNotification::NewMessage((peer, msg.to_string())))
                .await
                .expect("to be able to send to the channel ");
        }
    }

    Ok(())
}

async fn run() {
    env_logger::init();
    let (new_server_notification_tx, new_server_notification_rx) = unbounded::<NewNotification>();

    let addr = "127.0.0.1:8080";
    let listener = TcpListener::bind(&addr).await.expect("Can't listen");
    info!("Listening on: {}", addr);

    async_std::task::spawn(listen_new_messages(new_server_notification_rx));

    while let Ok((stream, _)) = listener.accept().await {
        let peer = stream
            .peer_addr()
            .expect("connected streams should have a peer address");
        info!("Peer address: {}", peer);

        async_std::task::spawn(accept_connection(
            peer,
            stream,
            new_server_notification_tx.clone(),
        ));
    }
}

async fn listen_new_messages(new_server_request_data_rx: Receiver<NewNotification>) {
    let mut connected_clients: Vec<(SocketAddr, SplitSink<WebSocketStream<TcpStream>, Message>)> =
        vec![];
    loop {
        if let Ok(result) = new_server_request_data_rx.recv().await {
            match result {
                NewNotification::NewClient(new_client) => connected_clients.push(new_client),
                NewNotification::NewMessage((peer, new_msg)) => {
                    for (current_client_peer, client_write_stream) in &mut connected_clients {
                        if &peer != current_client_peer {
                            client_write_stream
                                .send(tungstenite::Message::Text(new_msg.clone()))
                                .await
                                .expect("to be able to send a message")
                        }
                    }
                }
            }
        }
    }
}
fn main() {
    async_std::task::block_on(run());
}
