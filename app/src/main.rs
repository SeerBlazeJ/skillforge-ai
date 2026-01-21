#![allow(non_snake_case)]
use dioxus::prelude::*;

mod models;
mod pages;
mod server_functions;
mod utils;

use pages::*;

#[derive(Debug, Clone, Routable, PartialEq)]
#[rustfmt::skip]
enum Route {
    #[route("/")]
    Landing {},
    #[route("/login")]
    Login {},
    #[route("/signup")]
    Signup {},
    #[route("/dashboard")]
    Dashboard {},
    #[route("/profile")]
    Profile {},
    #[route("/roadmap/:id")]
    RoadmapView { id: String },
    #[route("/create-roadmap")]
    CreateRoadmap {},
}

const MAIN_CSS: Asset = asset!("/assets/main.css");
const TAILWIND_CSS: Asset = asset!("/assets/tailwind.css");

fn main() {
    #[cfg(feature = "server")]
    {
        use dioxus::server::IncrementalRendererConfig;

        tracing_subscriber::fmt::init();

        LaunchBuilder::new()
            .with_cfg(server_only! {
                ServeConfig::builder().incremental(IncrementalRendererConfig::default())
            })
            .launch(App);
    }
    #[cfg(not(feature = "server"))]
    {
        dioxus::launch(App);
    }
}

#[component]
fn App() -> Element {
    rsx! {
        document::Link { rel: "stylesheet", href: MAIN_CSS }
        document::Link { rel: "stylesheet", href: TAILWIND_CSS }
        Router::<Route> {}
    }
}
