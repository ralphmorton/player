use crate::player::*;
use leptos::*;
#[allow(unused_imports)]
use std::sync::{Arc, RwLock};

#[server(ListMedia, "/api")]
pub async fn list_media(cx: Scope) -> Result<Vec<String>, ServerFnError> {
    let MediaRoot(media_root) = expect_context::<MediaRoot>(cx);
    let root = std::path::Path::new(&media_root);

    let entries = walkdir::WalkDir::new(&root)
        .into_iter()
        .filter_map(|file| file.ok())
        .filter(|e| e.metadata().unwrap().is_file())
        .map(|e| format!("{}", e.path().strip_prefix(root).unwrap().display()))
        .collect();

    Ok(entries)
}

#[server(FetchPlayerState, "/api")]
pub async fn fetch_player_state(cx: Scope) -> Result<PlayerState, ServerFnError> {
    let player_state = expect_context::<Arc<RwLock<PlayerState>>>(cx).read()?.clone();

    Ok(player_state)
}

#[server(Instruct, "/api")]
pub async fn instruct(cx: Scope, i: Instruction) -> Result<(), ServerFnError> {
    let instruction = expect_context::<Arc<RwLock<Option<Instruction>>>>(cx);

    let mut write = instruction.write().unwrap();
    *write = Some(i);

    Ok(())
}
