use crate::state::{use_app_state, View};
use leptos::prelude::*;
use shared::grocery::format_quantity;

/// Drill-down for a single base: what it's made of, and which recipes use it.
#[component]
pub fn BaseDetailPage(base_id: i64) -> impl IntoView {
    let state = use_app_state();

    let base = move || {
        state
            .db
            .get()
            .bases
            .iter()
            .find(|b| b.id == base_id)
            .cloned()
    };

    let used_by = move || {
        state
            .db
            .get()
            .recipes
            .iter()
            .filter(|r| {
                r.items
                    .iter()
                    .any(|i| i.ref_type == shared::models::RefType::Base && i.ref_id == base_id)
            })
            .map(|r| r.name.clone())
            .collect::<Vec<_>>()
    };

    let ingredient_name = move |id: i64| {
        state
            .db
            .get()
            .ingredients
            .iter()
            .find(|i| i.id == id)
            .map(|i| i.name.clone())
            .unwrap_or_else(|| "unknown ingredient".to_string())
    };

    let state_for_back = state.clone();

    view! {
        <div class="base-detail-page">
            <button on:click=move |_| state_for_back.go(View::Recipes)>"< Back to recipes"</button>
            {move || match base() {
                None => view! { <p class="hint">"That base no longer exists."</p> }.into_any(),
                Some(base) => {
                    let members = base.members.clone();
                    view! {
                        <div>
                            <h2>
                                <span class="badge badge-base">"base"</span>
                                " "
                                {base.name.clone()}
                            </h2>
                            {(!base.description.is_empty()).then(|| view! {
                                <p class="hint">{base.description.clone()}</p>
                            })}
                            <ul class="entity-list">
                                <For each=move || members.clone() key=|m| m.ingredient_id let:member>
                                    <li>
                                        <span class="name">{move || ingredient_name(member.ingredient_id)}</span>
                                        <span class="meta">
                                            {format_quantity(member.quantity)} " " {member.unit.clone()}
                                        </span>
                                    </li>
                                </For>
                            </ul>
                            <h3>"Used in"</h3>
                            <ul class="entity-list">
                                <For each=used_by key=|name| name.clone() let:name>
                                    <li><span class="name">{name.clone()}</span></li>
                                </For>
                            </ul>
                        </div>
                    }.into_any()
                }
            }}
        </div>
    }
}
