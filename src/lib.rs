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
    
    // Reset sync status so UI knows to wait for fresh data
    storage::reset_sync_status();
    
    // Sync from Supabase in background when app starts
    supabase::sync_from_cloud();
    
    mount_to_body(app::App);
}
