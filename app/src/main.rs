mod app;
mod routes;
mod store;
mod store_models;
mod utils;

use app::App;

fn main() {
    sycamore::render(App)
}
