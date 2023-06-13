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
    Stop,
    Play { path: String },
    Pause,
    Resume
}

#[derive(Clone)]
struct AppState {
    args: Arc<Args>,
    instruction: Arc<RwLock<Instruction>>
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    let play = ServeDir::new(String::from(&args.play_root));
    let spa = ServeDir::new("static").not_found_service(ServeFile::new("static/index.html"));
    let instruction = Arc::new(RwLock::new(Instruction::Stop));
    let state = AppState { args: Arc::new(args), instruction };

    let app = Router::new()
        .route("/ls", get(ls))
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
        .map(|e| format!("{}", e.path().strip_prefix(path).unwrap().display()))
        .collect();

    Json(entries)
}

async fn instruct(State(state): State<AppState>, Json(i): Json<Instruction>) {
    let mut instruction = state.instruction.write().unwrap();

    match i {
        Instruction::Play{ path } => {
            let path = format!("/play/{}", path);
            *instruction = Instruction::Play { path };
        },
        _ => {
            *instruction = i;
        }
    }
    
}

async fn update(State(state): State<AppState>) -> Json<Instruction> {
    match state.instruction.read() {
        Ok(instruction) => Json(instruction.clone()),
        Err(_) => Json(Instruction::Stop)
    }
}
