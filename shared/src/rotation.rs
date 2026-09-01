use crate::models::Recipe;
use rand::rngs::SmallRng;
use rand::seq::SliceRandom;
use rand::{Rng, SeedableRng};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum RotationError {
    #[error("not enough recipes match the given filters: need {needed}, have {available}")]
    NotEnoughRecipes { needed: usize, available: usize },
    #[error("pinned recipe id {0} does not exist or does not match the tag filters")]
    InvalidPin(i64),
    #[error("day index {0} is out of range for a plan of length {1}")]
    DayOutOfRange(usize, usize),
}

/// Filter recipes so that every requested tag is present on the recipe (AND semantics).
pub fn filter_by_tags<'a>(recipes: &'a [Recipe], tags: &[String]) -> Vec<&'a Recipe> {
    recipes
        .iter()
        .filter(|r| tags.iter().all(|t| r.tags.iter().any(|rt| rt == t)))
        .collect()
}

/// Generate a random weekly (or any-length) menu rotation.
///
/// * `recipes` - the full recipe library.
/// * `days` - number of days/slots to fill.
/// * `tag_filters` - only recipes containing all of these tags are eligible.
/// * `pinned` - day index -> recipe id that must be used on that day (locked).
/// * `seed` - optional seed for deterministic output (used in tests).
///
/// Returns one recipe id per day. No recipe repeats within a plan. Unpinned
/// days are filled preferring recipes with an older (or absent) `last_used`,
/// while still being randomized so the rotation doesn't feel mechanical.
pub fn generate_rotation(
    recipes: &[Recipe],
    days: usize,
    tag_filters: &[String],
    pinned: &HashMap<usize, i64>,
    seed: Option<u64>,
) -> Result<Vec<i64>, RotationError> {
    for &day in pinned.keys() {
        if day >= days {
            return Err(RotationError::DayOutOfRange(day, days));
        }
    }

    let eligible = filter_by_tags(recipes, tag_filters);
    let eligible_ids: HashSet<i64> = eligible.iter().map(|r| r.id).collect();

    for &recipe_id in pinned.values() {
        if !eligible_ids.contains(&recipe_id) {
            return Err(RotationError::InvalidPin(recipe_id));
        }
    }

    if eligible.len() < days {
        return Err(RotationError::NotEnoughRecipes {
            needed: days,
            available: eligible.len(),
        });
    }

    let mut rng: SmallRng = match seed {
        Some(s) => SmallRng::seed_from_u64(s),
        None => SmallRng::from_entropy(),
    };

    let mut used: HashSet<i64> = pinned.values().copied().collect();
    let mut result: Vec<i64> = vec![0; days];
    for (&day, &recipe_id) in pinned {
        result[day] = recipe_id;
    }

    // Candidates not pinned and not already used, ranked least-recently-used first.
    let mut candidates: Vec<&Recipe> = eligible
        .into_iter()
        .filter(|r| !used.contains(&r.id))
        .collect();
    candidates.sort_by_key(|r| r.last_used.unwrap_or(i64::MIN));

    #[allow(clippy::needless_range_loop)] // `day` is used as a pinned-map key, not just an index
    for day in 0..days {
        if pinned.contains_key(&day) {
            continue;
        }
        // Weighted pick preferring the front (least-recently-used) of `candidates`
        // via a triangular distribution over the rank, then remove it.
        let n = candidates.len();
        debug_assert!(n > 0);
        let idx = weighted_lru_index(&mut rng, n);
        let chosen = candidates.remove(idx);
        used.insert(chosen.id);
        result[day] = chosen.id;
    }

    Ok(result)
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

/// Re-roll a single day of an existing plan, keeping every other day fixed and
/// avoiding repeats against the rest of the plan.
pub fn reroll_day(
    recipes: &[Recipe],
    plan: &[i64],
    day: usize,
    tag_filters: &[String],
    seed: Option<u64>,
) -> Result<i64, RotationError> {
    if day >= plan.len() {
        return Err(RotationError::DayOutOfRange(day, plan.len()));
    }
    let eligible = filter_by_tags(recipes, tag_filters);
    let used: HashSet<i64> = plan
        .iter()
        .enumerate()
        .filter(|(i, _)| *i != day)
        .map(|(_, id)| *id)
        .collect();

    let mut candidates: Vec<&Recipe> = eligible
        .into_iter()
        .filter(|r| !used.contains(&r.id))
        .collect();
    if candidates.is_empty() {
        return Err(RotationError::NotEnoughRecipes {
            needed: 1,
            available: 0,
        });
    }
    candidates.sort_by_key(|r| r.last_used.unwrap_or(i64::MIN));

    let mut rng: SmallRng = match seed {
        Some(s) => SmallRng::seed_from_u64(s),
        None => SmallRng::from_entropy(),
    };
    let idx = weighted_lru_index(&mut rng, candidates.len());
    Ok(candidates[idx].id)
}

/// Shuffle helper kept separate so it can be unit tested independently of the
/// LRU weighting above.
#[allow(dead_code)]
fn shuffle<T>(rng: &mut SmallRng, items: &mut [T]) {
    items.shuffle(rng);
}
