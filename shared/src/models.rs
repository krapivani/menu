use serde::{Deserialize, Serialize};
use std::fmt;

/// A single named ingredient, e.g. "ginger paste".
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Ingredient {
    pub id: i64,
    pub name: String,
    pub category: String,
    pub default_unit: String,
    pub aisle: String,
}

/// One ingredient inside a base, with the proportion used within that base.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BaseMember {
    pub ingredient_id: i64,
    pub quantity: f64,
    pub unit: String,
}

/// A named building block of ingredients that are almost always used
/// together, e.g. "ginger-garlic-chilli paste".
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Base {
    pub id: i64,
    pub name: String,
    pub description: String,
    pub members: Vec<BaseMember>,
}

/// Whether a recipe line references a single ingredient or a whole base.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RefType {
    Ingredient,
    /// `alias` keeps JSON exported before the cluster->base rename readable.
    #[serde(alias = "cluster")]
    Base,
}

/// A single line in a recipe: either an ingredient or a base, with quantity/unit.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RecipeItem {
    pub ref_type: RefType,
    pub ref_id: i64,
    pub quantity: f64,
    pub unit: String,
}

/// A recipe: a name, some tags for filtering, instructions, servings, and a list
/// of ingredient/base references with quantities.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Recipe {
    pub id: i64,
    pub name: String,
    #[serde(default)]
    pub role: RecipeRole,
    #[serde(default = "default_cuisine")]
    pub cuisine: String,
    #[serde(default)]
    pub treat: bool,
    pub tags: Vec<String>,
    pub instructions: String,
    pub servings: i32,
    /// Unix epoch milliseconds of the last time this recipe was used in a plan.
    pub last_used: Option<i64>,
    pub items: Vec<RecipeItem>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RecipeRole {
    Dal,
    Kadhi,
    Rice,
    Sabji,
    Roti,
    #[default]
    OnePot,
}

impl RecipeRole {
    pub const ALL: [RecipeRole; 6] = [
        RecipeRole::Dal,
        RecipeRole::Kadhi,
        RecipeRole::Rice,
        RecipeRole::Sabji,
        RecipeRole::Roti,
        RecipeRole::OnePot,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            RecipeRole::Dal => "dal",
            RecipeRole::Kadhi => "kadhi",
            RecipeRole::Rice => "rice",
            RecipeRole::Sabji => "sabji",
            RecipeRole::Roti => "roti",
            RecipeRole::OnePot => "one_pot",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            RecipeRole::Dal => "dal",
            RecipeRole::Kadhi => "kadhi",
            RecipeRole::Rice => "rice",
            RecipeRole::Sabji => "sabji",
            RecipeRole::Roti => "roti",
            RecipeRole::OnePot => "one pot",
        }
    }
}

impl fmt::Display for RecipeRole {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl From<&str> for RecipeRole {
    fn from(value: &str) -> Self {
        match value {
            "dal" => Self::Dal,
            "kadhi" => Self::Kadhi,
            "rice" => Self::Rice,
            "sabji" => Self::Sabji,
            "roti" => Self::Roti,
            "one_pot" | "one-pot" => Self::OnePot,
            _ => Self::OnePot,
        }
    }
}

fn default_cuisine() -> String {
    "unspecified".to_string()
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlanDay {
    pub recipe_ids: Vec<i64>,
}

impl PlanDay {
    pub fn new(recipe_ids: Vec<i64>) -> Self {
        Self { recipe_ids }
    }
}

/// The whole database, used for JSON export/import.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Database {
    pub ingredients: Vec<Ingredient>,
    /// `alias` keeps JSON exported before the cluster->base rename importable.
    #[serde(alias = "clusters")]
    pub bases: Vec<Base>,
    pub recipes: Vec<Recipe>,
}
