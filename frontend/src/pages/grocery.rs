use crate::state::use_app_state;
use leptos::prelude::*;
use shared::grocery::{build_grocery_list, group_by_category};
use std::collections::{HashMap, HashSet};

#[component]
pub fn GroceryPage() -> impl IntoView {
    let state = use_app_state();
    let checked = RwSignal::new(HashSet::<String>::new());

    let grouped = move || {
        let db = state.db.get();
        let plan = state.current_plan.get();
        let recipes: Vec<_> = db
            .recipes
            .iter()
            .filter(|r| plan.contains(&r.id))
            .cloned()
            .collect();
        let ingredients: HashMap<i64, _> =
            db.ingredients.iter().map(|i| (i.id, i.clone())).collect();
        let clusters: HashMap<i64, _> = db.clusters.iter().map(|c| (c.id, c.clone())).collect();
        let lines = build_grocery_list(
            &recipes,
            &ingredients,
            &clusters,
            state.expand_clusters.get(),
        );
        group_by_category(&lines)
    };

    view! {
        <div class="grocery-page">
            <h2>"Grocery List"</h2>
            <label class="toggle">
                <input type="checkbox"
                    prop:checked=move || state.expand_clusters.get()
                    on:change=move |_| state.expand_clusters.update(|v| *v = !*v) />
                " Expand clusters into individual ingredients"
            </label>

            {move || if state.current_plan.get().is_empty() {
                Some(view! { <p class="hint">"Generate a menu first to build a grocery list."</p> })
            } else {
                None
            }}

            <div class="grocery-groups">
                <For each=grouped key=|(cat, _)| cat.clone() let:cat_group>
                    {
                        let (category, aisles) = cat_group;
                        view! {
                            <section class="grocery-category">
                                <h3>{category.clone()}</h3>
                                <For each=move || aisles.clone() key=|(aisle, _)| aisle.clone() let:aisle_group>
                                    {
                                        let (aisle, lines) = aisle_group;
                                        view! {
                                            <div class="grocery-aisle">
                                                <h4>{aisle.clone()}</h4>
                                                <ul class="grocery-lines">
                                                    <For each=move || lines.clone() key=|l| format!("{}-{}", l.name, l.unit) let:line>
                                                        {
                                                            let key = format!("{}-{}", line.name, line.unit);
                                                            let key_for_checked = key.clone();
                                                            view! {
                                                                <li>
                                                                    <label>
                                                                        <input type="checkbox"
                                                                            prop:checked=move || checked.get().contains(&key_for_checked)
                                                                            on:change=move |_| {
                                                                                checked.update(|c| {
                                                                                    if !c.remove(&key) {
                                                                                        c.insert(key.clone());
                                                                                    }
                                                                                });
                                                                            } />
                                                                        {format!("{} {} {}", line.quantity, line.unit, line.name)}
                                                                    </label>
                                                                </li>
                                                            }
                                                        }
                                                    </For>
                                                </ul>
                                            </div>
                                        }
                                    }
                                </For>
                            </section>
                        }
                    }
                </For>
            </div>
        </div>
    }
}
