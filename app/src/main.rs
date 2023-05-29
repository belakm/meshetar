mod app;
mod routes;
mod store;
mod store_models;

use app::App;

fn main() {
    sycamore::render(App)
}
