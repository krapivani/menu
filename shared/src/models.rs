use serde::{Deserialize, Serialize};

/// A single named ingredient, e.g. "ginger paste".
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Ingredient {
    pub id: i64,
    pub name: String,
    pub category: String,
    pub default_unit: String,
    pub aisle: String,
}

/// One ingredient inside a cluster, with the proportion used within that cluster.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ClusterMember {
    pub ingredient_id: i64,
    pub quantity: f64,
    pub unit: String,
}

/// A named group of ingredients that are almost always used together,
/// e.g. "ginger-garlic-chilli paste".
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IngredientCluster {
    pub id: i64,
    pub name: String,
    pub description: String,
    pub members: Vec<ClusterMember>,
}

/// Whether a recipe line references a single ingredient or a whole cluster.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RefType {
    Ingredient,
    Cluster,
}

/// A single line in a recipe: either an ingredient or a cluster, with quantity/unit.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RecipeItem {
    pub ref_type: RefType,
    pub ref_id: i64,
    pub quantity: f64,
    pub unit: String,
}

/// A recipe: a name, some tags for filtering, instructions, servings, and a list
/// of ingredient/cluster references with quantities.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Recipe {
    pub id: i64,
    pub name: String,
    pub tags: Vec<String>,
    pub instructions: String,
    pub servings: i32,
    /// Unix epoch milliseconds of the last time this recipe was used in a plan.
    pub last_used: Option<i64>,
    pub items: Vec<RecipeItem>,
}

/// The whole database, used for JSON export/import.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Database {
    pub ingredients: Vec<Ingredient>,
    pub clusters: Vec<IngredientCluster>,
    pub recipes: Vec<Recipe>,
}
