mod browse;
mod fns;
mod media;

use crate::error::DisplayError;
use crate::loading::Loading;
use crate::player::PlayerState;
use crate::remote::browse::Browse;
use crate::remote::fns::*;
use crate::remote::media::Media;
use leptos::*;
use leptos_server_signal::create_server_signal;

#[component]
pub fn Remote(cx: Scope) -> impl IntoView {
    let player_state = create_server_signal::<PlayerState>(cx, "player_state");

    // Split out to allow controls to minimize control re-renders

    let is_playing = create_memo(
        cx,
        move |_| {
            match player_state.get() {
                PlayerState::Media(_) => true,
                _ => false
            }
        }
    );

    let media_state = create_memo(
        cx,
        move |_| {
            match player_state.get() {
                PlayerState::Media(state) => Some(state),
                _ => None
            }
        }
    );

    let media = create_resource(
        cx,
        || (),
        move |_| list_media(cx)
    );

    view! { cx,
        <div class="container-xl">
            <Transition fallback=|| ()>
                {move || {
                    if is_playing.get() {
                        view! { cx, <Media state=media_state/> }.into_view(cx)
                    } else {
                        match media.read(cx) {
                            None => {
                                view! { cx, <Loading/> }.into_view(cx)
                            },
                            Some(Err(_)) => {
                                view! { cx, <DisplayError error="Failed to load media library"/> }.into_view(cx)
                            },
                            Some(Ok(files)) => {
                                view! { cx, <Browse media_files=files/> }.into_view(cx)
                            }
                        }
                    }
                }}
            </Transition>
        </div>
    }
}
