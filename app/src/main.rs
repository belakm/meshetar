// mod store;
// mod store_models;
use gloo_timers::future::TimeoutFuture;
// use store::Store;
use sycamore::futures::spawn_local_scoped;
use sycamore::prelude::*;

async fn get_status() -> Result<String, Box<dyn std::error::Error>> {
    let resp: String = reqwest::get("http://localhost:8000/status")
        .await?
        .text()
        .await?;
    Ok(resp)
}

fn main() {
    sycamore::render(|cx| {
        let store = String::new();
        let store: &Signal<String> = create_signal(cx, store);
        spawn_local_scoped(cx, async move {
            loop {
                let status = get_status().await;
                match status {
                    Ok(s) => {
                        store.set(s);
                        println!("{:?}", store.get())
                    }
                    _ => (),
                }
                TimeoutFuture::new(3000).await;
            }
        });
        view! { cx,
            App(store=store)
        }
    });
}

#[derive(Prop)]
struct AppProps<'a> {
    store: &'a ReadSignal<String>,
}

#[component]
fn App<'a, G: Html>(cx: Scope<'a>, props: AppProps<'a>) -> View<G> {
    let store = props.store.get();
    println!("{:?}", store);
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
                   (store)
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
