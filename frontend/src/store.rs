//! The `Store` trait is the seam between the app's UI/business logic and its
//! persistence backend. [`SqliteWasmStore`] is the only implementation today
//! (SQLite compiled to WASM, running entirely in the browser), but the trait
//! is written so a future `TursoStore` (hosted libSQL, for multi-device sync)
//! can be swapped in without touching any UI code.

use crate::db::{Conn, DbError, Value};
use async_trait::async_trait;
use shared::models::{ClusterMember, Database, Ingredient, IngredientCluster, RecipeItem, RefType};
use shared::Recipe;
use std::cell::RefCell;
use std::rc::Rc;

const MIGRATION_SCHEMA: &str = include_str!("../../migrations/0001_initial_schema.sql");
const MIGRATION_SEED: &str = include_str!("../../migrations/0002_seed_data.sql");
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

    async fn save_ingredient(&self, ingredient: Ingredient) -> Result<Ingredient, StoreError>;
    async fn delete_ingredient(&self, id: i64) -> Result<(), StoreError>;

    async fn save_cluster(
        &self,
        cluster: IngredientCluster,
    ) -> Result<IngredientCluster, StoreError>;
    async fn delete_cluster(&self, id: i64) -> Result<(), StoreError>;

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
        conn.execute_batch(MIGRATION_SCHEMA)?;

        if let Some(json) = local_storage_get(LOCAL_STORAGE_KEY) {
            let db: Database = serde_json::from_str(&json)?;
            restore_database(&conn, &db)?;
        } else {
            conn.execute_batch(MIGRATION_SEED)?;
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
    let clusters = read_clusters(conn)?;
    let recipes = read_recipes(conn)?;
    Ok(Database {
        ingredients,
        clusters,
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

fn read_clusters(conn: &Conn) -> Result<Vec<IngredientCluster>, StoreError> {
    let rows = conn.query(
        "SELECT id, name, description FROM ingredient_clusters ORDER BY name",
        &[],
    )?;
    let mut clusters: Vec<IngredientCluster> = rows
        .into_iter()
        .map(|r| IngredientCluster {
            id: r[0].as_i64(),
            name: r[1].as_str(),
            description: r[2].as_str(),
            members: vec![],
        })
        .collect();
    for cluster in &mut clusters {
        let member_rows = conn.query(
            "SELECT ingredient_id, quantity, unit FROM cluster_members WHERE cluster_id = ?1",
            &[Value::Int(cluster.id)],
        )?;
        cluster.members = member_rows
            .into_iter()
            .map(|r| ClusterMember {
                ingredient_id: r[0].as_i64(),
                quantity: r[1].as_f64(),
                unit: r[2].as_str(),
            })
            .collect();
    }
    Ok(clusters)
}

fn read_recipes(conn: &Conn) -> Result<Vec<Recipe>, StoreError> {
    let rows = conn.query(
        "SELECT id, name, tags, instructions, servings, last_used FROM recipes ORDER BY name",
        &[],
    )?;
    let mut recipes: Vec<Recipe> = rows
        .into_iter()
        .map(|r| Recipe {
            id: r[0].as_i64(),
            name: r[1].as_str(),
            tags: split_tags(&r[2].as_str()),
            instructions: r[3].as_str(),
            servings: r[4].as_i64() as i32,
            last_used: r[5].as_opt_i64(),
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
                ref_type: if r[0].as_str() == "cluster" {
                    RefType::Cluster
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
         DELETE FROM cluster_members; DELETE FROM ingredient_clusters; \
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
    for cluster in &db.clusters {
        conn.execute(
            "INSERT INTO ingredient_clusters (id, name, description) VALUES (?1, ?2, ?3)",
            &[
                Value::Int(cluster.id),
                Value::Text(cluster.name.clone()),
                Value::Text(cluster.description.clone()),
            ],
        )?;
        for member in &cluster.members {
            conn.execute(
                "INSERT INTO cluster_members (cluster_id, ingredient_id, quantity, unit) VALUES (?1, ?2, ?3, ?4)",
                &[
                    Value::Int(cluster.id),
                    Value::Int(member.ingredient_id),
                    Value::Real(member.quantity),
                    Value::Text(member.unit.clone()),
                ],
            )?;
        }
    }
    for recipe in &db.recipes {
        conn.execute(
            "INSERT INTO recipes (id, name, tags, instructions, servings, last_used) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            &[
                Value::Int(recipe.id),
                Value::Text(recipe.name.clone()),
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
                        RefType::Cluster => "cluster".to_string(),
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

    async fn save_cluster(
        &self,
        cluster: IngredientCluster,
    ) -> Result<IngredientCluster, StoreError> {
        let conn = self.conn.borrow();
        let id = if cluster.id == 0 {
            conn.insert(
                "INSERT INTO ingredient_clusters (name, description) VALUES (?1, ?2)",
                &[
                    Value::Text(cluster.name.clone()),
                    Value::Text(cluster.description.clone()),
                ],
            )?
        } else {
            conn.execute(
                "UPDATE ingredient_clusters SET name = ?2, description = ?3 WHERE id = ?1",
                &[
                    Value::Int(cluster.id),
                    Value::Text(cluster.name.clone()),
                    Value::Text(cluster.description.clone()),
                ],
            )?;
            cluster.id
        };
        conn.execute(
            "DELETE FROM cluster_members WHERE cluster_id = ?1",
            &[Value::Int(id)],
        )?;
        for member in &cluster.members {
            conn.execute(
                "INSERT INTO cluster_members (cluster_id, ingredient_id, quantity, unit) VALUES (?1, ?2, ?3, ?4)",
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
        Ok(IngredientCluster { id, ..cluster })
    }

    async fn delete_cluster(&self, id: i64) -> Result<(), StoreError> {
        self.conn.borrow().execute(
            "DELETE FROM ingredient_clusters WHERE id = ?1",
            &[Value::Int(id)],
        )?;
        self.persist()
    }

    async fn save_recipe(&self, recipe: Recipe) -> Result<Recipe, StoreError> {
        let conn = self.conn.borrow();
        let id = if recipe.id == 0 {
            conn.insert(
                "INSERT INTO recipes (name, tags, instructions, servings, last_used) VALUES (?1, ?2, ?3, ?4, ?5)",
                &[
                    Value::Text(recipe.name.clone()),
                    Value::Text(join_tags(&recipe.tags)),
                    Value::Text(recipe.instructions.clone()),
                    Value::Int(recipe.servings as i64),
                    Value::from(recipe.last_used),
                ],
            )?
        } else {
            conn.execute(
                "UPDATE recipes SET name = ?2, tags = ?3, instructions = ?4, servings = ?5, last_used = ?6 WHERE id = ?1",
                &[
                    Value::Int(recipe.id),
                    Value::Text(recipe.name.clone()),
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
                        RefType::Cluster => "cluster".to_string(),
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
fn local_storage_get(key: &str) -> Option<String> {
    let window = web_sys::window()?;
    let storage = window.local_storage().ok()??;
    storage.get_item(key).ok()?
}

/// Best-effort write to `window.localStorage`. Silently ignored if
/// unavailable (e.g. privacy mode) — data still lives for the session in the
/// in-memory database.
fn local_storage_set(key: &str, value: &str) {
    if let Some(window) = web_sys::window() {
        if let Ok(Some(storage)) = window.local_storage() {
            let _ = storage.set_item(key, value);
        }
    }
}
