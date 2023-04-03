use async_std::channel::{unbounded, Receiver, Sender};
use async_std::sync::{Arc, Mutex};

use async_std::net::{SocketAddr, TcpListener, TcpStream};
use async_tungstenite::{
    accept_async,
    tungstenite::{Error, Result},
    WebSocketStream,
};
use futures::{prelude::*, stream::SplitSink};
use log::*;
use tungstenite::Message;

async fn accept_connection(
    peer: SocketAddr,
    stream: TcpStream,
    new_server_request_data_tx: Sender<String>,
    web_socket_shared_container: Arc<Mutex<WebSocketSharedContainer>>,
) {
    if let Err(e) = handle_connection(
        peer,
        stream,
        new_server_request_data_tx,
        web_socket_shared_container,
    )
    .await
    {
        match e {
            Error::ConnectionClosed | Error::Protocol(_) | Error::Utf8 => (),
            err => error!("Error processing connection: {}", err),
        }
    }
}

async fn handle_connection(
    peer: SocketAddr,
    stream: TcpStream,
    new_server_request_data_tx: Sender<String>,
    web_socket_shared_container: Arc<Mutex<WebSocketSharedContainer>>,
) -> Result<()> {
    let ws_stream = accept_async(stream).await.expect("Failed to accept");
    let (mut sender, mut receiver) = ws_stream.split();
    web_socket_shared_container
        .lock()
        .await
        .web_socket_shared_container(sender);

    info!("New WebSocket connection: {}", peer);
    while let Some(msg) = receiver.next().await {
        let msg = msg?;
        if msg.is_text() || msg.is_binary() {
            // let response = sender.send(Message::Text(msg.to_string())).await;
            new_server_request_data_tx.send(msg.to_string()).await;
        }
    }

    Ok(())
}

async fn run() {
    env_logger::init();
    let web_socket_shared_container = Arc::new(Mutex::new(WebSocketSharedContainer {
        web_socket_stream: None,
    }));

    let (new_server_request_data_tx, new_server_request_data_rx) = unbounded::<String>();

    let addr = "127.0.0.1:8080";
    let listener = TcpListener::bind(&addr).await.expect("Can't listen");
    info!("Listening on: {}", addr);

    let new_server_request_data_rx_arc = Arc::new(Mutex::new(new_server_request_data_rx));
    async_std::task::spawn(listen_new_messages(
        Arc::clone(&new_server_request_data_rx_arc),
        Arc::clone(&web_socket_shared_container),
    ));

    while let Ok((stream, _)) = listener.accept().await {
        let peer = stream
            .peer_addr()
            .expect("connected streams should have a peer address");
        info!("Peer address: {}", peer);

        async_std::task::spawn(accept_connection(
            peer,
            stream,
            new_server_request_data_tx.clone(),
            Arc::clone(&web_socket_shared_container),
        ));
    }
}

async fn listen_new_messages(
    new_server_request_data_rx_arc: Arc<Mutex<Receiver<String>>>,
    web_socket_shared_container: Arc<Mutex<WebSocketSharedContainer>>,
) {
    loop {
        let locked = new_server_request_data_rx_arc.lock().await;
        if let Ok(result) = locked.recv().await {
            let mut locked_muy = web_socket_shared_container.lock().await;
            locked_muy.send_message(result).await;
        }
    }
}
fn main() {
    async_std::task::block_on(run());
}

struct WebSocketSharedContainer {
    pub web_socket_stream: Option<SplitSink<WebSocketStream<TcpStream>, Message>>,
}

impl WebSocketSharedContainer {
    pub fn web_socket_shared_container(
        &mut self,
        websocket: SplitSink<WebSocketStream<TcpStream>, Message>,
    ) {
        self.web_socket_stream = Some(websocket);
    }
    pub async fn send_message(&mut self, message: String) {
        let _ = self
            .web_socket_stream
            .as_mut()
            .unwrap()
            .send(tungstenite::Message::Text(message))
            .await;
    }
}
