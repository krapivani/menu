use crate::state::{use_app_state, View};
use crate::store::Store;
use leptos::prelude::*;
use shared::models::{PlanDay, RecipeRole};
use shared::rotation::generate_rotation;
use std::collections::{HashMap, HashSet};

/// The primary screen: filter, generate a rotation, review it day by day, and
/// — only once a plan exists — head to its grocery list.
#[component]
pub fn HomePage() -> impl IntoView {
    let state = use_app_state();

    let days = RwSignal::new("7".to_string());
    let selected_tags = RwSignal::new(HashSet::<String>::new());
    let selected_cuisines = RwSignal::new(HashSet::<String>::new());
    let pinned = RwSignal::new(HashMap::<usize, PlanDay>::new());
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

    let all_cuisines = move || {
        let mut cuisines: Vec<String> = state
            .db
            .get()
            .recipes
            .iter()
            .map(|r| r.cuisine.clone())
            .filter(|c| !c.is_empty())
            .collect();
        cuisines.sort();
        cuisines.dedup();
        cuisines
    };

    let recipe_summary = move |id: i64| {
        state
            .db
            .get()
            .recipes
            .iter()
            .find(|r| r.id == id)
            .map(|r| (r.name.clone(), r.role, r.treat))
            .unwrap_or_else(|| ("unknown recipe".to_string(), RecipeRole::OnePot, false))
    };

    let state_for_generate = state.clone();
    let generate = move |_| {
        let n: usize = days.get().parse().unwrap_or(7);
        let tags: Vec<String> = selected_tags.get().into_iter().collect();
        let cuisines: Vec<String> = selected_cuisines.get().into_iter().collect();
        let db = state_for_generate.db.get();
        match generate_rotation(&db.recipes, n, &tags, &cuisines, &pinned.get(), None) {
            Ok(result) => plan.set(result),
            Err(e) => state_for_generate.set_error(e.to_string()),
        }
    };

    let state_for_reroll = state.clone();
    let reroll = move |day: usize| {
        let tags: Vec<String> = selected_tags.get().into_iter().collect();
        let cuisines: Vec<String> = selected_cuisines.get().into_iter().collect();
        let db = state_for_reroll.db.get();
        let current = plan.get();
        match shared::rotation::reroll_day(&db.recipes, &current, day, &tags, &cuisines, None) {
            Ok(new_day) => plan.update(|p| {
                if let Some(slot) = p.get_mut(day) {
                    *slot = new_day;
                }
            }),
            Err(e) => state_for_reroll.set_error(e.to_string()),
        }
    };

    let state_for_use = state.clone();
    let use_plan = move |_| {
        let state = state_for_use.clone();
        let recipe_ids: Vec<i64> = plan
            .get()
            .into_iter()
            .flat_map(|day| day.recipe_ids.into_iter())
            .collect();
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

    let state_for_grocery = state.clone();
    let open_grocery = move |_| state_for_grocery.go(View::Grocery);

    view! {
        <div class="home-page">
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

                <fieldset>
                    <legend>"Cuisine filters"</legend>
                    <For each=all_cuisines key=|c| c.clone() let:cuisine>
                        {
                            let cuisine_for_checked = cuisine.clone();
                            let cuisine_for_click = cuisine.clone();
                            view! {
                                <label class="tag-filter">
                                    <input type="checkbox"
                                        prop:checked=move || selected_cuisines.get().contains(&cuisine_for_checked)
                                        on:change=move |_| {
                                            selected_cuisines.update(|s| {
                                                if !s.remove(&cuisine_for_click) {
                                                    s.insert(cuisine_for_click.clone());
                                                }
                                            });
                                        } />
                                    {cuisine.clone()}
                                </label>
                            }
                        }
                    </For>
                </fieldset>

                <button class="primary" on:click=generate>"Generate"</button>
            </div>

            <ol class="plan-list">
                <For each={move || plan.get().into_iter().enumerate().collect::<Vec<_>>()}
                    key=|(day, plan_day)| (*day, plan_day.recipe_ids.clone())
                    let:entry
                >
                    {
                        let (day, plan_day) = entry;
                        let pin_day = plan_day.clone();
                        view! {
                            <li class:pinned=move || pinned.get().contains_key(&day)>
                                <span class="day">"Day " {day + 1}</span>
                                <div class="day-recipes">
                                    <For each=move || plan_day.recipe_ids.clone() key=|id| *id let:recipe_id>
                                        {
                                            let (name, role, treat) = recipe_summary(recipe_id);
                                            view! {
                                                <div class="day-recipe">
                                                    <span class="badge">{role.label()}</span>
                                                    {treat.then(|| view! { <span class="badge">"treat"</span> })}
                                                    <span class="name">{name}</span>
                                                </div>
                                            }
                                        }
                                    </For>
                                </div>
                                <label class="pin">
                                    <input type="checkbox"
                                        prop:checked=move || pinned.get().contains_key(&day)
                                        on:change=move |_| {
                                            pinned.update(|p| {
                                                if p.remove(&day).is_none() {
                                                    p.insert(day, pin_day.clone());
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

            // The grocery list only exists once there is a plan to shop for,
            // so its entry point appears here rather than in the top nav.
            {move || {
                let use_plan = use_plan.clone();
                let open_grocery = open_grocery.clone();
                (!plan.get().is_empty()).then(|| view! {
                    <div class="actions plan-actions">
                        <button class="primary" on:click=open_grocery>"View grocery list"</button>
                        <button on:click=use_plan>"Use this plan (mark recipes as used)"</button>
                    </div>
                })
            }}
        </div>
    }
}
