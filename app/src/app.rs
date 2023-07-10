use crate::routes::{
    self, change_interval, change_pair, fetch_last_kline_time, get_status, plot_chart,
};
use crate::store::Store;
use crate::store_models::{Interval, Meshetar, Pair, Status};
use crate::utils::{console_log, get_timestamp, readable_date};
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
pub fn Divider<'a, G: Html>(cx: Scope<'a>) -> View<G> {
    view! { cx,
        div {
            hr(style="margin: calc(2*var(--spacing)) 0; margin-top: var(--spacing);") {}
        }
    }
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

    // For handling chart versioning
    let chart_path = create_signal(cx, String::from(""));

    // For handling states
    let is_normally_disabled = create_signal(cx, *store.server_state.get() != Status::Idle);
    let is_stop_disabled = create_signal(
        cx,
        *store.server_state.get() == Status::Idle || *store.server_state.get() == Status::Stopping,
    );
    let meshetar_state_style = create_signal(cx, "idle".to_string());
    create_effect(cx, || {
        is_normally_disabled.set(*store.server_state.get() != Status::Idle);
        is_stop_disabled.set(
            *store.server_state.get() == Status::Idle
                || *store.server_state.get() == Status::Stopping,
        );
    });
    create_effect(cx, || {
        let class_string = *store.server_state.get();
        let class_string = format!("status-{}", class_string.to_string());
        meshetar_state_style.set(class_string);
    });

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
            match plot_chart().await {
                Ok(path) => chart_path.set(path),
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
    let create_new_model = move |_| {
        spawn_local_scoped(cx, async move {
            match routes::create_new_model().await {
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

    view! {cx,
        header(class=format!("container {}", *meshetar_state_style.get())) {
            h1 {
                span(class="title-icon") {
                    "🫰"
                }
                " MESHETAR"
            }
            div(class="grid") {
                p {
                    strong {"Current status: "}
                    span(class="status-label", aria-busy=*is_normally_disabled.get()) { (store.mode) }
                }
                p {
                    strong {"Last kline time: "} (readable_date(&store.last_kline_time.get()))
                }
            }
        }
        main(class=format!("container {}", *meshetar_state_style.get())) {
            article {
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
                    button(class="secondary", on:click=fetch_history, disabled=*is_normally_disabled.get()) {
                        "📥 Fetch history"
                    }
                    button(class="secondary", on:click=clear_history, disabled=*is_normally_disabled.get()) {
                        "🧹 Clear history"
                    }
                    button(class="secondary", on:click=create_new_model, disabled=*is_normally_disabled.get()) {
                        "🪩 Create new model"
                    }
                }
                div(class="grid") {
                    button(on:click=run, disabled=*is_normally_disabled.get()) {
                        "▶️ START"
                    }
                    button(on:click=stop, disabled=*is_stop_disabled.get()) {
                        "⏹︎ STOP"
                    }
                }
                Divider{}
                div(class="chart-container") {
                    img(src=format!("http://localhost:8000/{}?ver={}", *chart_path.get(), get_timestamp()))
                }
            }
        }
    }
}
