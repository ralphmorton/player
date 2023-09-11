use crate::remote::fns::Instruct;
use crate::player::*;
use leptos::*;

#[component]
pub fn Media(
    cx: Scope,
    state: Memo<Option<MediaState>>
) -> impl IntoView {
    let instruct = create_server_action::<Instruct>(cx);

    let path = move || {
        state.get().map(|state| match state {
            MediaState::Playing(path, _, _) => path,
            MediaState::Paused(path, _, _) => path
        })
    };

    let duration = move || {
        state.get().map(|state| match state {
            MediaState::Playing(_, duration, _) => duration.floor(),
            MediaState::Paused(_, duration, _) => duration.floor()
        })
    };

    let time = move || {
        state.get().map(|state| match state {
            MediaState::Playing(_, _, from) => from.floor(),
            MediaState::Paused(_, _, at) => at.floor()
        })
    };

    let play_from = move |from| {
        if let Some(path) = path() {
            instruct.dispatch(Instruct { i: Instruction::Play(path, from) } )
        }
    };

    view! { cx,
        <div class="card shadow mt-5">
            <div class="card-body">
                <div class="text-center mb-4">
                    <h3>{path}</h3>
                </div>
                <div class="row d-flex justify-content-center mb-4">
                    <div class="col-auto">
                        <div class="row">
                            <div class="col-auto">
                                <button
                                    class="btn btn-primary"
                                    on:click=move |_| {
                                        if let Some(from) = time() {
                                            play_from(from - 60.0);
                                        }
                                    }
                                >
                                    <i class="bx bx-rewind"></i>
                                </button>
                            </div>
                            <div class="col-auto">
                                <button
                                    class="btn btn-primary"
                                    on:click=move |_| {
                                        if let Some(from) = time() {
                                            play_from(from);
                                        }
                                    }
                                >
                                    <i class="bx bx-play"></i>
                                </button>
                            </div>
                            <div class="col-auto">
                                <button
                                    class="btn btn-primary"
                                    on:click=move |_| {
                                        if let Some(path) = path() {
                                            if let Some(at) = time() {
                                                instruct.dispatch(Instruct { i: Instruction::Pause(path, at) } )
                                            }
                                        }
                                    }
                                >
                                    <i class="bx bx-pause"></i>
                                </button>
                            </div>
                            <div class="col-auto">
                                <button
                                    class="btn btn-primary"
                                    on:click=move |_| {
                                        instruct.dispatch(Instruct { i: Instruction::Stop } )
                                    }
                                >
                                    <i class="bx bx-stop"></i>
                                </button>
                            </div>
                            <div class="col-auto">
                                <button
                                    class="btn btn-primary"
                                    on:click=move |_| {
                                        if let Some(from) = time() {
                                            play_from(from + 60.0);
                                        }
                                    }
                                >
                                    <i class="bx bx-fast-forward"></i>
                                </button>
                            </div>
                        </div>
                    </div>
                </div>
                <progress class="w-100" max=duration value=time/>
            </div>
        </div>
    }
}
