pub mod binance_client;
pub mod formatting;
pub mod load_config;
pub mod serde_utils;

pub fn remove_vec_items_from_start<T>(mut vec: Vec<T>, n: usize) -> Vec<T> {
    vec.drain(0..n);
    vec
}
