use crate::state::use_app_state;
use crate::store::Store;
use leptos::prelude::*;
use shared::rotation::generate_rotation;
use std::collections::{HashMap, HashSet};

#[component]
pub fn MenuPage() -> impl IntoView {
    let state = use_app_state();

    let days = RwSignal::new("7".to_string());
    let selected_tags = RwSignal::new(HashSet::<String>::new());
    let pinned = RwSignal::new(HashMap::<usize, i64>::new());
    let plan = state.current_plan;

    let all_tags = move || {
        let mut tags: Vec<String> = state
            .db
            .get()
            .recipes
            .iter()
            .flat_map(|r| r.tags.clone())
            .collect();
        tags.sort();
        tags.dedup();
        tags
    };

    let recipe_name = move |id: i64| {
        state
            .db
            .get()
            .recipes
            .iter()
            .find(|r| r.id == id)
            .map(|r| r.name.clone())
            .unwrap_or_else(|| "unknown recipe".to_string())
    };

    let state_for_generate = state.clone();
    let generate = move |_| {
        let n: usize = days.get().parse().unwrap_or(7);
        let tags: Vec<String> = selected_tags.get().into_iter().collect();
        let db = state_for_generate.db.get();
        match generate_rotation(&db.recipes, n, &tags, &pinned.get(), None) {
            Ok(result) => plan.set(result),
            Err(e) => state_for_generate.set_error(e.to_string()),
        }
    };

    let state_for_reroll = state.clone();
    let reroll = move |day: usize| {
        let tags: Vec<String> = selected_tags.get().into_iter().collect();
        let db = state_for_reroll.db.get();
        let current = plan.get();
        match shared::rotation::reroll_day(&db.recipes, &current, day, &tags, None) {
            Ok(new_id) => plan.update(|p| p[day] = new_id),
            Err(e) => state_for_reroll.set_error(e.to_string()),
        }
    };

    let state_for_use = state.clone();
    let use_plan = move |_| {
        let state = state_for_use.clone();
        let recipe_ids: Vec<i64> = plan.get();
        leptos::task::spawn_local(async move {
            let now = js_sys::Date::now() as i64;
            for id in recipe_ids {
                if let Err(e) = state.store.touch_recipe_last_used(id, now).await {
                    state.set_error(e.to_string());
                }
            }
            state.reload();
        });
    };

    view! {
        <div class="menu-page">
            <h2>"Generate Menu"</h2>
            <div class="card">
                <label>"Days" <input type="number" min="1" max="14"
                    prop:value=move || days.get()
                    on:input=move |ev| days.set(event_target_value(&ev)) /></label>

                <fieldset>
                    <legend>"Tag filters"</legend>
                    <For each=all_tags key=|t| t.clone() let:tag>
                        {
                            let tag_for_checked = tag.clone();
                            let tag_for_click = tag.clone();
                            view! {
                                <label class="tag-filter">
                                    <input type="checkbox"
                                        prop:checked=move || selected_tags.get().contains(&tag_for_checked)
                                        on:change=move |_| {
                                            selected_tags.update(|s| {
                                                if !s.remove(&tag_for_click) {
                                                    s.insert(tag_for_click.clone());
                                                }
                                            });
                                        } />
                                    {tag.clone()}
                                </label>
                            }
                        }
                    </For>
                </fieldset>

                <button on:click=generate>"Generate rotation"</button>
            </div>

            <ol class="plan-list">
                <For each={move || plan.get().into_iter().enumerate().collect::<Vec<_>>()}
                    key=|(day, id)| (*day, *id)
                    let:entry
                >
                    {
                        let (day, recipe_id) = entry;
                        view! {
                            <li>
                                <span class="day">"Day " {day + 1}</span>
                                <span class="name">{move || recipe_name(recipe_id)}</span>
                                <label class="pin">
                                    <input type="checkbox"
                                        prop:checked=move || pinned.get().contains_key(&day)
                                        on:change=move |_| {
                                            pinned.update(|p| {
                                                if p.remove(&day).is_none() {
                                                    p.insert(day, recipe_id);
                                                }
                                            });
                                        } />
                                    " pin"
                                </label>
                                <button type="button" on:click={let reroll = reroll.clone(); move |_| reroll(day)}>"Re-roll"</button>
                            </li>
                        }
                    }
                </For>
            </ol>

            {move || {
                let use_plan = use_plan.clone();
                (!plan.get().is_empty()).then(|| view! {
                    <button class="primary" on:click=use_plan>"Use this plan (mark recipes as used)"</button>
                })
            }}
        </div>
    }
}
