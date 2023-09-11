use cfg_if::cfg_if;
use http::status::StatusCode;
use leptos::*;

#[cfg(feature = "ssr")]
use leptos_axum::ResponseOptions;

#[component]
pub fn DisplayError(
    cx: Scope,
    error: &'static str,
    #[prop(optional, default = StatusCode::INTERNAL_SERVER_ERROR)]
    #[allow(unused_variables)]
    status_code: StatusCode
) -> impl IntoView {
    cfg_if! { if #[cfg(feature="ssr")] {
        let response = use_context::<ResponseOptions>(cx);
        if let Some(response) = response {
            response.set_status(status_code);
        }
    }}

    view! { cx,
        <h1>Error</h1>
        <p>{error}</p>
    }
}
