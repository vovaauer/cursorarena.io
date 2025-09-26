use dotenv::dotenv;
use futures_util::{SinkExt, StreamExt};
use log::{info, warn};
use native_tls::{Identity, TlsAcceptor};
use std::{
    collections::HashMap,
    env,
    fs::File,
    io::Read,
    net::SocketAddr,
    sync::{
        atomic::{AtomicU32, Ordering},
        Arc,
    },
    time::Duration,
};
use tokio::{
    net::{TcpListener, TcpStream},
    sync::Mutex,
    time::interval,
};
use tokio_native_tls::TlsStream;
use tokio_tungstenite::{
    accept_async,
    tungstenite::protocol::Message,
    WebSocketStream,
};
use game_logic::{Game, PlayerInput, PlayerId, GameState};
use serde::Serialize;
use tokio_native_tls::TlsAcceptor as TokioTlsAcceptor;


type PeerMap = Arc<Mutex<HashMap<SocketAddr, futures_util::stream::SplitSink<WebSocketStream<TlsStream<TcpStream>>, Message>>>>;
type InputQueue = Arc<Mutex<Vec<(PlayerId, PlayerInput)>>>;

#[derive(Serialize)]
#[serde(tag = "type")]
enum ServerMessage<'a> {
    Welcome { id: PlayerId },
    GameState(&'a GameState),
}

#[tokio::main]
async fn main() {
    dotenv().ok();
    env_logger::init();

    let addr = "0.0.0.0:8088";
    let cert_path = env::var("CERT_PATH").expect("CERT_PATH must be set");
    let cert_pass = env::var("CERT_PASS").expect("CERT_PASS must be set");

    let mut cert_file = File::open(&cert_path).expect("cannot open certificate");
    let mut cert_buf = Vec::new();
    cert_file.read_to_end(&mut cert_buf).expect("cannot read certificate");
    let identity = Identity::from_pkcs12(&cert_buf, &cert_pass).expect("cannot create identity");
    let tls_acceptor = Arc::new(TokioTlsAcceptor::from(
        TlsAcceptor::builder(identity).build().expect("cannot create acceptor"),
    ));

    let listener = TcpListener::bind(&addr).await.expect("Failed to bind");
    info!("Listening on: wss://{}", addr);

    let peer_map = PeerMap::new(Mutex::new(HashMap::new()));
    let game = Arc::new(Mutex::new(Game::new(None)));
    let player_id_counter = Arc::new(AtomicU32::new(1));
    let input_queue = InputQueue::new(Mutex::new(Vec::new()));

    // Spawn the game loop
    tokio::spawn(game_loop(peer_map.clone(), game.clone(), input_queue.clone()));

    while let Ok((stream, addr)) = listener.accept().await {
        let player_id = player_id_counter.fetch_add(1, Ordering::SeqCst);
        let acceptor = tls_acceptor.clone();
        tokio::spawn(handle_connection(acceptor, peer_map.clone(), game.clone(), input_queue.clone(), stream, addr, player_id));
    }
}

async fn game_loop(peer_map: PeerMap, game: Arc<Mutex<Game>>, input_queue: InputQueue) {
    let mut interval = interval(Duration::from_millis(1000 / 60)); // 60 FPS
    loop {
        interval.tick().await;

        let mut inputs = input_queue.lock().await;
        let mut game = game.lock().await;

        for (player_id, input) in inputs.drain(..) {
            game.apply_input(player_id, input);
        }

        game.tick();

        let game_state = game.get_game_state();
        let game_state_msg = ServerMessage::GameState(&game_state);
        let game_state_json = serde_json::to_string(&game_state_msg).unwrap();

        let mut peers = peer_map.lock().await;
        for (addr, writer) in peers.iter_mut() {
            if let Err(e) = writer.send(Message::Text(game_state_json.clone())).await {
                warn!("Failed to send game state to {}: {}. Peer will be removed.", addr, e);
            }
        }
    }
}

async fn handle_connection(
    tls_acceptor: Arc<TokioTlsAcceptor>,
    peer_map: PeerMap,
    game: Arc<Mutex<Game>>,
    input_queue: InputQueue,
    raw_stream: TcpStream,
    addr: SocketAddr,
    player_id: PlayerId,
) {
    info!("Incoming TCP connection from: {} with player_id: {}", addr, player_id);

    let tls_stream = match tls_acceptor.accept(raw_stream).await {
        Ok(s) => s,
        Err(e) => {
            warn!("Failed to perform TLS handshake with {}: {}", addr, e);
            return;
        }
    };

    let ws_stream = match accept_async(tls_stream).await {
        Ok(ws) => ws,
        Err(e) => {
            warn!("Failed to accept websocket connection from {}: {}", addr, e);
            return;
        }
    };
    info!("WebSocket connection established: {}", addr);

    let (mut write, mut read) = ws_stream.split();

    let welcome_msg = ServerMessage::Welcome { id: player_id };
    let welcome_json = serde_json::to_string(&welcome_msg).unwrap();
    if let Err(e) = write.send(Message::Text(welcome_json)).await {
        warn!("Failed to send welcome message to {}: {}", addr, e);
        return;
    }

    peer_map.lock().await.insert(addr, write);
    game.lock().await.add_player(player_id);

    while let Some(Ok(msg)) = read.next().await {
        if let Message::Text(text) = msg {
            match serde_json::from_str::<PlayerInput>(&text) {
                Ok(input) => {
                    input_queue.lock().await.push((player_id, input));
                }
                Err(e) => {
                    warn!("Failed to deserialize input from {}: {}", addr, e);
                }
            }
        }
    }

    info!("{} disconnected", addr);
    peer_map.lock().await.remove(&addr);
    game.lock().await.remove_player(player_id);
}