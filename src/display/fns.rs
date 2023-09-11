use crate::player::*;
use leptos::*;
#[allow(unused_imports)]
use std::sync::{Arc, RwLock};

#[server(GetInstruction, "/api")]
pub async fn get_instruction(cx: Scope) -> Result<Option<Instruction>, ServerFnError> {
    let instruction = expect_context::<Arc<RwLock<Option<Instruction>>>>(cx);

    let mut writable = instruction.write().unwrap();
    let pending = writable.clone();
    *writable = None;

    Ok(pending)
}

#[server(SetPlayerState, "/api")]
pub async fn set_player_state(cx: Scope, state: PlayerState) -> Result<(), ServerFnError> {
    let player_state = expect_context::<Arc<RwLock<PlayerState>>>(cx);

    let mut writable = player_state.write().unwrap();
    *writable = state;

    Ok(())
}
