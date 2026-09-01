use crate::pages::{base_detail, data, grocery, home, ingredients, recipes};
use crate::state::{use_app_state, AppState, View};
use crate::store::SqliteWasmStore;
use leptos::prelude::*;

/// Which of the two overflow menus in the top bar is open, if any.
#[derive(Clone, Copy, PartialEq, Eq)]
enum OpenMenu {
    Add,
    Settings,
}

#[component]
pub fn App() -> impl IntoView {
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

    let view_signal = state.view;
    let open_menu = RwSignal::new(None::<OpenMenu>);

    let navigate = {
        let state = state.clone();
        move |target: View| {
            open_menu.set(None);
            state.go(target);
        }
    };

    view! {
        <main class="app">
            <header class="topbar">
                <button class="brand" on:click={
                    let navigate = navigate.clone();
                    move |_| navigate(View::Home)
                }>"Menu Planner"</button>

                <nav class="nav-actions">
                    <button
                        class="nav-link"
                        class:active=move || view_signal.get() == View::Recipes
                        on:click={let navigate = navigate.clone(); move |_| navigate(View::Recipes)}
                    >"Recipes"</button>

                    <div class="menu-wrap">
                        <button
                            class="nav-icon"
                            aria-haspopup="true"
                            aria-expanded=move || (open_menu.get() == Some(OpenMenu::Add)).to_string()
                            title="Add"
                            on:click=move |_| open_menu.update(|m| {
                                *m = if *m == Some(OpenMenu::Add) { None } else { Some(OpenMenu::Add) };
                            })
                        >"+ Add"</button>
                        {
                            let navigate = navigate.clone();
                            move || (open_menu.get() == Some(OpenMenu::Add)).then(|| {
                            let navigate_ing = navigate.clone();
                            let navigate_rec = navigate.clone();
                            view! {
                                <div class="menu-popover" role="menu">
                                    <button role="menuitem" on:click=move |_| navigate_ing(View::Ingredients)>"Add ingredient"</button>
                                    <button role="menuitem" on:click=move |_| navigate_rec(View::Recipes)>"Add recipe"</button>
                                </div>
                            }
                        })}
                    </div>

                    <div class="menu-wrap">
                        <button
                            class="nav-icon"
                            aria-haspopup="true"
                            aria-expanded=move || (open_menu.get() == Some(OpenMenu::Settings)).to_string()
                            aria-label="Settings"
                            title="Settings"
                            on:click=move |_| open_menu.update(|m| {
                                *m = if *m == Some(OpenMenu::Settings) { None } else { Some(OpenMenu::Settings) };
                            })
                        >"⚙"</button>
                        {
                            let navigate = navigate.clone();
                            move || (open_menu.get() == Some(OpenMenu::Settings)).then(|| {
                            let navigate_ing = navigate.clone();
                            let navigate_data = navigate.clone();
                            view! {
                                <div class="menu-popover" role="menu">
                                    <button role="menuitem" on:click=move |_| navigate_ing(View::Ingredients)>"Manage ingredients"</button>
                                    <button role="menuitem" on:click=move |_| navigate_data(View::Data)>"Backup & restore (JSON)"</button>
                                </div>
                            }
                        })}
                    </div>
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
                {move || match view_signal.get() {
                    View::Home => home::HomePage().into_any(),
                    View::Grocery => grocery::GroceryPage().into_any(),
                    View::Recipes => recipes::RecipesPage().into_any(),
                    View::Base(id) => view! { <base_detail::BaseDetailPage base_id=id /> }.into_any(),
                    View::Ingredients => ingredients::IngredientsPage().into_any(),
                    View::Data => data::DataPage().into_any(),
                }}
            </section>
        </main>
    }
    .into_any()
}
