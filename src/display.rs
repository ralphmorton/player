mod fns;

use crate::display::fns::*;
use crate::player::*;
use leptos::*;

static MEDIA_ROOT : &'static str = "/play/";
static VIDEO_STOPPED_SRC : &'static str = "https://video.is.stopped/";

#[component]
pub fn Display(cx: Scope) -> impl IntoView {
    let video = create_node_ref::<leptos::html::Video>(cx);
    let (heartbeat, tick) = create_signal(cx, 0);

    let instruction = create_local_resource(
        cx,
        heartbeat,
        move |_| get_instruction(cx)
    );

    create_effect(
        cx,
        move |_| {
            if let Some(video) = video() {
                if let Some(Ok(Some(instruction))) = instruction.read(cx) {
                    match instruction {
                        Instruction::Stop => {
                            video.set_src(VIDEO_STOPPED_SRC);
                        },
                        Instruction::Play(src, from) => {
                            let url = format!("{}{}", MEDIA_ROOT, src);
                            video.set_src(url.as_str());
                            video.set_current_time(from);
                            let _ = video.play();
                        },
                        Instruction::Pause(src, at) => {
                            let url = format!("{}{}", MEDIA_ROOT, src);
                            video.set_src(url.as_str());
                            video.set_current_time(at);
                            let _ = video.pause();
                        }
                    }
                }
    
                let src = video.src();
                let current_time = video.current_time();
                let paused = video.paused();

                let duration = video.duration();
                let duration = if duration.is_nan() { 0.0 } else { duration };

                let path = if let Ok(url) = url::Url::parse(src.as_str()) {
                    url.path().strip_prefix(MEDIA_ROOT).unwrap_or(url.path()).to_string()
                } else {
                    src.clone()
                };
    
                let state = if src.as_str() == VIDEO_STOPPED_SRC || src.as_str() == "" {
                    PlayerState::Idle
                } else if paused {
                    PlayerState::Media(MediaState::Playing(path, duration, current_time))
                } else {
                    PlayerState::Media(MediaState::Paused(path, duration, current_time))
                };
    
                spawn_local(async move {
                    let _ = set_player_state(cx, state).await;
                })
            }
        }
    );

    #[cfg(not(feature = "ssr"))]
    set_interval(
        move || {
            tick.update(|c| *c += 1);
        },
        std::time::Duration::from_millis(100)
    );

    view! { cx,
        <div class="video-wrapper">
            <video class="video" autoplay=true node_ref=video/>
        </div>
    }
}
