use crate::store::{local_storage_get, local_storage_set, SqliteWasmStore, Store};
use leptos::prelude::*;
use shared::models::Database;
use std::sync::Arc;

const EXTRA_ITEMS_KEY: &str = "menu-extra-items-v1";

/// Every screen the app can show. Navigation is signal-driven rather than
/// URL-driven, which keeps the GitHub Pages deploy a plain static bundle with
/// no SPA fallback rewrites to get wrong.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum View {
    /// The primary screen: generate a menu, then jump to its grocery list.
    Home,
    /// The generated plan's grocery list. Only reachable once a plan exists.
    Grocery,
    Recipes,
    /// Drill-down into one base's member ingredients.
    Base(i64),
    Ingredients,
    /// JSON export/import, tucked away in the settings overflow.
    Data,
}

/// Shared app state, provided via Leptos context to every page.
#[derive(Clone)]
pub struct AppState {
    pub store: Arc<SqliteWasmStore>,
    pub db: RwSignal<Database>,
    /// Which screen is currently shown.
    pub view: RwSignal<View>,
    /// Toggle for the grocery list: expand bases into member ingredients
    /// (default) or show them as single line items.
    pub expand_bases: RwSignal<bool>,
    pub error: RwSignal<Option<String>>,
    /// The most recently generated menu plan (recipe ids, one per day),
    /// consumed by the Grocery List page.
    pub current_plan: RwSignal<Vec<i64>>,
    /// Ad-hoc grocery items typed by hand (milk, bin bags) that aren't
    /// derived from any recipe. Persisted to `localStorage` so they survive a
    /// reload alongside the database snapshot.
    pub extra_items: RwSignal<Vec<String>>,
}

impl AppState {
    pub fn new(store: SqliteWasmStore, initial: Database) -> Self {
        let extra_items = local_storage_get(EXTRA_ITEMS_KEY)
            .and_then(|json| serde_json::from_str::<Vec<String>>(&json).ok())
            .unwrap_or_default();
        Self {
            store: Arc::new(store),
            db: RwSignal::new(initial),
            view: RwSignal::new(View::Home),
            expand_bases: RwSignal::new(true),
            error: RwSignal::new(None),
            current_plan: RwSignal::new(Vec::new()),
            extra_items: RwSignal::new(extra_items),
        }
    }

    /// Navigate to a screen. The grocery list is gated behind having a plan,
    /// so it can never be reached before a menu has been generated.
    pub fn go(&self, view: View) {
        if view == View::Grocery && self.current_plan.get_untracked().is_empty() {
            self.set_error("Generate a menu first to build a grocery list.");
            return;
        }
        self.view.set(view);
    }

    /// Replace the ad-hoc grocery items and persist them.
    pub fn set_extra_items(&self, items: Vec<String>) {
        if let Ok(json) = serde_json::to_string(&items) {
            local_storage_set(EXTRA_ITEMS_KEY, &json);
        }
        self.extra_items.set(items);
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
