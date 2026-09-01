pub mod grocery;
pub mod models;
pub mod rotation;

pub use grocery::{
    build_grocery_list, format_quantity, grocery_list_to_text, group_by_category, GroceryLine,
};
pub use models::Recipe;
pub use models::{
    Base, BaseMember, Database, Ingredient, PlanDay, RecipeItem, RecipeRole, RefType,
};
pub use rotation::{generate_rotation, reroll_day, RotationError};

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn recipe(
        id: i64,
        name: &str,
        role: RecipeRole,
        cuisine: &str,
        treat: bool,
        tags: &[&str],
        last_used: Option<i64>,
    ) -> Recipe {
        Recipe {
            id,
            name: name.to_string(),
            role,
            cuisine: cuisine.to_string(),
            treat,
            tags: tags.iter().map(|s| s.to_string()).collect(),
            instructions: String::new(),
            servings: 4,
            last_used,
            items: vec![],
        }
    }

    #[test]
    fn rotation_has_no_duplicates() {
        let recipes: Vec<Recipe> = (0..10)
            .map(|i| {
                recipe(
                    i,
                    &format!("r{i}"),
                    RecipeRole::OnePot,
                    "gujarati",
                    false,
                    &[],
                    None,
                )
            })
            .collect();
        let plan = generate_rotation(&recipes, 7, &[], &[], &HashMap::new(), Some(42)).unwrap();
        let ids: Vec<_> = plan.iter().flat_map(|day| day.recipe_ids.clone()).collect();
        let unique: std::collections::HashSet<_> = ids.iter().collect();
        assert_eq!(unique.len(), ids.len());
        assert_eq!(plan.len(), 7);
    }

    #[test]
    fn rotation_respects_pins() {
        let recipes: Vec<Recipe> = (0..10)
            .map(|i| {
                recipe(
                    i,
                    &format!("r{i}"),
                    RecipeRole::OnePot,
                    "gujarati",
                    false,
                    &[],
                    None,
                )
            })
            .collect();
        let mut pinned = HashMap::new();
        pinned.insert(2usize, PlanDay::new(vec![5]));
        pinned.insert(5usize, PlanDay::new(vec![1]));
        let plan = generate_rotation(&recipes, 7, &[], &[], &pinned, Some(1)).unwrap();
        assert_eq!(plan[2].recipe_ids, vec![5]);
        assert_eq!(plan[5].recipe_ids, vec![1]);
        let ids: Vec<_> = plan.iter().flat_map(|day| day.recipe_ids.clone()).collect();
        let unique: std::collections::HashSet<_> = ids.iter().collect();
        assert_eq!(unique.len(), ids.len());
    }

    #[test]
    fn rotation_respects_tag_filters() {
        let recipes = vec![
            recipe(
                1,
                "veg1",
                RecipeRole::OnePot,
                "gujarati",
                false,
                &["vegetarian"],
                None,
            ),
            recipe(
                2,
                "veg2",
                RecipeRole::OnePot,
                "gujarati",
                false,
                &["vegetarian"],
                None,
            ),
            recipe(
                3,
                "meat1",
                RecipeRole::OnePot,
                "gujarati",
                false,
                &["beef"],
                None,
            ),
        ];
        let plan = generate_rotation(
            &recipes,
            2,
            &["vegetarian".to_string()],
            &[],
            &HashMap::new(),
            Some(7),
        )
        .unwrap();
        assert!(plan
            .iter()
            .flat_map(|day| day.recipe_ids.iter())
            .all(|id| *id == 1 || *id == 2));
    }

    #[test]
    fn rotation_prefers_least_recently_used() {
        // One recipe never used, the rest used very recently: over many seeds
        // the never-used one should be picked noticeably more than the ~1/6
        // uniform baseline, thanks to the least-recently-used weighting.
        let mut recipes = vec![recipe(
            1,
            "stale",
            RecipeRole::OnePot,
            "gujarati",
            false,
            &[],
            None,
        )];
        for i in 2..=6 {
            recipes.push(recipe(
                i,
                &format!("fresh{i}"),
                RecipeRole::OnePot,
                "gujarati",
                false,
                &[],
                Some(1_000_000),
            ));
        }
        let mut stale_first = 0;
        for seed in 0..50u64 {
            let plan =
                generate_rotation(&recipes, 1, &[], &[], &HashMap::new(), Some(seed)).unwrap();
            if plan[0].recipe_ids == vec![1] {
                stale_first += 1;
            }
        }
        assert!(
            stale_first > 10,
            "expected LRU bias above the ~8/50 uniform baseline, got {stale_first}/50"
        );
    }

    #[test]
    fn rotation_errors_when_not_enough_recipes() {
        let recipes = vec![recipe(
            1,
            "only",
            RecipeRole::Dal,
            "gujarati",
            false,
            &[],
            None,
        )];
        let err = generate_rotation(&recipes, 3, &[], &[], &HashMap::new(), Some(1)).unwrap_err();
        assert!(matches!(err, RotationError::NotEnoughRecipes { .. }));
    }

    #[test]
    fn rotation_assembles_combo_days_and_rotates_sabji() {
        let recipes = vec![
            recipe(1, "dal", RecipeRole::Dal, "gujarati", false, &[], None),
            recipe(2, "rice", RecipeRole::Rice, "gujarati", false, &[], None),
            recipe(
                3,
                "shaak 1",
                RecipeRole::Sabji,
                "gujarati",
                false,
                &[],
                None,
            ),
            recipe(
                4,
                "shaak 2",
                RecipeRole::Sabji,
                "gujarati",
                false,
                &[],
                None,
            ),
        ];
        let plan = generate_rotation(&recipes, 2, &[], &[], &HashMap::new(), Some(1)).unwrap();
        let sabjis: Vec<_> = plan
            .iter()
            .flat_map(|day| day.recipe_ids.iter())
            .filter(|id| **id == 3 || **id == 4)
            .copied()
            .collect();
        assert_eq!(sabjis.len(), 2);
        assert_ne!(sabjis[0], sabjis[1]);
    }

    #[test]
    fn rotation_caps_treats_per_week() {
        let recipes = vec![
            recipe(
                1,
                "khichdi",
                RecipeRole::OnePot,
                "gujarati",
                false,
                &[],
                None,
            ),
            recipe(
                2,
                "pani puri",
                RecipeRole::OnePot,
                "gujarati",
                true,
                &[],
                None,
            ),
            recipe(
                3,
                "sandwich",
                RecipeRole::OnePot,
                "gujarati",
                true,
                &[],
                None,
            ),
        ];
        let plan = generate_rotation(&recipes, 7, &[], &[], &HashMap::new(), Some(5)).unwrap();
        let treats = plan
            .iter()
            .flat_map(|day| day.recipe_ids.iter())
            .filter(|id| **id == 2 || **id == 3)
            .count();
        assert!(treats <= 1, "{plan:?}");
    }

    #[test]
    fn cuisine_filter_limits_generated_days() {
        let recipes = vec![
            recipe(
                1,
                "khichdi",
                RecipeRole::OnePot,
                "gujarati",
                false,
                &[],
                None,
            ),
            recipe(2, "pasta", RecipeRole::OnePot, "italian", false, &[], None),
        ];
        let plan = generate_rotation(
            &recipes,
            1,
            &[],
            &["italian".to_string()],
            &HashMap::new(),
            Some(5),
        )
        .unwrap();
        assert_eq!(plan[0].recipe_ids, vec![2]);
    }

    fn ingredient(id: i64, name: &str, category: &str, aisle: &str) -> Ingredient {
        Ingredient {
            id,
            name: name.to_string(),
            category: category.to_string(),
            default_unit: "tsp".to_string(),
            aisle: aisle.to_string(),
        }
    }

    #[test]
    fn base_expansion_scales_member_quantities() {
        let ginger = ingredient(1, "ginger paste", "spice", "produce");
        let garlic = ingredient(2, "garlic paste", "spice", "produce");
        let mut ingredients = HashMap::new();
        ingredients.insert(1, ginger);
        ingredients.insert(2, garlic);

        let base = Base {
            id: 100,
            name: "ginger-garlic-chilli paste".to_string(),
            description: "".to_string(),
            members: vec![
                BaseMember {
                    ingredient_id: 1,
                    quantity: 1.0,
                    unit: "tbsp".to_string(),
                },
                BaseMember {
                    ingredient_id: 2,
                    quantity: 1.0,
                    unit: "tbsp".to_string(),
                },
            ],
        };
        let mut bases = HashMap::new();
        bases.insert(100, base);

        let recipe = Recipe {
            id: 1,
            name: "curry".to_string(),
            role: RecipeRole::OnePot,
            cuisine: "gujarati".to_string(),
            treat: false,
            tags: vec![],
            instructions: String::new(),
            servings: 4,
            last_used: None,
            items: vec![RecipeItem {
                ref_type: RefType::Base,
                ref_id: 100,
                quantity: 2.0,
                unit: "batch".to_string(),
            }],
        };

        let lines = build_grocery_list(&[recipe], &ingredients, &bases, true);
        let ginger_line = lines.iter().find(|l| l.name == "ginger paste").unwrap();
        assert_eq!(ginger_line.quantity, 2.0);
        assert_eq!(ginger_line.unit, "tbsp");
    }

    #[test]
    fn expanded_base_members_merge_with_individual_ingredients() {
        let ginger = ingredient(1, "ginger paste", "spice", "produce");
        let mut ingredients = HashMap::new();
        ingredients.insert(1, ginger);

        let base = Base {
            id: 100,
            name: "ginger-garlic-chilli paste".to_string(),
            description: "".to_string(),
            members: vec![BaseMember {
                ingredient_id: 1,
                quantity: 1.0,
                unit: "tbsp".to_string(),
            }],
        };
        let mut bases = HashMap::new();
        bases.insert(100, base);

        let recipe_with_base = Recipe {
            id: 1,
            name: "curry".to_string(),
            role: RecipeRole::OnePot,
            cuisine: "gujarati".to_string(),
            treat: false,
            tags: vec![],
            instructions: String::new(),
            servings: 4,
            last_used: None,
            items: vec![RecipeItem {
                ref_type: RefType::Base,
                ref_id: 100,
                quantity: 1.0,
                unit: "batch".to_string(),
            }],
        };
        let recipe_with_plain = Recipe {
            id: 2,
            name: "soup".to_string(),
            role: RecipeRole::OnePot,
            cuisine: "gujarati".to_string(),
            treat: false,
            tags: vec![],
            instructions: String::new(),
            servings: 2,
            last_used: None,
            items: vec![RecipeItem {
                ref_type: RefType::Ingredient,
                ref_id: 1,
                quantity: 2.0,
                unit: "tbsp".to_string(),
            }],
        };

        let lines = build_grocery_list(
            &[recipe_with_base, recipe_with_plain],
            &ingredients,
            &bases,
            true,
        );
        let ginger_lines: Vec<_> = lines.iter().filter(|l| l.name == "ginger paste").collect();
        assert_eq!(ginger_lines.len(), 1, "expected a single merged line");
        assert_eq!(ginger_lines[0].quantity, 3.0);
    }

    #[test]
    fn incompatible_units_stay_separate() {
        let ginger = ingredient(1, "ginger paste", "spice", "produce");
        let ingredients: HashMap<i64, Ingredient> = [(1, ginger)].into_iter().collect();
        let bases = HashMap::new();

        let recipe_tbsp = Recipe {
            id: 1,
            name: "curry".to_string(),
            role: RecipeRole::OnePot,
            cuisine: "gujarati".to_string(),
            treat: false,
            tags: vec![],
            instructions: String::new(),
            servings: 4,
            last_used: None,
            items: vec![RecipeItem {
                ref_type: RefType::Ingredient,
                ref_id: 1,
                quantity: 1.0,
                unit: "tbsp".to_string(),
            }],
        };
        let recipe_g = Recipe {
            id: 2,
            name: "soup".to_string(),
            role: RecipeRole::OnePot,
            cuisine: "gujarati".to_string(),
            treat: false,
            tags: vec![],
            instructions: String::new(),
            servings: 2,
            last_used: None,
            items: vec![RecipeItem {
                ref_type: RefType::Ingredient,
                ref_id: 1,
                quantity: 10.0,
                unit: "g".to_string(),
            }],
        };

        let lines = build_grocery_list(&[recipe_tbsp, recipe_g], &ingredients, &bases, true);
        let ginger_lines: Vec<_> = lines.iter().filter(|l| l.name == "ginger paste").collect();
        assert_eq!(
            ginger_lines.len(),
            2,
            "incompatible units must stay separate"
        );
    }

    #[test]
    fn grocery_text_export_groups_and_lists_extras() {
        let lines = vec![
            GroceryLine {
                name: "ginger paste".to_string(),
                quantity: 2.0,
                unit: "tbsp".to_string(),
                category: "spice".to_string(),
                aisle: "produce".to_string(),
                checked: false,
            },
            GroceryLine {
                name: "yogurt".to_string(),
                quantity: 1.5,
                unit: "cup".to_string(),
                category: "dairy".to_string(),
                aisle: "dairy".to_string(),
                checked: false,
            },
        ];
        let text = grocery_list_to_text(&lines, &["bin bags".to_string()]);
        assert!(text.contains("spice"));
        assert!(text.contains("- 2 tbsp ginger paste"), "{text}");
        assert!(text.contains("- 1.5 cup yogurt"), "{text}");
        assert!(text.contains("Other items"));
        assert!(text.contains("- bin bags"));
    }

    #[test]
    fn legacy_cluster_json_still_imports_as_bases() {
        let json = r#"{
            "ingredients": [],
            "clusters": [
                {"id": 1, "name": "tadka base", "description": "", "members": []}
            ],
            "recipes": [
                {"id": 1, "name": "kadhi", "tags": [], "instructions": "", "servings": 4,
                 "last_used": null,
                 "items": [{"ref_type": "cluster", "ref_id": 1, "quantity": 1.0, "unit": "batch"}]}
            ]
        }"#;
        let db: Database = serde_json::from_str(json).unwrap();
        assert_eq!(db.bases.len(), 1);
        assert_eq!(db.bases[0].name, "tadka base");
        assert_eq!(db.recipes[0].items[0].ref_type, RefType::Base);
        assert_eq!(db.recipes[0].role, RecipeRole::OnePot);
        assert_eq!(db.recipes[0].cuisine, "unspecified");
        assert!(!db.recipes[0].treat);
    }

    #[test]
    fn unexpanded_base_is_single_line() {
        let ginger = ingredient(1, "ginger paste", "spice", "produce");
        let ingredients: HashMap<i64, Ingredient> = [(1, ginger)].into_iter().collect();
        let base = Base {
            id: 100,
            name: "ginger-garlic-chilli paste".to_string(),
            description: "".to_string(),
            members: vec![BaseMember {
                ingredient_id: 1,
                quantity: 1.0,
                unit: "tbsp".to_string(),
            }],
        };
        let bases: HashMap<i64, Base> = [(100, base)].into_iter().collect();

        let recipe = Recipe {
            id: 1,
            name: "curry".to_string(),
            role: RecipeRole::OnePot,
            cuisine: "gujarati".to_string(),
            treat: false,
            tags: vec![],
            instructions: String::new(),
            servings: 4,
            last_used: None,
            items: vec![RecipeItem {
                ref_type: RefType::Base,
                ref_id: 100,
                quantity: 1.0,
                unit: "batch".to_string(),
            }],
        };

        let lines = build_grocery_list(&[recipe], &ingredients, &bases, false);
        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0].name, "ginger-garlic-chilli paste");
    }
}
