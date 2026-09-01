use crate::state::use_app_state;
use crate::store::Store;
use leptos::prelude::*;
use shared::models::{RecipeItem, RefType};
use shared::Recipe;

#[component]
pub fn RecipesPage() -> impl IntoView {
    let state = use_app_state();

    let editing_id = RwSignal::new(0i64);
    let name = RwSignal::new(String::new());
    let tags = RwSignal::new(String::new());
    let instructions = RwSignal::new(String::new());
    let servings = RwSignal::new("4".to_string());
    let items = RwSignal::new(Vec::<RecipeItem>::new());

    let item_ref_type = RwSignal::new("ingredient".to_string());
    let item_ref_id = RwSignal::new(0i64);
    let item_quantity = RwSignal::new(String::new());
    let item_unit = RwSignal::new(String::new());

    let reset_form = move || {
        editing_id.set(0);
        name.set(String::new());
        tags.set(String::new());
        instructions.set(String::new());
        servings.set("4".to_string());
        items.set(Vec::new());
        item_ref_type.set("ingredient".to_string());
        item_ref_id.set(0);
        item_quantity.set(String::new());
        item_unit.set(String::new());
    };

    let edit = move |recipe: Recipe| {
        editing_id.set(recipe.id);
        name.set(recipe.name);
        tags.set(recipe.tags.join(", "));
        instructions.set(recipe.instructions);
        servings.set(recipe.servings.to_string());
        items.set(recipe.items);
    };

    let add_item = move |_| {
        let ref_id = item_ref_id.get();
        let quantity: f64 = item_quantity.get().parse().unwrap_or(0.0);
        let unit = item_unit.get();
        if ref_id == 0 || quantity <= 0.0 || unit.trim().is_empty() {
            return;
        }
        let ref_type = if item_ref_type.get() == "cluster" {
            RefType::Cluster
        } else {
            RefType::Ingredient
        };
        items.update(|list| {
            list.push(RecipeItem {
                ref_type,
                ref_id,
                quantity,
                unit,
            })
        });
        item_ref_id.set(0);
        item_quantity.set(String::new());
        item_unit.set(String::new());
    };

    let state_for_save = state.clone();
    let save = move |_: ()| {
        let recipe = Recipe {
            id: editing_id.get(),
            name: name.get(),
            tags: tags
                .get()
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect(),
            instructions: instructions.get(),
            servings: servings.get().parse().unwrap_or(4),
            last_used: None,
            items: items.get(),
        };
        if recipe.name.trim().is_empty() {
            state_for_save.set_error("Recipe name is required");
            return;
        }
        // Preserve last_used when editing an existing recipe.
        let existing_last_used = state_for_save
            .db
            .get()
            .recipes
            .iter()
            .find(|r| r.id == recipe.id)
            .and_then(|r| r.last_used);
        let recipe = Recipe {
            last_used: existing_last_used,
            ..recipe
        };
        let state = state_for_save.clone();
        leptos::task::spawn_local(async move {
            match state.store.save_recipe(recipe).await {
                Ok(_) => state.reload(),
                Err(e) => state.set_error(e.to_string()),
            }
        });
        reset_form();
    };

    let state_for_delete = state.clone();
    let delete = move |id: i64| {
        let state = state_for_delete.clone();
        leptos::task::spawn_local(async move {
            match state.store.delete_recipe(id).await {
                Ok(_) => state.reload(),
                Err(e) => state.set_error(e.to_string()),
            }
        });
    };

    view! {
        <div class="recipes-page">
            <h2>"Recipes"</h2>
            <form class="card" on:submit=move |ev| { ev.prevent_default(); save(()); }>
                <label>"Name" <input type="text"
                    prop:value=move || name.get()
                    on:input=move |ev| name.set(event_target_value(&ev)) /></label>
                <label>"Tags (comma separated)" <input type="text" placeholder="vegetarian, quick, chicken"
                    prop:value=move || tags.get()
                    on:input=move |ev| tags.set(event_target_value(&ev)) /></label>
                <label>"Servings" <input type="number" min="1"
                    prop:value=move || servings.get()
                    on:input=move |ev| servings.set(event_target_value(&ev)) /></label>
                <label>"Instructions" <textarea
                    prop:value=move || instructions.get()
                    on:input=move |ev| instructions.set(event_target_value(&ev))></textarea></label>

                <fieldset>
                    <legend>"Ingredients / clusters"</legend>
                    <ul class="entity-list">
                        <For each={move || items.get().into_iter().enumerate().collect::<Vec<_>>()}
                            key=|(i, it)| (*i, it.ref_id)
                            let:entry
                        >
                            {
                                let (idx, item) = entry;
                                let label = move || {
                                    let db = state.db.get();
                                    match item.ref_type {
                                        RefType::Ingredient => db.ingredients.iter()
                                            .find(|i| i.id == item.ref_id)
                                            .map(|i| i.name.clone())
                                            .unwrap_or_else(|| "unknown ingredient".to_string()),
                                        RefType::Cluster => db.clusters.iter()
                                            .find(|c| c.id == item.ref_id)
                                            .map(|c| format!("{} (cluster)", c.name))
                                            .unwrap_or_else(|| "unknown cluster".to_string()),
                                    }
                                };
                                view! {
                                    <li>
                                        <span class="name">{label}</span>
                                        <span class="meta">{item.quantity} " " {item.unit.clone()}</span>
                                        <button type="button" on:click=move |_| {
                                            items.update(|list| { list.remove(idx); });
                                        }>"Remove"</button>
                                    </li>
                                }
                            }
                        </For>
                    </ul>

                    <div class="member-add">
                        <select on:change=move |ev| item_ref_type.set(event_target_value(&ev))>
                            <option value="ingredient">"Ingredient"</option>
                            <option value="cluster">"Cluster"</option>
                        </select>
                        <select on:change=move |ev| {
                            item_ref_id.set(event_target_value(&ev).parse().unwrap_or(0));
                        }>
                            <option value="0">"Choose..."</option>
                            {move || if item_ref_type.get() == "cluster" {
                                state.db.get().clusters.iter()
                                    .map(|c| view! { <option value=c.id.to_string()>{c.name.clone()}</option> })
                                    .collect::<Vec<_>>()
                            } else {
                                state.db.get().ingredients.iter()
                                    .map(|i| view! { <option value=i.id.to_string()>{i.name.clone()}</option> })
                                    .collect::<Vec<_>>()
                            }}
                        </select>
                        <input type="text" placeholder="quantity"
                            prop:value=move || item_quantity.get()
                            on:input=move |ev| item_quantity.set(event_target_value(&ev)) />
                        <input type="text" placeholder="unit"
                            prop:value=move || item_unit.get()
                            on:input=move |ev| item_unit.set(event_target_value(&ev)) />
                        <button type="button" on:click=add_item>"Add item"</button>
                    </div>
                </fieldset>

                <div class="actions">
                    <button type="submit">{move || if editing_id.get() == 0 { "Add recipe" } else { "Save changes" }}</button>
                    <button type="button" on:click=move |_| reset_form()>"Cancel"</button>
                </div>
            </form>

            <ul class="entity-list">
                <For each=move || state.db.get().recipes key=|r| r.id let:recipe>
                    <li>
                        <span class="name">{recipe.name.clone()}</span>
                        <span class="meta">{recipe.tags.join(", ")}</span>
                        <span class="row-actions">
                            <button on:click={
                                let recipe = recipe.clone();
                                move |_| edit(recipe.clone())
                            }>"Edit"</button>
                            <button on:click={let delete = delete.clone(); move |_| delete(recipe.id)}>"Delete"</button>
                        </span>
                    </li>
                </For>
            </ul>
        </div>
    }
}
