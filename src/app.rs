use crate::display::Display;
use crate::error::DisplayError;
use crate::remote::Remote;
use leptos::*;
use leptos_meta::*;
use leptos_router::*;

#[component]
pub fn App(cx: Scope) -> impl IntoView {
    provide_meta_context(cx);

    view! {
        cx,
        <Html attributes=AdditionalAttributes::from(vec![("data-bs-theme", "dark")])/>
        <Stylesheet href="https://cdn.jsdelivr.net/npm/bootstrap@5.3.1/dist/css/bootstrap.min.css"/>
        <Stylesheet href="https://unpkg.com/boxicons@2.1.4/css/boxicons.min.css"/>
        <Stylesheet href="/pkg/player.css"/>
        <Script src="https://cdn.jsdelivr.net/npm/bootstrap@5.3.1/dist/js/bootstrap.bundle.min.js"></Script>
        <Title text="Player"/>
        <Body class="vh-100"/>
        <Router fallback=|cx| { view! { cx, <DisplayError error="Not Found" status_code=http::status::StatusCode::NOT_FOUND/> } }>
            <main class="h-100 overflow-hidden">
                <Routes>
                    <Route path="" view=|cx| view! { cx, <Remote/> }/>
                    <Route path="tv" view=|cx| view! { cx, <Display/> }/>
                </Routes>
            </main>
        </Router>
    }
}
