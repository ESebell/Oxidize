mod app;
mod types;
mod storage;
mod supabase;
mod stats;

use wasm_bindgen::prelude::*;
use leptos::*;

#[wasm_bindgen(start)]
pub fn main() {
    console_error_panic_hook::set_once();
    
    // Sync from Supabase in background when app starts
    supabase::sync_from_cloud();
    
    mount_to_body(app::App);
}
