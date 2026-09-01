use crate::store::{SqliteWasmStore, Store};
use leptos::prelude::*;
use shared::models::Database;
use std::sync::Arc;

/// Shared app state, provided via Leptos context to every page.
#[derive(Clone)]
pub struct AppState {
    pub store: Arc<SqliteWasmStore>,
    pub db: RwSignal<Database>,
    /// Toggle for the grocery list: expand clusters into member ingredients
    /// (default) or show them as single line items.
    pub expand_clusters: RwSignal<bool>,
    pub error: RwSignal<Option<String>>,
    /// The most recently generated menu plan (recipe ids, one per day),
    /// consumed by the Grocery List page.
    pub current_plan: RwSignal<Vec<i64>>,
}

impl AppState {
    pub fn new(store: SqliteWasmStore, initial: Database) -> Self {
        Self {
            store: Arc::new(store),
            db: RwSignal::new(initial),
            expand_clusters: RwSignal::new(true),
            error: RwSignal::new(None),
            current_plan: RwSignal::new(Vec::new()),
        }
    }

    /// Reload the whole in-memory database snapshot from the store.
    pub fn reload(&self) {
        let store = self.store.clone();
        let db = self.db;
        let error = self.error;
        leptos::task::spawn_local(async move {
            match store.load_database().await {
                Ok(fresh) => db.set(fresh),
                Err(e) => error.set(Some(e.to_string())),
            }
        });
    }

    pub fn set_error(&self, msg: impl Into<String>) {
        self.error.set(Some(msg.into()));
    }
}

pub fn use_app_state() -> AppState {
    use_context::<AppState>().expect("AppState not provided")
}
