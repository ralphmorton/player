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

#[component]
pub fn Remote(cx: Scope) -> impl IntoView {
    #[allow(unused_variables)]
    let (heartbeat, tick) = create_signal(cx, 0);

    let player_state_res = create_resource(
        cx,
        heartbeat,
        move |_| fetch_player_state(cx)
    );

    let player_state = create_memo(
        cx,
        move |_| player_state_res.read(cx)
    );

    // Split out to allow controls to minimize control re-renders

    let is_playing = create_memo(
        cx,
        move |_| {
            match player_state.get() {
                Some(Ok(PlayerState::Media(_))) => true,
                _ => false
            }
        }
    );

    let media_state = create_memo(
        cx,
        move |_| {
            match player_state.get() {
                Some(Ok(PlayerState::Media(state))) => Some(state),
                _ => None
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
                        Some(view! { cx, <Media state=media_state/> }.into_view(cx))
                    } else {
                        player_state.get().map(|result| {
                            match result {
                                Err(_) => view! { cx, <DisplayError error="Failed to fetch player state"/> },
                                Ok(_) => match media.read(cx) {
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
                        })
                    }
                }}
            </Transition>
        </div>
    }
}
