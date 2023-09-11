use leptos::*;

#[component]
pub fn Loading(
    cx: Scope
) -> impl IntoView {
    view! { cx,
      <div class="card shadow mt-5">
        <div class="card-body text-center fs-4">
          "Loading..."
        </div>
      </div>
    }
}
