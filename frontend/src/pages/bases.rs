use crate::state::use_app_state;
use crate::store::Store;
use leptos::prelude::*;
use shared::models::{Base, BaseMember};

/// Create and edit bases (named ingredient building blocks such as
/// "adu-marcha" or "tadka base"). Bases have no top-level destination of
/// their own — this editor is embedded in the recipe editor, where bases are
/// actually needed.
#[component]
pub fn BaseEditor() -> impl IntoView {
    let state = use_app_state();

    let editing_id = RwSignal::new(0i64);
    let name = RwSignal::new(String::new());
    let description = RwSignal::new(String::new());
    let members = RwSignal::new(Vec::<BaseMember>::new());

    let member_ingredient = RwSignal::new(0i64);
    let member_quantity = RwSignal::new(String::new());
    let member_unit = RwSignal::new(String::new());

    let reset_form = move || {
        editing_id.set(0);
        name.set(String::new());
        description.set(String::new());
        members.set(Vec::new());
        member_ingredient.set(0);
        member_quantity.set(String::new());
        member_unit.set(String::new());
    };

    let edit = move |base: Base| {
        editing_id.set(base.id);
        name.set(base.name);
        description.set(base.description);
        members.set(base.members);
    };

    let add_member = move |_| {
        let ingredient_id = member_ingredient.get();
        let quantity: f64 = member_quantity.get().parse().unwrap_or(0.0);
        let unit = member_unit.get();
        if ingredient_id == 0 || quantity <= 0.0 || unit.trim().is_empty() {
            return;
        }
        members.update(|m| {
            m.push(BaseMember {
                ingredient_id,
                quantity,
                unit,
            })
        });
        member_quantity.set(String::new());
        member_unit.set(String::new());
    };

    let state_for_save = state.clone();
    let save = move |_: ()| {
        let base = Base {
            id: editing_id.get(),
            name: name.get(),
            description: description.get(),
            members: members.get(),
        };
        if base.name.trim().is_empty() {
            state_for_save.set_error("Base name is required");
            return;
        }
        if base.members.is_empty() {
            state_for_save.set_error("Add at least one member ingredient");
            return;
        }
        let state = state_for_save.clone();
        leptos::task::spawn_local(async move {
            match state.store.save_base(base).await {
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
            match state.store.delete_base(id).await {
                Ok(_) => state.reload(),
                Err(e) => state.set_error(e.to_string()),
            }
        });
    };

    view! {
        <div class="base-editor">
            <p class="hint">"Named groups of ingredients almost always used together (e.g. ginger-garlic-chilli paste)."</p>
            <div class="card">
                <label>"Name" <input type="text"
                    prop:value=move || name.get()
                    on:input=move |ev| name.set(event_target_value(&ev)) /></label>
                <label>"Description" <input type="text"
                    prop:value=move || description.get()
                    on:input=move |ev| description.set(event_target_value(&ev)) /></label>

                <fieldset>
                    <legend>"Members"</legend>
                    <ul class="entity-list">
                        <For each={move || members.get().into_iter().enumerate().collect::<Vec<_>>()}
                            key=|(i, m)| (*i, m.ingredient_id)
                            let:entry
                        >
                            {
                                let (idx, member) = entry;
                                let ingredient_name = move || {
                                    state.db.get().ingredients.iter()
                                        .find(|i| i.id == member.ingredient_id)
                                        .map(|i| i.name.clone())
                                        .unwrap_or_else(|| "unknown".to_string())
                                };
                                view! {
                                    <li>
                                        <span class="name">{ingredient_name}</span>
                                        <span class="meta">{member.quantity} " " {member.unit.clone()}</span>
                                        <button type="button" on:click=move |_| {
                                            members.update(|m| { m.remove(idx); });
                                        }>"Remove"</button>
                                    </li>
                                }
                            }
                        </For>
                    </ul>

                    <div class="member-add">
                        <select on:change=move |ev| {
                            member_ingredient.set(event_target_value(&ev).parse().unwrap_or(0));
                        }>
                            <option value="0">"Choose ingredient..."</option>
                            <For each=move || state.db.get().ingredients key=|i| i.id let:ing>
                                <option value=ing.id.to_string()>{ing.name.clone()}</option>
                            </For>
                        </select>
                        <input type="text" placeholder="quantity"
                            prop:value=move || member_quantity.get()
                            on:input=move |ev| member_quantity.set(event_target_value(&ev)) />
                        <input type="text" placeholder="unit"
                            prop:value=move || member_unit.get()
                            on:input=move |ev| member_unit.set(event_target_value(&ev)) />
                        <button type="button" on:click=add_member>"Add member"</button>
                    </div>
                </fieldset>

                <div class="actions">
                    <button type="button" class="primary" on:click=move |_| save(())>
                        {move || if editing_id.get() == 0 { "Add base" } else { "Save base" }}
                    </button>
                    <button type="button" on:click=move |_| reset_form()>"Cancel"</button>
                </div>
            </div>

            <ul class="entity-list">
                <For each=move || state.db.get().bases key=|b| b.id let:base>
                    <li>
                        <span class="name">
                            <span class="badge badge-base">"base"</span>
                            " "
                            {base.name.clone()}
                        </span>
                        <span class="meta">{base.members.len()} " ingredients"</span>
                        <span class="row-actions">
                            <button type="button" on:click={
                                let base = base.clone();
                                move |_| edit(base.clone())
                            }>"Edit"</button>
                            <button type="button" on:click={let delete = delete.clone(); move |_| delete(base.id)}>"Delete"</button>
                        </span>
                    </li>
                </For>
            </ul>
        </div>
    }
}
