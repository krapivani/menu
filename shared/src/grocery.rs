use crate::models::{Base, Ingredient, Recipe, RefType};
use std::collections::HashMap;

/// One consolidated line on the grocery list.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct GroceryLine {
    pub name: String,
    pub quantity: f64,
    pub unit: String,
    pub category: String,
    pub aisle: String,
    pub checked: bool,
}

/// Build a consolidated grocery list from a set of selected recipes.
///
/// * `expand_bases` - when `true` (the default UI behavior), bases are
///   expanded into their member ingredients, scaled by the quantity of the
///   base the recipe called for, and merged with any individually-listed
///   ingredients of the same name and unit. When `false`, bases are kept
///   as single line items (for shoppers who buy pre-made pastes/blends).
///
/// Lines are merged only when both the (case-insensitive) name and unit
/// match; incompatible units for the same ingredient are kept on separate
/// lines rather than silently summed.
pub fn build_grocery_list(
    recipes: &[Recipe],
    ingredients: &HashMap<i64, Ingredient>,
    bases: &HashMap<i64, Base>,
    expand_bases: bool,
) -> Vec<GroceryLine> {
    // Keyed by (lowercase name, lowercase unit) to merge compatible lines.
    let mut merged: HashMap<(String, String), GroceryLine> = HashMap::new();

    let mut upsert = |name: &str, quantity: f64, unit: &str, category: &str, aisle: &str| {
        let key = (name.to_lowercase(), unit.to_lowercase());
        merged
            .entry(key)
            .and_modify(|line| line.quantity += quantity)
            .or_insert_with(|| GroceryLine {
                name: name.to_string(),
                quantity,
                unit: unit.to_string(),
                category: category.to_string(),
                aisle: aisle.to_string(),
                checked: false,
            });
    };

    for recipe in recipes {
        for item in &recipe.items {
            match item.ref_type {
                RefType::Ingredient => {
                    if let Some(ing) = ingredients.get(&item.ref_id) {
                        upsert(
                            &ing.name,
                            item.quantity,
                            &item.unit,
                            &ing.category,
                            &ing.aisle,
                        );
                    }
                }
                RefType::Base => {
                    if let Some(base) = bases.get(&item.ref_id) {
                        if expand_bases {
                            for member in &base.members {
                                if let Some(ing) = ingredients.get(&member.ingredient_id) {
                                    let scaled = member.quantity * item.quantity;
                                    upsert(
                                        &ing.name,
                                        scaled,
                                        &member.unit,
                                        &ing.category,
                                        &ing.aisle,
                                    );
                                }
                            }
                        } else {
                            upsert(&base.name, item.quantity, &item.unit, "base", "various");
                        }
                    }
                }
            }
        }
    }

    let mut lines: Vec<GroceryLine> = merged.into_values().collect();
    lines.sort_by(|a, b| {
        a.category
            .cmp(&b.category)
            .then(a.aisle.cmp(&b.aisle))
            .then(a.name.cmp(&b.name))
    });
    lines
}

/// A group of grocery lines sharing an aisle within a category.
pub type AisleGroup = (String, Vec<GroceryLine>);
/// A group of aisle groups sharing a category.
pub type CategoryGroup = (String, Vec<AisleGroup>);

/// Group already-built grocery lines by category, then by aisle, preserving
/// the sorted order produced by [`build_grocery_list`].
pub fn group_by_category(lines: &[GroceryLine]) -> Vec<CategoryGroup> {
    let mut categories: Vec<CategoryGroup> = Vec::new();
    for line in lines {
        let cat_entry =
            if let Some(entry) = categories.iter_mut().find(|(c, _)| c == &line.category) {
                entry
            } else {
                categories.push((line.category.clone(), Vec::new()));
                categories.last_mut().unwrap()
            };
        let aisles = &mut cat_entry.1;
        let aisle_entry = if let Some(entry) = aisles.iter_mut().find(|(a, _)| a == &line.aisle) {
            entry
        } else {
            aisles.push((line.aisle.clone(), Vec::new()));
            aisles.last_mut().unwrap()
        };
        aisle_entry.1.push(line.clone());
    }
    categories
}

/// Render a quantity without a trailing `.0`, so a list reads "2 tbsp salt"
/// rather than "2 tbsp salt".
pub fn format_quantity(quantity: f64) -> String {
    if (quantity.fract()).abs() < f64::EPSILON {
        format!("{}", quantity as i64)
    } else {
        format!("{quantity}")
    }
}

/// Render a grocery list as plain text grouped by category and aisle, for
/// pasting into a notes app or saving as a `.txt` file. `extras` are ad-hoc
/// items the user added by hand, listed under their own heading.
pub fn grocery_list_to_text(lines: &[GroceryLine], extras: &[String]) -> String {
    let mut out = String::from("Grocery list\n");
    for (category, aisles) in group_by_category(lines) {
        out.push_str(&format!("\n{category}\n"));
        for (aisle, aisle_lines) in aisles {
            out.push_str(&format!("  {aisle}\n"));
            for line in aisle_lines {
                out.push_str(&format!(
                    "    - {} {} {}\n",
                    format_quantity(line.quantity),
                    line.unit,
                    line.name
                ));
            }
        }
    }
    if !extras.is_empty() {
        out.push_str("\nOther items\n");
        for extra in extras {
            out.push_str(&format!("    - {extra}\n"));
        }
    }
    out
}
