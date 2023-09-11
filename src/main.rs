use cfg_if::cfg_if;

cfg_if! {
    if #[cfg(feature = "ssr")] {
        //
        //
        //

        use axum::{
            response::{Response, IntoResponse},
            routing::get,
            extract::{FromRef, Path, State, RawQuery},
            http::{Request, header::HeaderMap},
            body::Body as AxumBody,
            Router
        };
        use clap::Parser;
        use leptos::*;
        use leptos_axum::{generate_route_list, handle_server_fns_with_context, LeptosRoutes};
        use player::app::*;
        use player::files::file_handler;
        use player::player::{Instruction, MediaRoot, PlayerState};
        use std::sync::{Arc, RwLock};
        use tower::ServiceExt;
        use tower_http::services::ServeDir;

        #[derive(Clone, Parser)]
        struct Args {
            #[arg(long = "root", default_value = "dev-media")]
            media_root: String
        }
        
        #[derive(Clone, FromRef)]
        struct AppState {
            args: Args,
            leptos: LeptosOptions,
            player: Arc<RwLock<PlayerState>>,
            instruction: Arc<RwLock<Option<Instruction>>>
        }
        
        #[tokio::main]
        async fn main() {
            simple_logger::init_with_level(log::Level::Warn).expect("couldn't initialize logging");

            let args = Args::parse();
        
            let conf = get_configuration(None).await.unwrap();
            let leptos_options = conf.leptos_options;
            let addr = leptos_options.site_addr;
            let routes = generate_route_list(|cx| view! { cx, <App/> }).await;
            let media_server =  ServeDir::new(String::from(&args.media_root));
        
            let app_state = AppState {
                args,
                leptos: leptos_options,
                player: Arc::new(RwLock::new(PlayerState::Idle)),
                instruction: Arc::new(RwLock::new(None))
            };
        
            let app = Router::new()
                .route("/api/*fn_name", get(server_fn_handler).post(server_fn_handler))
                .leptos_routes_with_handler(routes, get(leptos_routes_handler))
                .nest_service(
                    "/play",
                    get(move |request| media_server.oneshot(request))
                )
                .fallback(file_handler)
                .with_state(app_state);
        
            axum::Server::bind(&addr)
                .serve(app.into_make_service())
                .await
                .unwrap();
        }
        
        async fn server_fn_handler(
            State(app_state): State<AppState>,
            path: Path<String>,
            headers: HeaderMap,
            raw_query: RawQuery,
            request: Request<AxumBody>
        ) -> impl IntoResponse {
            handle_server_fns_with_context(
                path,
                headers,
                raw_query,
                move |cx| {
                    provide_context(cx, MediaRoot(app_state.args.media_root.clone()));
                    provide_context(cx, app_state.player.clone());
                    provide_context(cx, app_state.instruction.clone());
                },
                request
            ).await
        }
        
        async fn leptos_routes_handler(
            State(app_state): State<AppState>,
            req: Request<AxumBody>
        ) -> Response {
            let handler = leptos_axum::render_app_to_stream_with_context(
                app_state.leptos.clone(),
                move |cx| {
                    provide_context(cx, MediaRoot(app_state.args.media_root.clone()));
                    provide_context(cx, app_state.player.clone());
                    provide_context(cx, app_state.instruction.clone());
                },
                |cx| view! { cx, <App/> }
            );
        
            handler(req).await.into_response()
        }

        //
        //
        //
    } else {
        fn main() {}
    }
}
