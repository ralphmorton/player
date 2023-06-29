use axum::{Router, Server};
use axum::body::Body;
use axum::extract::{Json, State};
use axum::http::Request;
use axum::routing::{get, post};
use clap::Parser;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, RwLock};
use tower::ServiceExt;
use tower_http::services::{ServeDir, ServeFile};

#[derive(Parser)]
struct Args {
    #[arg(long = "play")]
    play_root: String
}

#[derive(Clone, Deserialize, Serialize)]
#[serde(tag = "tag")]
enum Instruction {
    Idle,
    Play { path: String, from: Option<f32>, behaviour: Behaviour }
}

#[derive(Clone, Deserialize, Serialize)]
#[serde(tag = "tag")]
enum Behaviour {
    Playing,
    Paused
}

#[derive(Clone, Deserialize, Serialize)]
struct PlayerState {
    path: String,
    duration: f32,
    time: f32
}

#[derive(Clone)]
struct AppState {
    args: Arc<Args>,
    pending_instruction: Arc<RwLock<Option<Instruction>>>,
    player_state: Arc<RwLock<Option<PlayerState>>>
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    let play = ServeDir::new(String::from(&args.play_root));
    let spa = ServeDir::new("static").not_found_service(ServeFile::new("static/index.html"));
    let pending_instruction = Arc::new(RwLock::new(None));
    let player_state = Arc::new(RwLock::new(None));
    let state = AppState { args: Arc::new(args), pending_instruction, player_state };

    let app = Router::new()
        .route("/ls", get(ls))
        .route("/state", get(poll_player_state))
        .route("/instruction", post(instruct))
        .route("/update", post(update))
        .nest_service(
            "/play",
            get(move |request: Request<Body>| async {
                play.oneshot(request).await
            })
        )
        .nest_service(
            "/assets",
            get(move |request: Request<Body>| async {
                spa.oneshot(request).await
            })
        )
        .nest_service("/tv", ServeFile::new("static/index.html"))
        .fallback_service(ServeFile::new("static/remote.html"))
        .with_state(state);

    Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn ls(State(state): State<AppState>) -> Json<Vec<String>> {
    let path = std::path::Path::new(&state.args.play_root);

    let entries = walkdir::WalkDir::new(&path)
        .into_iter()
        .filter_map(|file| file.ok())
        .filter(|e| e.metadata().unwrap().is_file())
        .map(|e| format!("/play/{}", e.path().strip_prefix(path).unwrap().display()))
        .collect();

    Json(entries)
}

async fn poll_player_state(State(state): State<AppState>) -> Json<Option<PlayerState>> {
    Json(state.player_state.read().unwrap().clone())
}

async fn instruct(State(state): State<AppState>, Json(i): Json<Instruction>) {
    let mut pending_instruction = state.pending_instruction.write().unwrap();
    *pending_instruction = Some(i);
}

async fn update(State(state): State<AppState>, Json(p): Json<Option<PlayerState>>) -> Json<Option<Instruction>> {
    let mut player_state = state.player_state.write().unwrap();
    *player_state = p.clone();

    let mut pending_instruction = state.pending_instruction.write().unwrap();
    let i = pending_instruction.clone();
    *pending_instruction = None;

    Json(i)
}
