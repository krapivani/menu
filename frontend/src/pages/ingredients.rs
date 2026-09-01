use crate::state::use_app_state;
use crate::store::Store;
use leptos::prelude::*;
use shared::models::Ingredient;

#[component]
pub fn IngredientsPage() -> impl IntoView {
    let state = use_app_state();

    let editing_id = RwSignal::new(0i64);
    let name = RwSignal::new(String::new());
    let category = RwSignal::new(String::new());
    let default_unit = RwSignal::new(String::new());
    let aisle = RwSignal::new(String::new());

    let reset_form = move || {
        editing_id.set(0);
        name.set(String::new());
        category.set(String::new());
        default_unit.set(String::new());
        aisle.set(String::new());
    };

    let edit = move |ing: Ingredient| {
        editing_id.set(ing.id);
        name.set(ing.name);
        category.set(ing.category);
        default_unit.set(ing.default_unit);
        aisle.set(ing.aisle);
    };

    let state_for_save = state.clone();
    let save = move |_: ()| {
        let ingredient = Ingredient {
            id: editing_id.get(),
            name: name.get(),
            category: category.get(),
            default_unit: default_unit.get(),
            aisle: aisle.get(),
        };
        if ingredient.name.trim().is_empty() {
            state_for_save.set_error("Ingredient name is required");
            return;
        }
        let state = state_for_save.clone();
        leptos::task::spawn_local(async move {
            match state.store.save_ingredient(ingredient).await {
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
            match state.store.delete_ingredient(id).await {
                Ok(_) => state.reload(),
                Err(e) => state.set_error(e.to_string()),
            }
        });
    };

    view! {
        <div class="ingredients-page">
            <h2>"Ingredients"</h2>
            <form class="card" on:submit=move |ev| { ev.prevent_default(); save(()); }>
                <label>"Name" <input type="text"
                    prop:value=move || name.get()
                    on:input=move |ev| name.set(event_target_value(&ev)) /></label>
                <label>"Category" <input type="text" placeholder="produce / dairy / meat / spice / pantry"
                    prop:value=move || category.get()
                    on:input=move |ev| category.set(event_target_value(&ev)) /></label>
                <label>"Default unit" <input type="text" placeholder="tsp, cup, lb..."
                    prop:value=move || default_unit.get()
                    on:input=move |ev| default_unit.set(event_target_value(&ev)) /></label>
                <label>"Aisle" <input type="text"
                    prop:value=move || aisle.get()
                    on:input=move |ev| aisle.set(event_target_value(&ev)) /></label>
                <div class="actions">
                    <button type="submit">{move || if editing_id.get() == 0 { "Add ingredient" } else { "Save changes" }}</button>
                    <button type="button" on:click=move |_| reset_form()>"Cancel"</button>
                </div>
            </form>

            <ul class="entity-list">
                <For
                    each=move || {
                        let mut items = state.db.get().ingredients;
                        items.sort_by(|a, b| a.name.cmp(&b.name));
                        items
                    }
                    key=|ing| ing.id
                    let:ing
                >
                    <li>
                        <span class="name">{ing.name.clone()}</span>
                        <span class="meta">{ing.category.clone()} " · " {ing.default_unit.clone()} " · " {ing.aisle.clone()}</span>
                        <span class="row-actions">
                            <button on:click={
                                let ing = ing.clone();
                                move |_| edit(ing.clone())
                            }>"Edit"</button>
                            <button on:click={let delete = delete.clone(); move |_| delete(ing.id)}>"Delete"</button>
                        </span>
                    </li>
                </For>
            </ul>
        </div>
    }
}
