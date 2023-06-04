use crate::routes::{self, change_interval, change_pair, fetch_last_kline_time, get_status};
use crate::store::Store;
use crate::store_models::{Interval, Pair, Status};
use crate::utils::{console_log, readable_date};
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
        last_kline_time: create_rc_signal(String::from("0")),
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
            match get_status().await {
                Ok(meshetar) => {
                    store.server_state.set(meshetar.status);
                    store.pair.set(meshetar.pair.to_string());
                    store.interval.set(meshetar.interval.to_string());
                    store.mode.set(meshetar.status.to_string());
                }
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
                    p {
                        strong {"Current status: "} (store.mode)
                    }
                    p {
                        strong {"Last kline time: "} (readable_date(&store.last_kline_time.get()))
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
