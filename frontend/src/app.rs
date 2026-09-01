use crate::pages::{clusters, data, grocery, ingredients, menu, recipes};
use crate::state::{use_app_state, AppState};
use crate::store::SqliteWasmStore;
use leptos::prelude::*;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Page {
    Menu,
    Grocery,
    Recipes,
    Ingredients,
    Clusters,
    Data,
}

#[component]
pub fn App() -> impl IntoView {
    let page = RwSignal::new(Page::Menu);

    let store = match SqliteWasmStore::new() {
        Ok(s) => s,
        Err(e) => {
            return view! {
                <main class="app">
                    <p class="error">"Failed to open the local database: " {e.to_string()}</p>
                </main>
            }
            .into_any();
        }
    };
    let initial = store.load_database_sync();
    let state = AppState::new(store, initial);
    provide_context(state.clone());

    view! {
        <main class="app">
            <header class="topbar">
                <h1>"Menu Planner"</h1>
                <nav class="tabs">
                    <button class:active=move || page.get() == Page::Menu on:click=move |_| page.set(Page::Menu)>"Menu"</button>
                    <button class:active=move || page.get() == Page::Grocery on:click=move |_| page.set(Page::Grocery)>"Grocery"</button>
                    <button class:active=move || page.get() == Page::Recipes on:click=move |_| page.set(Page::Recipes)>"Recipes"</button>
                    <button class:active=move || page.get() == Page::Ingredients on:click=move |_| page.set(Page::Ingredients)>"Ingredients"</button>
                    <button class:active=move || page.get() == Page::Clusters on:click=move |_| page.set(Page::Clusters)>"Clusters"</button>
                    <button class:active=move || page.get() == Page::Data on:click=move |_| page.set(Page::Data)>"Data"</button>
                </nav>
            </header>

            {move || {
                let state = use_app_state();
                state.error.get().map(|msg| view! {
                    <p class="error" on:click=move |_| use_app_state().error.set(None)>
                        {msg} " (tap to dismiss)"
                    </p>
                })
            }}

            <section class="page">
                {move || match page.get() {
                    Page::Menu => menu::MenuPage().into_any(),
                    Page::Grocery => grocery::GroceryPage().into_any(),
                    Page::Recipes => recipes::RecipesPage().into_any(),
                    Page::Ingredients => ingredients::IngredientsPage().into_any(),
                    Page::Clusters => clusters::ClustersPage().into_any(),
                    Page::Data => data::DataPage().into_any(),
                }}
            </section>
        </main>
    }
    .into_any()
}
