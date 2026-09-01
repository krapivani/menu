-- Migration 3: rename "cluster" to "base"
-- "Cluster" read like infrastructure rather than cooking; a base is the
-- culinary term for a prep (paste, tadka, masala blend) that dishes are
-- built on. Tables, columns and the `recipe_items.ref_type` values all move
-- over. Applied as a separate numbered migration so databases created under
-- schema version 1 upgrade cleanly instead of tripping over `PRAGMA
-- user_version`.

ALTER TABLE ingredient_clusters RENAME TO bases;
ALTER TABLE cluster_members RENAME TO base_members;
ALTER TABLE base_members RENAME COLUMN cluster_id TO base_id;

-- `recipe_items.ref_type` carries a CHECK constraint, which SQLite cannot
-- alter in place, so the table is rebuilt with the new allowed values and
-- the existing rows are migrated across.
DROP INDEX IF EXISTS idx_recipe_items_recipe;

CREATE TABLE recipe_items_new (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    recipe_id INTEGER NOT NULL REFERENCES recipes(id) ON DELETE CASCADE,
    ref_type TEXT NOT NULL CHECK (ref_type IN ('ingredient', 'base')),
    ref_id INTEGER NOT NULL,
    quantity REAL NOT NULL,
    unit TEXT NOT NULL
);

INSERT INTO recipe_items_new (id, recipe_id, ref_type, ref_id, quantity, unit)
SELECT id,
       recipe_id,
       CASE ref_type WHEN 'cluster' THEN 'base' ELSE ref_type END,
       ref_id,
       quantity,
       unit
FROM recipe_items;

DROP TABLE recipe_items;
ALTER TABLE recipe_items_new RENAME TO recipe_items;

DROP INDEX IF EXISTS idx_cluster_members_cluster;
CREATE INDEX idx_base_members_base ON base_members(base_id);
CREATE INDEX idx_recipe_items_recipe ON recipe_items(recipe_id);

PRAGMA user_version = 3;
