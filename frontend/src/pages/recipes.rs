use crate::pages::bases::BaseEditor;
use crate::state::{use_app_state, View};
use crate::store::Store;
use leptos::prelude::*;
use shared::grocery::format_quantity;
use shared::models::{RecipeItem, RecipeRole, RefType};
use shared::Recipe;

/// A recipe's ingredient lines, shown exactly as authored: plain ingredients
/// as themselves, and a base as a single named line. Base lines expand
/// in-place to reveal their members and scaled quantities, rather than being
/// flattened into them.
#[component]
fn RecipeItemList(items: Vec<RecipeItem>) -> impl IntoView {
    let state = use_app_state();
    let expanded = RwSignal::new(None::<i64>);

    view! {
        <ul class="recipe-items">
            <For each={move || items.clone().into_iter().enumerate().collect::<Vec<_>>()}
                key=|(i, item)| (*i, item.ref_id)
                let:entry
            >
                {
                    let (_, item) = entry;
                    match item.ref_type {
                        RefType::Ingredient => {
                            let name = move || state.db.get().ingredients.iter()
                                .find(|i| i.id == item.ref_id)
                                .map(|i| i.name.clone())
                                .unwrap_or_else(|| "unknown ingredient".to_string());
                            view! {
                                <li class="recipe-item">
                                    <span class="name">{name}</span>
                                    <span class="meta">{format_quantity(item.quantity)} " " {item.unit.clone()}</span>
                                </li>
                            }.into_any()
                        }
                        RefType::Base => {
                            let base = move || state.db.get().bases.iter()
                                .find(|b| b.id == item.ref_id)
                                .cloned();
                            let is_open = move || expanded.get() == Some(item.ref_id);
                            let state_for_open = state.clone();
                            view! {
                                <li class="recipe-item recipe-item-base">
                                    <button type="button" class="base-line"
                                        aria-expanded=move || is_open().to_string()
                                        on:click=move |_| expanded.update(|e| {
                                            *e = if *e == Some(item.ref_id) { None } else { Some(item.ref_id) };
                                        })
                                    >
                                        <span class="badge badge-base">"base"</span>
                                        <span class="name">
                                            {move || base().map(|b| b.name).unwrap_or_else(|| "unknown base".to_string())}
                                        </span>
                                        <span class="meta">{format_quantity(item.quantity)} " " {item.unit.clone()}</span>
                                        <span class="chevron">{move || if is_open() { "▾" } else { "▸" }}</span>
                                    </button>
                                    {move || is_open().then(|| {
                                        let members = base().map(|b| b.members).unwrap_or_default();
                                        let state_for_link = state_for_open.clone();
                                        view! {
                                            <div class="base-members">
                                                <ul class="recipe-items">
                                                    <For each=move || members.clone() key=|m| m.ingredient_id let:member>
                                                        <li class="recipe-item">
                                                            <span class="name">
                                                                {move || state.db.get().ingredients.iter()
                                                                    .find(|i| i.id == member.ingredient_id)
                                                                    .map(|i| i.name.clone())
                                                                    .unwrap_or_else(|| "unknown ingredient".to_string())}
                                                            </span>
                                                            <span class="meta">
                                                                {format_quantity(member.quantity * item.quantity)}
                                                                " " {member.unit.clone()}
                                                            </span>
                                                        </li>
                                                    </For>
                                                </ul>
                                                <button type="button" class="link-button"
                                                    on:click=move |_| state_for_link.go(View::Base(item.ref_id))
                                                >"Open base"</button>
                                            </div>
                                        }
                                    })}
                                </li>
                            }.into_any()
                        }
                    }
                }
            </For>
        </ul>
    }
}

#[component]
pub fn RecipesPage() -> impl IntoView {
    let state = use_app_state();

    let editing_id = RwSignal::new(0i64);
    let name = RwSignal::new(String::new());
    let role = RwSignal::new(RecipeRole::OnePot.as_str().to_string());
    let cuisine = RwSignal::new("gujarati".to_string());
    let treat = RwSignal::new(false);
    let tags = RwSignal::new(String::new());
    let instructions = RwSignal::new(String::new());
    let servings = RwSignal::new("4".to_string());
    let items = RwSignal::new(Vec::<RecipeItem>::new());
    let show_base_editor = RwSignal::new(false);
    let opened_recipe = RwSignal::new(None::<i64>);

    let item_ref_type = RwSignal::new("ingredient".to_string());
    let item_ref_id = RwSignal::new(0i64);
    let item_quantity = RwSignal::new(String::new());
    let item_unit = RwSignal::new(String::new());

    let reset_form = move || {
        editing_id.set(0);
        name.set(String::new());
        role.set(RecipeRole::OnePot.as_str().to_string());
        cuisine.set("gujarati".to_string());
        treat.set(false);
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
        role.set(recipe.role.as_str().to_string());
        cuisine.set(recipe.cuisine);
        treat.set(recipe.treat);
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
        let ref_type = if item_ref_type.get() == "base" {
            RefType::Base
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
            role: RecipeRole::from(role.get().as_str()),
            cuisine: cuisine.get().trim().to_string(),
            treat: treat.get(),
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
        if recipe.cuisine.trim().is_empty() {
            state_for_save.set_error("Cuisine is required");
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
                <label>"Role"
                    <select prop:value=move || role.get()
                        on:change=move |ev| role.set(event_target_value(&ev))>
                        <For each=move || RecipeRole::ALL key=|r| r.as_str() let:recipe_role>
                            <option value=recipe_role.as_str()>{recipe_role.label()}</option>
                        </For>
                    </select>
                </label>
                <label>"Cuisine" <input type="text" placeholder="gujarati"
                    prop:value=move || cuisine.get()
                    on:input=move |ev| cuisine.set(event_target_value(&ev)) /></label>
                <label class="toggle">
                    <input type="checkbox"
                        prop:checked=move || treat.get()
                        on:change=move |_| treat.update(|v| *v = !*v) />
                    " Treat / cheat meal"
                </label>
                <label>"Tags (comma separated)" <input type="text" placeholder="vegetarian, quick, gujarati"
                    prop:value=move || tags.get()
                    on:input=move |ev| tags.set(event_target_value(&ev)) /></label>
                <label>"Servings" <input type="number" min="1"
                    prop:value=move || servings.get()
                    on:input=move |ev| servings.set(event_target_value(&ev)) /></label>
                <label>"Instructions" <textarea
                    prop:value=move || instructions.get()
                    on:input=move |ev| instructions.set(event_target_value(&ev))></textarea></label>

                <fieldset>
                    <legend>"Ingredients & bases"</legend>
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
                                        RefType::Base => db.bases.iter()
                                            .find(|b| b.id == item.ref_id)
                                            .map(|b| b.name.clone())
                                            .unwrap_or_else(|| "unknown base".to_string()),
                                    }
                                };
                                let is_base = item.ref_type == RefType::Base;
                                view! {
                                    <li>
                                        <span class="name">
                                            {is_base.then(|| view! { <span class="badge badge-base">"base"</span> })}
                                            " "
                                            {label}
                                        </span>
                                        <span class="meta">{format_quantity(item.quantity)} " " {item.unit.clone()}</span>
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
                            <option value="base">"Base"</option>
                        </select>
                        <select on:change=move |ev| {
                            item_ref_id.set(event_target_value(&ev).parse().unwrap_or(0));
                        }>
                            <option value="0">"Choose..."</option>
                            {move || if item_ref_type.get() == "base" {
                                state.db.get().bases.iter()
                                    .map(|b| view! { <option value=b.id.to_string()>{b.name.clone()}</option> })
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

                    // Bases have no top-level destination; they're managed
                    // right here, where they get used.
                    <button type="button" class="link-button"
                        on:click=move |_| show_base_editor.update(|v| *v = !*v)
                    >{move || if show_base_editor.get() { "Hide bases" } else { "Manage bases" }}</button>
                    {move || show_base_editor.get().then(|| view! { <BaseEditor /> })}
                </fieldset>

                <div class="actions">
                    <button type="submit">{move || if editing_id.get() == 0 { "Add recipe" } else { "Save changes" }}</button>
                    <button type="button" on:click=move |_| reset_form()>"Cancel"</button>
                </div>
            </form>

            <ul class="entity-list">
                <For each=move || state.db.get().recipes key=|r| r.id let:recipe>
                    {
                        let recipe_items = recipe.items.clone();
                        let recipe_id = recipe.id;
                        let is_open = move || opened_recipe.get() == Some(recipe_id);
                        view! {
                            <li class="recipe-row">
                                <span class="name">
                                    <button type="button" class="link-button"
                                        aria-expanded=move || is_open().to_string()
                                        on:click=move |_| opened_recipe.update(|o| {
                                            *o = if *o == Some(recipe_id) { None } else { Some(recipe_id) };
                                        })
                                    >{recipe.name.clone()}</button>
                                </span>
                                <span class="meta">
                                    <span class="badge">{recipe.role.label()}</span>
                                    " "
                                    <span class="badge">{recipe.cuisine.clone()}</span>
                                    " "
                                    {recipe.treat.then(|| view! { <span class="badge">"treat"</span> })}
                                    " "
                                    {recipe.tags.join(", ")}
                                </span>
                                <span class="row-actions">
                                    <button type="button" on:click={
                                        let recipe = recipe.clone();
                                        move |_| edit(recipe.clone())
                                    }>"Edit"</button>
                                    <button type="button" on:click={let delete = delete.clone(); move |_| delete(recipe_id)}>"Delete"</button>
                                </span>
                                {move || is_open().then(|| {
                                    let items = recipe_items.clone();
                                    view! { <RecipeItemList items=items /> }
                                })}
                            </li>
                        }
                    }
                </For>
            </ul>
        </div>
    }
}
