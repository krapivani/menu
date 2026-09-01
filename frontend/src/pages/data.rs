use crate::state::use_app_state;
use crate::store::Store;
use leptos::prelude::*;

#[component]
pub fn DataPage() -> impl IntoView {
    let state = use_app_state();

    let export_href = RwSignal::new(String::new());
    let import_text = RwSignal::new(String::new());
    let status = RwSignal::new(String::new());

    let state_for_export = state.clone();
    let export = move |_| {
        let state = state_for_export.clone();
        leptos::task::spawn_local(async move {
            match state.store.export_json().await {
                Ok(json) => {
                    let encoded = js_sys::encode_uri_component(&json);
                    export_href.set(format!("data:application/json;charset=utf-8,{encoded}"));
                }
                Err(e) => state.set_error(e.to_string()),
            }
        });
    };

    let state_for_import = state.clone();
    let import = move |_| {
        let state = state_for_import.clone();
        let json = import_text.get();
        leptos::task::spawn_local(async move {
            match state.store.import_json(&json).await {
                Ok(_) => {
                    state.reload();
                    status.set("Import successful.".to_string());
                }
                Err(e) => state.set_error(e.to_string()),
            }
        });
    };

    view! {
        <div class="data-page">
            <h2>"Backup & Restore"</h2>
            <p class="hint">
                "The whole database lives in this browser (localStorage / OPFS). "
                "Clearing site data will delete it, so export a backup regularly."
            </p>

            <div class="card">
                <h3>"Export"</h3>
                <button on:click=export>"Generate export file"</button>
                {move || (!export_href.get().is_empty()).then(|| {
                    let href = export_href.get();
                    view! {
                        <a href=href download="menu-export.json" class="download-link">
                            "Download menu-export.json"
                        </a>
                    }
                })}
            </div>

            <div class="card">
                <h3>"Import"</h3>
                <p class="hint">"Paste the contents of a previously exported JSON file. This replaces all current data."</p>
                <textarea rows="8"
                    prop:value=move || import_text.get()
                    on:input=move |ev| import_text.set(event_target_value(&ev))></textarea>
                <button on:click=import>"Import"</button>
                <p>{move || status.get()}</p>
            </div>
        </div>
    }
}
