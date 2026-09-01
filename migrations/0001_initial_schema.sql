-- Migration 1: initial schema
-- Normalized data model: individual ingredients and named clusters of
-- ingredients that are almost always used together are both first-class,
-- so recipe lines can reference either one.

CREATE TABLE ingredients (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE,
    category TEXT NOT NULL,
    default_unit TEXT NOT NULL,
    aisle TEXT NOT NULL
);

CREATE TABLE ingredient_clusters (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE,
    description TEXT NOT NULL DEFAULT ''
);

CREATE TABLE cluster_members (
    cluster_id INTEGER NOT NULL REFERENCES ingredient_clusters(id) ON DELETE CASCADE,
    ingredient_id INTEGER NOT NULL REFERENCES ingredients(id) ON DELETE CASCADE,
    quantity REAL NOT NULL,
    unit TEXT NOT NULL,
    PRIMARY KEY (cluster_id, ingredient_id)
);

CREATE TABLE recipes (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    tags TEXT NOT NULL DEFAULT '',
    instructions TEXT NOT NULL DEFAULT '',
    servings INTEGER NOT NULL DEFAULT 4,
    last_used INTEGER
);

CREATE TABLE recipe_items (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    recipe_id INTEGER NOT NULL REFERENCES recipes(id) ON DELETE CASCADE,
    ref_type TEXT NOT NULL CHECK (ref_type IN ('ingredient', 'cluster')),
    ref_id INTEGER NOT NULL,
    quantity REAL NOT NULL,
    unit TEXT NOT NULL
);

CREATE INDEX idx_cluster_members_cluster ON cluster_members(cluster_id);
CREATE INDEX idx_recipe_items_recipe ON recipe_items(recipe_id);

PRAGMA user_version = 1;
