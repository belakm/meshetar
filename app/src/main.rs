mod store;
mod store_models;

use chrono::prelude::*;
use gloo_timers::future::TimeoutFuture;
use store::Store;
use sycamore::futures::spawn_local_scoped;
use sycamore::prelude::*;

fn main() {
    sycamore::render(|cx| {
        let store = Store::new();
        let store: &Signal<Store> = create_signal(cx, store);
        /*
         * spawn_local_scoped(cx, async move {
            loop {
                let utc: String = Utc::now().format("%d. %m %Y %H:%M:%S").to_string();
                TimeoutFuture::new(10000).await;
                state.set(utc);
            }
        });*/
        view! { cx,
            App(store=store)
        }
    });
}

#[derive(Prop)]
struct AppProps<'a> {
    store: &'a ReadSignal<Store>,
}

#[component]
fn App<'a, G: Html>(cx: Scope<'a>, props: AppProps<'a>) -> View<G> {
    let store = props.store.get();
    store.
    view! {cx,
        header(class="container") {
            h1 {
                span(class="title-icon") {
                    "🦀"
                }
                " BIG IRON"
            }
        }
        main(class="container") {
            article {
                h2 {
                    "Status"
                }
                p {
                    "Server: " + store
                }
            }
            article {
                h2 {
                    "Portfolio"
                }
            }
            article {
                h2 {
                    "Trading view"
                }
            }
            // img(style="width: 100%;", src=format!("http://localhost:8000/plot/account_balance_history?timestamp={}", props.state.get()))
        }
    }
}
