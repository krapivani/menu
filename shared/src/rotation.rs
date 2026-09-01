use crate::models::{PlanDay, Recipe, RecipeRole};
use rand::rngs::SmallRng;
use rand::seq::SliceRandom;
use rand::{Rng, SeedableRng};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum RotationError {
    #[error("not enough recipes match the given filters: need {needed}, have {available}")]
    NotEnoughRecipes { needed: usize, available: usize },
    #[error("pinned recipe id {0} does not exist or does not match the filters")]
    InvalidPin(i64),
    #[error("day index {0} is out of range for a plan of length {1}")]
    DayOutOfRange(usize, usize),
    #[error("pinned treats exceed the limit of {limit} per {days} days")]
    TreatLimitExceeded { limit: usize, days: usize },
}

const PATTERNS: [&[RecipeRole]; 4] = [
    &[RecipeRole::Dal, RecipeRole::Rice, RecipeRole::Sabji],
    &[RecipeRole::Kadhi, RecipeRole::Rice, RecipeRole::Sabji],
    &[RecipeRole::Sabji, RecipeRole::Roti],
    &[RecipeRole::OnePot],
];

/// Filter recipes so that every requested tag is present on the recipe (AND
/// semantics), and at least one selected cuisine matches when cuisine filters
/// are supplied.
pub fn filter_recipes<'a>(
    recipes: &'a [Recipe],
    tags: &[String],
    cuisines: &[String],
) -> Vec<&'a Recipe> {
    recipes
        .iter()
        .filter(|r| tags.iter().all(|t| r.tags.iter().any(|rt| rt == t)))
        .filter(|r| cuisines.is_empty() || cuisines.iter().any(|c| c == &r.cuisine))
        .collect()
}

/// Generate a dinner plan. Each day is filled from one of the seeded patterns:
/// dal+rice+sabji, kadhi+rice+sabji, sabji+roti, or one_pot.
///
/// Recipe selection keeps the original behavior where possible: pinned days are
/// fixed, unpinned recipes are least-recently-used weighted, deterministic
/// seeds are supported, and recipes do not repeat until the eligible pool for a
/// role is exhausted. Sabjis receive the same no-repeat preference across the
/// whole plan. Treat recipes are hard-capped at one per seven generated days.
pub fn generate_rotation(
    recipes: &[Recipe],
    days: usize,
    tag_filters: &[String],
    cuisine_filters: &[String],
    pinned: &HashMap<usize, PlanDay>,
    seed: Option<u64>,
) -> Result<Vec<PlanDay>, RotationError> {
    for &day in pinned.keys() {
        if day >= days {
            return Err(RotationError::DayOutOfRange(day, days));
        }
    }

    let eligible = filter_recipes(recipes, tag_filters, cuisine_filters);
    let by_id: HashMap<i64, &Recipe> = eligible.iter().map(|r| (r.id, *r)).collect();
    for pinned_day in pinned.values() {
        for &recipe_id in &pinned_day.recipe_ids {
            if !by_id.contains_key(&recipe_id) {
                return Err(RotationError::InvalidPin(recipe_id));
            }
        }
    }

    if eligible.is_empty() || !has_fillable_pattern(&eligible) {
        return Err(RotationError::NotEnoughRecipes {
            needed: 1,
            available: eligible.len(),
        });
    }

    let treat_limit = treat_limit(days);
    let pinned_treats = pinned
        .values()
        .flat_map(|day| day.recipe_ids.iter())
        .filter(|id| by_id.get(id).is_some_and(|r| r.treat))
        .count();
    if pinned_treats > treat_limit {
        return Err(RotationError::TreatLimitExceeded {
            limit: treat_limit,
            days,
        });
    }

    let mut rng: SmallRng = match seed {
        Some(s) => SmallRng::seed_from_u64(s),
        None => SmallRng::from_entropy(),
    };

    let mut plan: Vec<PlanDay> = vec![PlanDay::new(Vec::new()); days];
    let mut used: HashSet<i64> = HashSet::new();
    let mut used_sabjis: HashSet<i64> = HashSet::new();
    let mut used_cuisines: HashSet<String> = HashSet::new();
    let mut treat_count = 0usize;

    for (&day, pinned_day) in pinned {
        for &id in &pinned_day.recipe_ids {
            if let Some(recipe) = by_id.get(&id) {
                used.insert(id);
                used_cuisines.insert(recipe.cuisine.clone());
                if recipe.role == RecipeRole::Sabji {
                    used_sabjis.insert(id);
                }
                if recipe.treat {
                    treat_count += 1;
                }
            }
        }
        plan[day] = pinned_day.clone();
    }

    #[allow(clippy::needless_range_loop)] // `day` is used as a pinned-map key.
    for day in 0..days {
        if pinned.contains_key(&day) {
            continue;
        }

        let preferred_cuisine = preferred_cuisine(&eligible, &used_cuisines);
        let mut picked = None;
        for allow_repeats in [false, true] {
            picked = fill_any_pattern(
                &eligible,
                &mut rng,
                &used,
                &used_sabjis,
                treat_limit.saturating_sub(treat_count),
                preferred_cuisine.as_deref(),
                allow_repeats,
            );
            if picked.is_some() {
                break;
            }
        }

        let Some(day_plan) = picked else {
            return Err(RotationError::NotEnoughRecipes {
                needed: 1,
                available: eligible.len(),
            });
        };

        for id in &day_plan.recipe_ids {
            if let Some(recipe) = by_id.get(id) {
                used.insert(*id);
                used_cuisines.insert(recipe.cuisine.clone());
                if recipe.role == RecipeRole::Sabji {
                    used_sabjis.insert(*id);
                }
                if recipe.treat {
                    treat_count += 1;
                }
            }
        }
        plan[day] = day_plan;
    }

    Ok(plan)
}

pub fn reroll_day(
    recipes: &[Recipe],
    plan: &[PlanDay],
    day: usize,
    tag_filters: &[String],
    cuisine_filters: &[String],
    seed: Option<u64>,
) -> Result<PlanDay, RotationError> {
    if day >= plan.len() {
        return Err(RotationError::DayOutOfRange(day, plan.len()));
    }
    let pinned: HashMap<usize, PlanDay> = plan
        .iter()
        .enumerate()
        .filter(|(i, _)| *i != day)
        .map(|(i, day)| (i, day.clone()))
        .collect();
    generate_rotation(
        recipes,
        plan.len(),
        tag_filters,
        cuisine_filters,
        &pinned,
        seed,
    )
    .map(|mut p| p.remove(day))
}

fn has_fillable_pattern(recipes: &[&Recipe]) -> bool {
    PATTERNS.iter().any(|pattern| {
        pattern
            .iter()
            .all(|role| recipes.iter().any(|r| r.role == *role))
    })
}

fn fill_any_pattern(
    recipes: &[&Recipe],
    rng: &mut SmallRng,
    used: &HashSet<i64>,
    used_sabjis: &HashSet<i64>,
    treat_remaining: usize,
    preferred_cuisine: Option<&str>,
    allow_repeats: bool,
) -> Option<PlanDay> {
    let mut patterns = PATTERNS.to_vec();
    patterns.shuffle(rng);

    for cuisine in [preferred_cuisine, None] {
        for pattern in &patterns {
            let mut temp_used = used.clone();
            let mut picked = Vec::with_capacity(pattern.len());
            let mut treats = 0usize;
            let mut ok = true;

            for &role in *pattern {
                let remaining = treat_remaining.saturating_sub(treats);
                let picker = PickContext {
                    recipes,
                    role,
                    used: &temp_used,
                    used_sabjis,
                    treat_remaining: remaining,
                    preferred_cuisine: cuisine,
                    allow_repeats,
                };
                let Some(recipe) = choose_recipe(picker, rng) else {
                    ok = false;
                    break;
                };
                temp_used.insert(recipe.id);
                treats += usize::from(recipe.treat);
                picked.push(recipe.id);
            }

            if ok {
                return Some(PlanDay::new(picked));
            }
        }
    }
    None
}

struct PickContext<'a, 'b> {
    recipes: &'b [&'a Recipe],
    role: RecipeRole,
    used: &'b HashSet<i64>,
    used_sabjis: &'b HashSet<i64>,
    treat_remaining: usize,
    preferred_cuisine: Option<&'b str>,
    allow_repeats: bool,
}

fn choose_recipe<'a>(ctx: PickContext<'a, '_>, rng: &mut SmallRng) -> Option<&'a Recipe> {
    let mut candidates: Vec<&Recipe> = ctx
        .recipes
        .iter()
        .copied()
        .filter(|r| r.role == ctx.role)
        .filter(|r| ctx.allow_repeats || !ctx.used.contains(&r.id))
        .filter(|r| !r.treat || ctx.treat_remaining > 0)
        .filter(|r| ctx.preferred_cuisine.is_none_or(|c| r.cuisine == c))
        .collect();

    if candidates.is_empty() && ctx.preferred_cuisine.is_some() {
        candidates = ctx
            .recipes
            .iter()
            .copied()
            .filter(|r| r.role == ctx.role)
            .filter(|r| ctx.allow_repeats || !ctx.used.contains(&r.id))
            .filter(|r| !r.treat || ctx.treat_remaining > 0)
            .collect();
    }

    if ctx.role == RecipeRole::Sabji && candidates.iter().any(|r| !ctx.used_sabjis.contains(&r.id))
    {
        candidates.retain(|r| !ctx.used_sabjis.contains(&r.id));
    }

    candidates.sort_by_key(|r| r.last_used.unwrap_or(i64::MIN));
    if candidates.is_empty() {
        None
    } else {
        Some(candidates[weighted_lru_index(rng, candidates.len())])
    }
}

fn preferred_cuisine(recipes: &[&Recipe], used_cuisines: &HashSet<String>) -> Option<String> {
    let mut cuisines: Vec<String> = recipes.iter().map(|r| r.cuisine.clone()).collect();
    cuisines.sort();
    cuisines.dedup();
    cuisines
        .iter()
        .find(|c| !used_cuisines.contains(*c))
        .cloned()
        .or_else(|| cuisines.first().cloned())
}

fn treat_limit(days: usize) -> usize {
    days.div_ceil(7)
}

/// Pick an index in `0..n` with a bias toward smaller indices (which callers sort
/// to be the least-recently-used candidates first).
fn weighted_lru_index(rng: &mut SmallRng, n: usize) -> usize {
    if n <= 1 {
        return 0;
    }
    // Triangular weighting: weight(i) = n - i, so index 0 is most likely.
    let total: u64 = (1..=n as u64).sum();
    let mut pick = rng.gen_range(0..total);
    for i in 0..n {
        let weight = (n - i) as u64;
        if pick < weight {
            return i;
        }
        pick -= weight;
    }
    n - 1
}
