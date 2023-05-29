use crate::routes::{self, get_status};
use crate::store::{self, Store};
use crate::store_models::{Pair, Status};
use gloo_timers::future::TimeoutFuture;
use sycamore::futures::spawn_local_scoped;
use sycamore::prelude::{Html, Scope};
use sycamore::reactive::{create_effect, create_rc_signal, create_signal, provide_context};
use sycamore::view::View;
use sycamore::{component, view};

#[component]
pub fn App<G: Html>(cx: Scope) -> View<G> {
    let is_running = create_signal(cx, false);
    let store = Store {
        message: create_rc_signal(String::from("")),
        mode: create_rc_signal(String::from("Idle")),
        pair: create_rc_signal(String::from("BTCUSDT")),
        interval: create_rc_signal(String::from("Minutes1")),
        server_state: create_rc_signal(Status::Idle),
    };
    let store = provide_context(cx, store);

    // Listener for server state
    create_effect(cx, move || {
        let server_state = *store.server_state.get();
        match server_state {
            Status::Idle => is_running.set(false),
            Status::FetchingHistory => is_running.set(true),
        }
    });

    spawn_local_scoped(cx, async move {
        loop {
            let status = get_status().await;
            match status {
                Ok(meshetar) => {
                    store.server_state.set(meshetar.status);
                    store.pair.set(meshetar.pair.to_string());
                    store.interval.set(meshetar.interval.to_string());
                    store.mode.set(meshetar.status.to_string());
                }
                _ => (),
            }
            TimeoutFuture::new(3000).await;
        }
    });
    let change_pair = |_| {
        let pair = store.pair.get();
        //change_pair_async(pair);
    };
    async fn change_pair_async(pair: String) -> () {
        //change_pair(pair).await;
    }
    let start_operation = |_| {
        routes::start_operation();
    };
    let stop_operation = |_| {
        routes::stop_operation();
    };
    view! {cx,
        header(class="container") {
            h1 {
                span(class="title-icon") {
                    "🫰"
                }
                " MESHETAR"
            }
        }
        main(class="container") {
            article {
                div {
                    select(bind:value=store.pair, on:change=change_pair) {
                        option {
                            "BTCUSDT"
                        }
                        option {
                            "BTCETH"
                        }
                    }
                    select(bind:value=store.interval) {
                        option {
                            "Minutes1"
                        }
                        option {
                            "Minutes3"
                        }
                    }
                    select(bind:value=store.mode) {
                        option {
                            "Idle"
                        }
                        option {
                            "FetchingHistory"
                        }
                    }
                    div(class="grid") {
                        button(on:click=start_operation, disabled=*is_running.get()) {
                            "⏺︎ START"
                        }
                        button(on:click=stop_operation, disabled=!*is_running.get()) {
                            "⏹︎ STOP"
                        }
                    }
                }
            }
            // img(style="width: 100%;", src=format!("http://localhost:8000/plot/account_balance_history?timestamp={}", props.state.get()))
        }
    }
}
