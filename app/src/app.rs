use crate::routes::{self, change_interval, change_pair, fetch_last_kline_time, get_status};
use crate::store::Store;
use crate::store_models::{Interval, Meshetar, Pair, Status};
use crate::utils::{console_log, readable_date};
use gloo_timers::future::TimeoutFuture;
use sycamore::futures::spawn_local_scoped;
use sycamore::prelude::{Html, Scope};
use sycamore::reactive::{create_effect, create_rc_signal, create_signal, provide_context};
use sycamore::view::View;
use sycamore::{component, view};

fn sync_store(store: &Store, meshetar: Meshetar) {
    store.server_state.set(meshetar.status);
    store.pair.set(meshetar.pair.to_string());
    store.interval.set(meshetar.interval.to_string());
    store.mode.set(meshetar.status.to_string());
}

#[component]
pub fn App<G: Html>(cx: Scope) -> View<G> {
    let store = Store {
        message: create_rc_signal(String::from("")),
        mode: create_rc_signal(String::from("Idle")),
        pair: create_rc_signal(String::from("BTCUSDT")),
        interval: create_rc_signal(String::from("Minutes1")),
        server_state: create_rc_signal(Status::Idle),
        last_kline_time: create_rc_signal(String::from("0")),
    };
    let store = provide_context(cx, store);

    spawn_local_scoped(cx, async move {
        loop {
            match get_status().await {
                Ok(meshetar) => sync_store(store, meshetar),
                _ => (),
            }
            match fetch_last_kline_time().await {
                Ok(last_kline_time) => {
                    store.last_kline_time.set(last_kline_time);
                }
                _ => (),
            }
            TimeoutFuture::new(3000).await;
        }
    });
    let handle_change_pair = move |_| {
        let pair = store.pair.get();
        let pair = pair.parse::<Pair>();
        match pair {
            Ok(pair) => {
                spawn_local_scoped(cx, async move {
                    match change_pair(pair).await {
                        Err(e) => console_log(&e.to_string()),
                        Ok(pair) => {
                            store.pair.set(pair.to_string());
                        }
                    }
                });
            }
            Err(e) => console_log(&e.to_string()),
        }
    };
    let handle_change_interval = move |_| {
        let interval = store.interval.get();
        let interval = interval.parse::<Interval>();
        match interval {
            Ok(interval) => {
                spawn_local_scoped(cx, async move {
                    match change_interval(interval).await {
                        Err(e) => console_log(&e.to_string()),
                        Ok(interval) => {
                            store.interval.set(interval.to_string());
                        }
                    }
                });
            }
            Err(e) => console_log(&e.to_string()),
        }
    };
    let fetch_history = move |_| {
        spawn_local_scoped(cx, async move {
            match routes::fetch_history().await {
                Ok(meshetar) => sync_store(store, meshetar),
                _ => (),
            }
        });
    };
    let run = move |_| {
        spawn_local_scoped(cx, async move {
            match routes::run().await {
                Ok(meshetar) => sync_store(store, meshetar),
                _ => (),
            }
        });
    };
    let stop = move |_| {
        spawn_local_scoped(cx, async move {
            match routes::stop().await {
                Ok(meshetar) => sync_store(store, meshetar),
                _ => (),
            }
        });
    };
    let clear_history = move |_| {
        spawn_local_scoped(cx, async move {
            match routes::clear_history().await {
                Ok(meshetar) => sync_store(store, meshetar),
                _ => (),
            }
        });
    };
    let create_model = move |_| {
        spawn_local_scoped(cx, async move {
            match routes::create_model().await {
                Ok(meshetar) => sync_store(store, meshetar),
                _ => (),
            }
        });
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
                p {
                    strong {"Current status: "} (store.mode)
                }
                p {
                    strong {"Last kline time: "} (readable_date(&store.last_kline_time.get()))
                }
                div(class="grid") {
                    select(bind:value=store.pair, on:change=handle_change_pair) {
                        option {
                            "BTCUSDT"
                        }
                        option {
                            "ETHBTC"
                        }
                    }
                    select(bind:value=store.interval, on:change=handle_change_interval) {
                        option {
                            "Minutes1"
                        }
                        option {
                            "Minutes3"
                        }
                    }
                }
                div(class="grid") {
                    button(class="secondary", on:click=fetch_history, disabled=*store.server_state.get() != Status::Idle) {
                        "📥 Fetch history"
                    }
                    button(class="secondary", on:click=clear_history, disabled=*store.server_state.get() != Status::Idle) {
                        "🧹 Clear history"
                    }
                }
                div {
                    hr(style="margin: calc(2*var(--spacing)) 0; margin-top: var(--spacing);") {}
                }
                div(class="grid") {
                    button(on:click=run, disabled=*store.server_state.get() != Status::Idle) {
                        "▶️ START"
                    }
                    button(on:click=stop, disabled=*store.server_state.get() == Status::Idle) {
                        "⏹︎ STOP"
                    }
                }
                div(class="grid") {

                }
            }
            // img(style="width: 100%;", src=format!("http://localhost:8000/plot/account_balance_history?timestamp={}", props.state.get()))
        }
    }
}
