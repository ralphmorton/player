use serde::{Deserialize, Serialize};

#[derive(Clone)]
pub struct MediaRoot(pub String);

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub enum PlayerState {
    Idle,
    Media(MediaState)
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub enum MediaState {
    Paused(String, f64, f64),
    Playing(String, f64, f64)
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub enum Instruction {
    Stop,
    Play(String, f64),
    Pause(String, f64)
}
