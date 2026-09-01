use crate::state::{use_app_state, View};
use leptos::prelude::*;
use shared::grocery::{
    build_grocery_list, format_quantity, grocery_list_to_text, group_by_category,
};
use std::collections::{HashMap, HashSet};
use wasm_bindgen::{JsCast, JsValue};

/// Start an async clipboard write, returning the pending promise. `None`
/// means the browser doesn't expose the API at all (older browsers, or an
/// insecure origin), so the caller can fall back to the `.txt` download.
///
/// `web-sys`' `Clipboard` binding is behind `web_sys_unstable_apis`, which
/// would mean threading a `RUSTFLAGS` cfg through every build; reflecting on
/// `navigator.clipboard` keeps the build plain.
fn clipboard_write(text: &str) -> Option<js_sys::Promise> {
    let window = web_sys::window()?;
    let navigator = window.navigator();
    let clipboard = js_sys::Reflect::get(&navigator, &JsValue::from_str("clipboard")).ok()?;
    if clipboard.is_undefined() || clipboard.is_null() {
        return None;
    }
    let write_text = js_sys::Reflect::get(&clipboard, &JsValue::from_str("writeText"))
        .ok()?
        .dyn_into::<js_sys::Function>()
        .ok()?;
    write_text
        .call1(&clipboard, &JsValue::from_str(text))
        .ok()?
        .dyn_into::<js_sys::Promise>()
        .ok()
}

/// Ask the browser to print the page. With the `@media print` rules in
/// `style.css` this doubles as "save as PDF" via the native print dialog,
/// which avoids shipping a PDF library in the WASM bundle.
fn print_page() -> bool {
    let Some(window) = web_sys::window() else {
        return false;
    };
    let Ok(print) = js_sys::Reflect::get(&window, &JsValue::from_str("print")) else {
        return false;
    };
    let Ok(print) = print.dyn_into::<js_sys::Function>() else {
        return false;
    };
    print.call0(&window).is_ok()
}

#[component]
pub fn GroceryPage() -> impl IntoView {
    let state = use_app_state();
    let checked = RwSignal::new(HashSet::<String>::new());
    let new_item = RwSignal::new(String::new());
    let status = RwSignal::new(String::new());
    let text_href = RwSignal::new(String::new());

    let lines = move || {
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
        let bases: HashMap<i64, _> = db.bases.iter().map(|b| (b.id, b.clone())).collect();
        build_grocery_list(&recipes, &ingredients, &bases, state.expand_bases.get())
    };

    let grouped = move || group_by_category(&lines());

    let as_text = move || grocery_list_to_text(&lines(), &state.extra_items.get());

    let state_for_add = state.clone();
    let add_item = move |_| {
        let text = new_item.get().trim().to_string();
        if text.is_empty() {
            return;
        }
        let mut items = state_for_add.extra_items.get();
        items.push(text);
        state_for_add.set_extra_items(items);
        new_item.set(String::new());
    };

    let state_for_remove = state.clone();
    let remove_item = move |idx: usize| {
        let mut items = state_for_remove.extra_items.get();
        if idx < items.len() {
            items.remove(idx);
            state_for_remove.set_extra_items(items);
        }
    };

    const CLIPBOARD_FALLBACK: &str =
        "Couldn't copy in this browser — use the .txt download instead.";

    let copy = move |_| {
        let text = as_text();
        match clipboard_write(&text) {
            // The write can still be rejected (denied permission), so wait
            // for the promise before claiming success.
            Some(promise) => leptos::task::spawn_local(async move {
                match wasm_bindgen_futures::JsFuture::from(promise).await {
                    Ok(_) => status.set("Copied to clipboard.".to_string()),
                    Err(_) => status.set(CLIPBOARD_FALLBACK.to_string()),
                }
            }),
            None => status.set(CLIPBOARD_FALLBACK.to_string()),
        }
    };

    let download_txt = move |_| {
        let encoded = js_sys::encode_uri_component(&as_text());
        text_href.set(format!("data:text/plain;charset=utf-8,{encoded}"));
    };

    let print = move |_| {
        if !print_page() {
            status.set("Printing isn't available in this browser.".to_string());
        }
    };

    let state_for_back = state.clone();
    let back_home = move |_| state_for_back.go(View::Home);

    view! {
        <div class="grocery-page">
            <h2>"Grocery List"</h2>

            <div class="actions no-print">
                <button on:click=back_home>"< Back to menu"</button>
                <button on:click=print>"Save as PDF / print"</button>
                <button on:click=copy>"Copy to clipboard"</button>
                <button on:click=download_txt>"Download .txt"</button>
            </div>

            {move || (!text_href.get().is_empty()).then(|| {
                let href = text_href.get();
                view! {
                    <a href=href download="grocery-list.txt" class="download-link no-print">
                        "Download grocery-list.txt"
                    </a>
                }
            })}

            {move || (!status.get().is_empty()).then(|| view! {
                <p class="hint no-print">{status.get()}</p>
            })}

            <label class="toggle no-print">
                <input type="checkbox"
                    prop:checked=move || state.expand_bases.get()
                    on:change=move |_| state.expand_bases.update(|v| *v = !*v) />
                " Expand bases into individual ingredients"
            </label>

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
                                                                        {format!("{} {} {}", format_quantity(line.quantity), line.unit, line.name)}
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

            // Ad-hoc items: things no recipe asks for (milk, bin bags). Kept
            // visually distinct from the generated lines above.
            <section class="grocery-category grocery-extras">
                <h3>"Other items"</h3>
                <ul class="grocery-lines">
                    <For each={move || state.extra_items.get().into_iter().enumerate().collect::<Vec<_>>()}
                        key=|(i, item)| (*i, item.clone())
                        let:entry
                    >
                        {
                            let (idx, item) = entry;
                            let key = format!("extra-{idx}-{item}");
                            let key_for_checked = key.clone();
                            let remove_item = remove_item.clone();
                            view! {
                                <li class="extra-item">
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
                                        <span class="badge">"added"</span>
                                        {item.clone()}
                                    </label>
                                    <button type="button" class="danger no-print"
                                        on:click=move |_| remove_item(idx)>"Remove"</button>
                                </li>
                            }
                        }
                    </For>
                </ul>

                <div class="member-add no-print">
                    <input type="text" placeholder="Add an item (milk, bin bags...)"
                        prop:value=move || new_item.get()
                        on:input=move |ev| new_item.set(event_target_value(&ev))
                        on:keydown={
                            let add_item = add_item.clone();
                            move |ev: leptos::ev::KeyboardEvent| {
                                if ev.key() == "Enter" {
                                    ev.prevent_default();
                                    add_item(());
                                }
                            }
                        } />
                    <button type="button" on:click={let add_item = add_item.clone(); move |_| add_item(())}>"Add"</button>
                </div>
            </section>
        </div>
    }
}
