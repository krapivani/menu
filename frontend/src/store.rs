//! The `Store` trait is the seam between the app's UI/business logic and its
//! persistence backend. [`SqliteWasmStore`] is the only implementation today
//! (SQLite compiled to WASM, running entirely in the browser), but the trait
//! is written so a future `TursoStore` (hosted libSQL, for multi-device sync)
//! can be swapped in without touching any UI code.

use crate::db::{Conn, DbError, Value};
use async_trait::async_trait;
use shared::models::{Base, BaseMember, Database, Ingredient, RecipeItem, RecipeRole, RefType};
use shared::Recipe;
use std::cell::RefCell;
use std::rc::Rc;

const MIGRATION_SCHEMA: &str = include_str!("../../migrations/0001_initial_schema.sql");
const MIGRATION_SEED: &str = include_str!("../../migrations/0002_seed_data.sql");
const MIGRATION_RENAME_BASES: &str =
    include_str!("../../migrations/0003_rename_clusters_to_bases.sql");
const MIGRATION_ROLES_COMBOS: &str =
    include_str!("../../migrations/0004_recipe_roles_combos_cuisine_treats.sql");
const LOCAL_STORAGE_KEY: &str = "menu-db-v1";

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum StoreError {
    #[error("database error: {0}")]
    Db(String),
    #[error("(de)serialization error: {0}")]
    Serde(String),
}

impl From<DbError> for StoreError {
    fn from(e: DbError) -> Self {
        StoreError::Db(e.0)
    }
}
impl From<serde_json::Error> for StoreError {
    fn from(e: serde_json::Error) -> Self {
        StoreError::Serde(e.to_string())
    }
}

/// Async CRUD + query surface used by every screen. Implemented today by
/// [`SqliteWasmStore`]; a `TursoStore` speaking the same trait over HTTP is
/// the intended future swap-in.
#[async_trait(?Send)]
pub trait Store {
    async fn load_database(&self) -> Result<Database, StoreError>;

    /// Insert or update. Implementations treat `id == 0` as "not yet
    /// persisted" (SQLite `AUTOINCREMENT` ids start at 1, so this is safe in
    /// practice, but a JSON import that supplies a crafted `id: 0` would be
    /// mistaken for a new record rather than updating an existing one).
    async fn save_ingredient(&self, ingredient: Ingredient) -> Result<Ingredient, StoreError>;
    async fn delete_ingredient(&self, id: i64) -> Result<(), StoreError>;

    async fn save_base(&self, base: Base) -> Result<Base, StoreError>;
    async fn delete_base(&self, id: i64) -> Result<(), StoreError>;

    async fn save_recipe(&self, recipe: Recipe) -> Result<Recipe, StoreError>;
    async fn delete_recipe(&self, id: i64) -> Result<(), StoreError>;
    async fn touch_recipe_last_used(&self, id: i64, timestamp_ms: i64) -> Result<(), StoreError>;

    async fn export_json(&self) -> Result<String, StoreError>;
    async fn import_json(&self, json: &str) -> Result<(), StoreError>;
}

/// SQLite-in-WASM implementation. Persists a JSON snapshot to `localStorage`
/// after every mutation so data survives a page reload; see the crate-level
/// README for the OPFS upgrade path and its durability trade-offs.
pub struct SqliteWasmStore {
    conn: Rc<RefCell<Conn>>,
}

// SAFETY: this app only targets `wasm32-unknown-unknown`, which is
// single-threaded (no real OS threads); Leptos' reactive system requires
// `Send + Sync` bounds even in CSR-only builds, so we assert them here.
#[allow(unsafe_code)]
unsafe impl Send for SqliteWasmStore {}
#[allow(unsafe_code)]
unsafe impl Sync for SqliteWasmStore {}

impl SqliteWasmStore {
    pub fn new() -> Result<Self, StoreError> {
        let conn = Conn::open_memory()?;

        // Migrations are replayed in numeric order against the fresh
        // in-memory database. The seed only runs when there is no snapshot to
        // restore, and sits between 0001 and 0003 exactly as it would against
        // a persistent database, so the rename migration converts it too.
        conn.execute_batch(MIGRATION_SCHEMA)?;
        let stored = local_storage_get(LOCAL_STORAGE_KEY);
        if stored.is_none() {
            conn.execute_batch(MIGRATION_SEED)?;
        }
        conn.execute_batch(MIGRATION_RENAME_BASES)?;
        conn.execute_batch(MIGRATION_ROLES_COMBOS)?;

        if let Some(json) = stored {
            let db: Database = serde_json::from_str(&json)?;
            restore_database(&conn, &db)?;
        }

        Ok(Self {
            conn: Rc::new(RefCell::new(conn)),
        })
    }

    fn persist(&self) -> Result<(), StoreError> {
        let db = read_database(&self.conn.borrow())?;
        let json = serde_json::to_string(&db)?;
        local_storage_set(LOCAL_STORAGE_KEY, &json);
        Ok(())
    }

    /// Synchronous read used once at startup to seed the initial reactive
    /// signal before any UI is mounted (no real I/O happens here, so there's
    /// nothing to `.await`).
    pub fn load_database_sync(&self) -> Database {
        read_database(&self.conn.borrow()).unwrap_or_default()
    }
}

fn read_database(conn: &Conn) -> Result<Database, StoreError> {
    let ingredients = read_ingredients(conn)?;
    let bases = read_bases(conn)?;
    let recipes = read_recipes(conn)?;
    Ok(Database {
        ingredients,
        bases,
        recipes,
    })
}

fn read_ingredients(conn: &Conn) -> Result<Vec<Ingredient>, StoreError> {
    let rows = conn.query(
        "SELECT id, name, category, default_unit, aisle FROM ingredients ORDER BY name",
        &[],
    )?;
    Ok(rows
        .into_iter()
        .map(|r| Ingredient {
            id: r[0].as_i64(),
            name: r[1].as_str(),
            category: r[2].as_str(),
            default_unit: r[3].as_str(),
            aisle: r[4].as_str(),
        })
        .collect())
}

fn read_bases(conn: &Conn) -> Result<Vec<Base>, StoreError> {
    let rows = conn.query("SELECT id, name, description FROM bases ORDER BY name", &[])?;
    let mut bases: Vec<Base> = rows
        .into_iter()
        .map(|r| Base {
            id: r[0].as_i64(),
            name: r[1].as_str(),
            description: r[2].as_str(),
            members: vec![],
        })
        .collect();
    for base in &mut bases {
        let member_rows = conn.query(
            "SELECT ingredient_id, quantity, unit FROM base_members WHERE base_id = ?1",
            &[Value::Int(base.id)],
        )?;
        base.members = member_rows
            .into_iter()
            .map(|r| BaseMember {
                ingredient_id: r[0].as_i64(),
                quantity: r[1].as_f64(),
                unit: r[2].as_str(),
            })
            .collect();
    }
    Ok(bases)
}

fn read_recipes(conn: &Conn) -> Result<Vec<Recipe>, StoreError> {
    let rows = conn.query(
        "SELECT id, name, role, cuisine, treat, tags, instructions, servings, last_used FROM recipes ORDER BY name",
        &[],
    )?;
    let mut recipes: Vec<Recipe> = rows
        .into_iter()
        .map(|r| Recipe {
            id: r[0].as_i64(),
            name: r[1].as_str(),
            role: RecipeRole::from(r[2].as_str().as_str()),
            cuisine: r[3].as_str(),
            treat: r[4].as_i64() != 0,
            tags: split_tags(&r[5].as_str()),
            instructions: r[6].as_str(),
            servings: r[7].as_i64() as i32,
            last_used: r[8].as_opt_i64(),
            items: vec![],
        })
        .collect();
    for recipe in &mut recipes {
        let item_rows = conn.query(
            "SELECT ref_type, ref_id, quantity, unit FROM recipe_items WHERE recipe_id = ?1",
            &[Value::Int(recipe.id)],
        )?;
        recipe.items = item_rows
            .into_iter()
            .map(|r| RecipeItem {
                ref_type: if r[0].as_str() == "base" {
                    RefType::Base
                } else {
                    RefType::Ingredient
                },
                ref_id: r[1].as_i64(),
                quantity: r[2].as_f64(),
                unit: r[3].as_str(),
            })
            .collect();
    }
    Ok(recipes)
}

fn split_tags(tags: &str) -> Vec<String> {
    tags.split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

fn join_tags(tags: &[String]) -> String {
    tags.join(",")
}

/// Wipe and reload every table from a JSON snapshot (used to restore the
/// localStorage fallback and for JSON import).
fn restore_database(conn: &Conn, db: &Database) -> Result<(), StoreError> {
    conn.execute_batch(
        "DELETE FROM recipe_items; DELETE FROM recipes; \
         DELETE FROM base_members; DELETE FROM bases; \
         DELETE FROM ingredients;",
    )?;

    for ing in &db.ingredients {
        conn.execute(
            "INSERT INTO ingredients (id, name, category, default_unit, aisle) VALUES (?1, ?2, ?3, ?4, ?5)",
            &[
                Value::Int(ing.id),
                Value::Text(ing.name.clone()),
                Value::Text(ing.category.clone()),
                Value::Text(ing.default_unit.clone()),
                Value::Text(ing.aisle.clone()),
            ],
        )?;
    }
    for base in &db.bases {
        conn.execute(
            "INSERT INTO bases (id, name, description) VALUES (?1, ?2, ?3)",
            &[
                Value::Int(base.id),
                Value::Text(base.name.clone()),
                Value::Text(base.description.clone()),
            ],
        )?;
        for member in &base.members {
            conn.execute(
                "INSERT INTO base_members (base_id, ingredient_id, quantity, unit) VALUES (?1, ?2, ?3, ?4)",
                &[
                    Value::Int(base.id),
                    Value::Int(member.ingredient_id),
                    Value::Real(member.quantity),
                    Value::Text(member.unit.clone()),
                ],
            )?;
        }
    }
    for recipe in &db.recipes {
        conn.execute(
            "INSERT INTO recipes (id, name, role, cuisine, treat, tags, instructions, servings, last_used) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            &[
                Value::Int(recipe.id),
                Value::Text(recipe.name.clone()),
                Value::Text(recipe.role.as_str().to_string()),
                Value::Text(recipe.cuisine.clone()),
                Value::Int(if recipe.treat { 1 } else { 0 }),
                Value::Text(join_tags(&recipe.tags)),
                Value::Text(recipe.instructions.clone()),
                Value::Int(recipe.servings as i64),
                Value::from(recipe.last_used),
            ],
        )?;
        for item in &recipe.items {
            conn.execute(
                "INSERT INTO recipe_items (recipe_id, ref_type, ref_id, quantity, unit) VALUES (?1, ?2, ?3, ?4, ?5)",
                &[
                    Value::Int(recipe.id),
                    Value::Text(match item.ref_type {
                        RefType::Ingredient => "ingredient".to_string(),
                        RefType::Base => "base".to_string(),
                    }),
                    Value::Int(item.ref_id),
                    Value::Real(item.quantity),
                    Value::Text(item.unit.clone()),
                ],
            )?;
        }
    }
    Ok(())
}

#[async_trait(?Send)]
impl Store for SqliteWasmStore {
    async fn load_database(&self) -> Result<Database, StoreError> {
        read_database(&self.conn.borrow())
    }

    async fn save_ingredient(&self, ingredient: Ingredient) -> Result<Ingredient, StoreError> {
        let conn = self.conn.borrow();
        let id = if ingredient.id == 0 {
            conn.insert(
                "INSERT INTO ingredients (name, category, default_unit, aisle) VALUES (?1, ?2, ?3, ?4)",
                &[
                    Value::Text(ingredient.name.clone()),
                    Value::Text(ingredient.category.clone()),
                    Value::Text(ingredient.default_unit.clone()),
                    Value::Text(ingredient.aisle.clone()),
                ],
            )?
        } else {
            conn.execute(
                "UPDATE ingredients SET name = ?2, category = ?3, default_unit = ?4, aisle = ?5 WHERE id = ?1",
                &[
                    Value::Int(ingredient.id),
                    Value::Text(ingredient.name.clone()),
                    Value::Text(ingredient.category.clone()),
                    Value::Text(ingredient.default_unit.clone()),
                    Value::Text(ingredient.aisle.clone()),
                ],
            )?;
            ingredient.id
        };
        drop(conn);
        self.persist()?;
        Ok(Ingredient { id, ..ingredient })
    }

    async fn delete_ingredient(&self, id: i64) -> Result<(), StoreError> {
        self.conn
            .borrow()
            .execute("DELETE FROM ingredients WHERE id = ?1", &[Value::Int(id)])?;
        self.persist()
    }

    async fn save_base(&self, base: Base) -> Result<Base, StoreError> {
        let conn = self.conn.borrow();
        let id = if base.id == 0 {
            conn.insert(
                "INSERT INTO bases (name, description) VALUES (?1, ?2)",
                &[
                    Value::Text(base.name.clone()),
                    Value::Text(base.description.clone()),
                ],
            )?
        } else {
            conn.execute(
                "UPDATE bases SET name = ?2, description = ?3 WHERE id = ?1",
                &[
                    Value::Int(base.id),
                    Value::Text(base.name.clone()),
                    Value::Text(base.description.clone()),
                ],
            )?;
            base.id
        };
        conn.execute(
            "DELETE FROM base_members WHERE base_id = ?1",
            &[Value::Int(id)],
        )?;
        for member in &base.members {
            conn.execute(
                "INSERT INTO base_members (base_id, ingredient_id, quantity, unit) VALUES (?1, ?2, ?3, ?4)",
                &[
                    Value::Int(id),
                    Value::Int(member.ingredient_id),
                    Value::Real(member.quantity),
                    Value::Text(member.unit.clone()),
                ],
            )?;
        }
        drop(conn);
        self.persist()?;
        Ok(Base { id, ..base })
    }

    async fn delete_base(&self, id: i64) -> Result<(), StoreError> {
        self.conn
            .borrow()
            .execute("DELETE FROM bases WHERE id = ?1", &[Value::Int(id)])?;
        self.persist()
    }

    async fn save_recipe(&self, recipe: Recipe) -> Result<Recipe, StoreError> {
        let conn = self.conn.borrow();
        let id = if recipe.id == 0 {
            conn.insert(
                "INSERT INTO recipes (name, role, cuisine, treat, tags, instructions, servings, last_used) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                &[
                    Value::Text(recipe.name.clone()),
                    Value::Text(recipe.role.as_str().to_string()),
                    Value::Text(recipe.cuisine.clone()),
                    Value::Int(if recipe.treat { 1 } else { 0 }),
                    Value::Text(join_tags(&recipe.tags)),
                    Value::Text(recipe.instructions.clone()),
                    Value::Int(recipe.servings as i64),
                    Value::from(recipe.last_used),
                ],
            )?
        } else {
            conn.execute(
                "UPDATE recipes SET name = ?2, role = ?3, cuisine = ?4, treat = ?5, tags = ?6, instructions = ?7, servings = ?8, last_used = ?9 WHERE id = ?1",
                &[
                    Value::Int(recipe.id),
                    Value::Text(recipe.name.clone()),
                    Value::Text(recipe.role.as_str().to_string()),
                    Value::Text(recipe.cuisine.clone()),
                    Value::Int(if recipe.treat { 1 } else { 0 }),
                    Value::Text(join_tags(&recipe.tags)),
                    Value::Text(recipe.instructions.clone()),
                    Value::Int(recipe.servings as i64),
                    Value::from(recipe.last_used),
                ],
            )?;
            recipe.id
        };
        conn.execute(
            "DELETE FROM recipe_items WHERE recipe_id = ?1",
            &[Value::Int(id)],
        )?;
        for item in &recipe.items {
            conn.execute(
                "INSERT INTO recipe_items (recipe_id, ref_type, ref_id, quantity, unit) VALUES (?1, ?2, ?3, ?4, ?5)",
                &[
                    Value::Int(id),
                    Value::Text(match item.ref_type {
                        RefType::Ingredient => "ingredient".to_string(),
                        RefType::Base => "base".to_string(),
                    }),
                    Value::Int(item.ref_id),
                    Value::Real(item.quantity),
                    Value::Text(item.unit.clone()),
                ],
            )?;
        }
        drop(conn);
        self.persist()?;
        Ok(Recipe { id, ..recipe })
    }

    async fn delete_recipe(&self, id: i64) -> Result<(), StoreError> {
        self.conn
            .borrow()
            .execute("DELETE FROM recipes WHERE id = ?1", &[Value::Int(id)])?;
        self.persist()
    }

    async fn touch_recipe_last_used(&self, id: i64, timestamp_ms: i64) -> Result<(), StoreError> {
        self.conn.borrow().execute(
            "UPDATE recipes SET last_used = ?2 WHERE id = ?1",
            &[Value::Int(id), Value::Int(timestamp_ms)],
        )?;
        self.persist()
    }

    async fn export_json(&self) -> Result<String, StoreError> {
        let db = read_database(&self.conn.borrow())?;
        Ok(serde_json::to_string_pretty(&db)?)
    }

    async fn import_json(&self, json: &str) -> Result<(), StoreError> {
        let db: Database = serde_json::from_str(json)?;
        restore_database(&self.conn.borrow(), &db)?;
        self.persist()
    }
}

/// Read a key from `window.localStorage`, if available.
pub fn local_storage_get(key: &str) -> Option<String> {
    let window = web_sys::window()?;
    let storage = window.local_storage().ok()??;
    storage.get_item(key).ok()?
}

/// Best-effort write to `window.localStorage`. Silently ignored if
/// unavailable (e.g. privacy mode) — data still lives for the session in the
/// in-memory database.
pub fn local_storage_set(key: &str, value: &str) {
    if let Some(window) = web_sys::window() {
        if let Ok(Some(storage)) = window.local_storage() {
            let _ = storage.set_item(key, value);
        }
    }
}
