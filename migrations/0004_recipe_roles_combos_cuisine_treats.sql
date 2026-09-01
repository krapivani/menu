-- Migration 4: recipe roles, cuisine, treat flag, and Gujarati starter data.

ALTER TABLE recipes ADD COLUMN role TEXT NOT NULL DEFAULT 'one_pot'
    CHECK (role IN ('dal', 'kadhi', 'rice', 'sabji', 'roti', 'one_pot'));
ALTER TABLE recipes ADD COLUMN cuisine TEXT NOT NULL DEFAULT 'gujarati';
ALTER TABLE recipes ADD COLUMN treat INTEGER NOT NULL DEFAULT 0
    CHECK (treat IN (0, 1));

-- Replace the placeholder starter library with the user's real Gujarati pantry.
DELETE FROM recipe_items;
DELETE FROM recipes;
DELETE FROM base_members;
DELETE FROM bases;
DELETE FROM ingredients;

INSERT INTO ingredients (name, category, default_unit, aisle) VALUES
    ('oil', 'pantry', 'tbsp', 'pantry'),
    ('jeeru', 'spice', 'tsp', 'pantry'),
    ('rai', 'spice', 'tsp', 'pantry'),
    ('hing', 'spice', 'tsp', 'pantry'),
    ('ginger', 'produce', 'tbsp', 'produce'),
    ('garlic', 'produce', 'tbsp', 'produce'),
    ('green chilli', 'produce', 'pc', 'produce'),
    ('salt', 'spice', 'tsp', 'pantry'),
    ('coriander', 'spice', 'tbsp', 'pantry'),
    ('cumin', 'spice', 'tbsp', 'pantry'),
    ('cilantro', 'produce', 'bunch', 'produce'),
    ('mint', 'produce', 'bunch', 'produce'),
    ('chat masala', 'spice', 'tsp', 'pantry'),
    ('pani puri masala', 'spice', 'tbsp', 'pantry'),
    ('dates', 'pantry', 'cup', 'pantry'),
    ('tamarind paste', 'pantry', 'tbsp', 'pantry'),
    ('jeeru powder', 'spice', 'tsp', 'pantry'),
    ('jaggery', 'pantry', 'tbsp', 'pantry'),
    ('red chilli powder', 'spice', 'tsp', 'pantry'),
    ('turmeric', 'spice', 'tsp', 'pantry'),
    ('onion', 'produce', 'pc', 'produce'),
    ('tomato', 'produce', 'pc', 'produce'),
    ('potato', 'produce', 'pc', 'produce'),
    ('tuvar dana', 'produce', 'cup', 'produce'),
    ('mixed vegetables', 'produce', 'cup', 'produce'),
    ('ghee', 'dairy', 'tbsp', 'dairy'),
    ('rice', 'pantry', 'cup', 'pantry'),
    ('tuvar dal', 'pantry', 'cup', 'pantry'),
    ('moong dal', 'pantry', 'cup', 'pantry'),
    ('water', 'pantry', 'cup', 'pantry'),
    ('yogurt', 'dairy', 'cup', 'dairy'),
    ('besan', 'pantry', 'tbsp', 'pantry'),
    ('kadhi masala', 'spice', 'tsp', 'pantry'),
    ('curry leaves', 'produce', 'sprig', 'produce'),
    ('cloves', 'spice', 'pc', 'pantry'),
    ('methi', 'spice', 'tsp', 'pantry'),
    ('peanuts', 'pantry', 'cup', 'pantry'),
    ('dal masala', 'spice', 'tsp', 'pantry'),
    ('bread', 'bakery', 'slice', 'bakery'),
    ('paneer', 'dairy', 'oz', 'dairy'),
    ('green pepper', 'produce', 'pc', 'produce'),
    ('butter', 'dairy', 'tbsp', 'dairy'),
    ('sandwich bread', 'bakery', 'slice', 'bakery'),
    ('peas', 'frozen', 'cup', 'freezer'),
    ('puris', 'pantry', 'pc', 'pantry'),
    ('black chana', 'pantry', 'cup', 'pantry'),
    ('moong', 'pantry', 'cup', 'pantry');

INSERT INTO bases (name, description) VALUES
    ('ginger base', 'Adu-marcha: ginger and green chilli.'),
    ('garlic base', 'Ginger, garlic, and green chilli paste.'),
    ('tadka base', 'Oil, rai, jeeru, and hing. Add recipe-specific extras separately.'),
    ('dhana-jeera powder', 'Coriander and cumin powder blend.'),
    ('green chutney', 'Cilantro, garlic, salt, and green chilli.'),
    ('green pani', 'Mint-cilantro pani for pani puri.'),
    ('tamarind pani', 'Sweet-sour tamarind pani for pani puri.');

INSERT INTO base_members (base_id, ingredient_id, quantity, unit) VALUES
    ((SELECT id FROM bases WHERE name = 'ginger base'), (SELECT id FROM ingredients WHERE name = 'ginger'), 1, 'tbsp'),
    ((SELECT id FROM bases WHERE name = 'ginger base'), (SELECT id FROM ingredients WHERE name = 'green chilli'), 2, 'pc'),
    ((SELECT id FROM bases WHERE name = 'garlic base'), (SELECT id FROM ingredients WHERE name = 'ginger'), 1, 'tbsp'),
    ((SELECT id FROM bases WHERE name = 'garlic base'), (SELECT id FROM ingredients WHERE name = 'garlic'), 1, 'tbsp'),
    ((SELECT id FROM bases WHERE name = 'garlic base'), (SELECT id FROM ingredients WHERE name = 'green chilli'), 2, 'pc'),
    ((SELECT id FROM bases WHERE name = 'tadka base'), (SELECT id FROM ingredients WHERE name = 'oil'), 1, 'tbsp'),
    ((SELECT id FROM bases WHERE name = 'tadka base'), (SELECT id FROM ingredients WHERE name = 'rai'), 1, 'tsp'),
    ((SELECT id FROM bases WHERE name = 'tadka base'), (SELECT id FROM ingredients WHERE name = 'jeeru'), 1, 'tsp'),
    ((SELECT id FROM bases WHERE name = 'tadka base'), (SELECT id FROM ingredients WHERE name = 'hing'), 0.25, 'tsp'),
    ((SELECT id FROM bases WHERE name = 'dhana-jeera powder'), (SELECT id FROM ingredients WHERE name = 'coriander'), 1, 'tbsp'),
    ((SELECT id FROM bases WHERE name = 'dhana-jeera powder'), (SELECT id FROM ingredients WHERE name = 'cumin'), 1, 'tbsp'),
    ((SELECT id FROM bases WHERE name = 'green chutney'), (SELECT id FROM ingredients WHERE name = 'cilantro'), 1, 'bunch'),
    ((SELECT id FROM bases WHERE name = 'green chutney'), (SELECT id FROM ingredients WHERE name = 'garlic'), 1, 'tbsp'),
    ((SELECT id FROM bases WHERE name = 'green chutney'), (SELECT id FROM ingredients WHERE name = 'salt'), 0.5, 'tsp'),
    ((SELECT id FROM bases WHERE name = 'green chutney'), (SELECT id FROM ingredients WHERE name = 'green chilli'), 2, 'pc'),
    ((SELECT id FROM bases WHERE name = 'green pani'), (SELECT id FROM ingredients WHERE name = 'mint'), 1, 'bunch'),
    ((SELECT id FROM bases WHERE name = 'green pani'), (SELECT id FROM ingredients WHERE name = 'cilantro'), 0.5, 'bunch'),
    ((SELECT id FROM bases WHERE name = 'green pani'), (SELECT id FROM ingredients WHERE name = 'ginger'), 1, 'tbsp'),
    ((SELECT id FROM bases WHERE name = 'green pani'), (SELECT id FROM ingredients WHERE name = 'salt'), 1, 'tsp'),
    ((SELECT id FROM bases WHERE name = 'green pani'), (SELECT id FROM ingredients WHERE name = 'chat masala'), 1, 'tsp'),
    ((SELECT id FROM bases WHERE name = 'green pani'), (SELECT id FROM ingredients WHERE name = 'pani puri masala'), 1, 'tbsp'),
    ((SELECT id FROM bases WHERE name = 'tamarind pani'), (SELECT id FROM ingredients WHERE name = 'dates'), 0.5, 'cup'),
    ((SELECT id FROM bases WHERE name = 'tamarind pani'), (SELECT id FROM ingredients WHERE name = 'tamarind paste'), 2, 'tbsp'),
    ((SELECT id FROM bases WHERE name = 'tamarind pani'), (SELECT id FROM ingredients WHERE name = 'jeeru powder'), 1, 'tsp'),
    ((SELECT id FROM bases WHERE name = 'tamarind pani'), (SELECT id FROM ingredients WHERE name = 'hing'), 0.25, 'tsp'),
    ((SELECT id FROM bases WHERE name = 'tamarind pani'), (SELECT id FROM ingredients WHERE name = 'jaggery'), 2, 'tbsp'),
    ((SELECT id FROM bases WHERE name = 'tamarind pani'), (SELECT id FROM ingredients WHERE name = 'red chilli powder'), 0.5, 'tsp'),
    ((SELECT id FROM bases WHERE name = 'tamarind pani'), (SELECT id FROM ingredients WHERE name = 'pani puri masala'), 1, 'tbsp');

INSERT INTO recipes (name, role, cuisine, treat, tags, instructions, servings) VALUES
    ('Masala khichdi', 'one_pot', 'gujarati', 0, 'gujarati', '', 4),
    ('Gujarati kadhi', 'kadhi', 'gujarati', 0, 'gujarati', '', 4),
    ('Gujarati dal', 'dal', 'gujarati', 0, 'gujarati', '', 4),
    ('Paneer sandwich', 'one_pot', 'gujarati', 1, 'gujarati,treat', '', 4),
    ('Aloo peas sandwich', 'one_pot', 'gujarati', 1, 'gujarati,treat', '', 4),
    ('Pani puri', 'one_pot', 'gujarati', 1, 'gujarati,treat', '', 4);

INSERT INTO recipe_items (recipe_id, ref_type, ref_id, quantity, unit) VALUES
    ((SELECT id FROM recipes WHERE name = 'Masala khichdi'), 'base', (SELECT id FROM bases WHERE name = 'tadka base'), 1, 'batch'),
    ((SELECT id FROM recipes WHERE name = 'Masala khichdi'), 'ingredient', (SELECT id FROM ingredients WHERE name = 'onion'), 1, 'pc'),
    ((SELECT id FROM recipes WHERE name = 'Masala khichdi'), 'base', (SELECT id FROM bases WHERE name = 'garlic base'), 1, 'batch'),
    ((SELECT id FROM recipes WHERE name = 'Masala khichdi'), 'ingredient', (SELECT id FROM ingredients WHERE name = 'salt'), 1.5, 'tsp'),
    ((SELECT id FROM recipes WHERE name = 'Masala khichdi'), 'ingredient', (SELECT id FROM ingredients WHERE name = 'tomato'), 2, 'pc'),
    ((SELECT id FROM recipes WHERE name = 'Masala khichdi'), 'ingredient', (SELECT id FROM ingredients WHERE name = 'potato'), 2, 'pc'),
    ((SELECT id FROM recipes WHERE name = 'Masala khichdi'), 'ingredient', (SELECT id FROM ingredients WHERE name = 'tuvar dana'), 1, 'cup'),
    ((SELECT id FROM recipes WHERE name = 'Masala khichdi'), 'ingredient', (SELECT id FROM ingredients WHERE name = 'mixed vegetables'), 1, 'cup'),
    ((SELECT id FROM recipes WHERE name = 'Masala khichdi'), 'ingredient', (SELECT id FROM ingredients WHERE name = 'turmeric'), 0.5, 'tsp'),
    ((SELECT id FROM recipes WHERE name = 'Masala khichdi'), 'ingredient', (SELECT id FROM ingredients WHERE name = 'red chilli powder'), 1, 'tsp'),
    ((SELECT id FROM recipes WHERE name = 'Masala khichdi'), 'base', (SELECT id FROM bases WHERE name = 'dhana-jeera powder'), 1, 'batch'),
    ((SELECT id FROM recipes WHERE name = 'Masala khichdi'), 'ingredient', (SELECT id FROM ingredients WHERE name = 'ghee'), 1, 'tbsp'),
    ((SELECT id FROM recipes WHERE name = 'Masala khichdi'), 'ingredient', (SELECT id FROM ingredients WHERE name = 'rice'), 1, 'cup'),
    ((SELECT id FROM recipes WHERE name = 'Masala khichdi'), 'ingredient', (SELECT id FROM ingredients WHERE name = 'moong dal'), 0.5, 'cup'),
    ((SELECT id FROM recipes WHERE name = 'Masala khichdi'), 'ingredient', (SELECT id FROM ingredients WHERE name = 'water'), 4, 'cup'),

    ((SELECT id FROM recipes WHERE name = 'Gujarati kadhi'), 'ingredient', (SELECT id FROM ingredients WHERE name = 'yogurt'), 2, 'cup'),
    ((SELECT id FROM recipes WHERE name = 'Gujarati kadhi'), 'ingredient', (SELECT id FROM ingredients WHERE name = 'water'), 4, 'cup'),
    ((SELECT id FROM recipes WHERE name = 'Gujarati kadhi'), 'ingredient', (SELECT id FROM ingredients WHERE name = 'besan'), 3, 'tbsp'),
    ((SELECT id FROM recipes WHERE name = 'Gujarati kadhi'), 'base', (SELECT id FROM bases WHERE name = 'ginger base'), 1, 'batch'),
    ((SELECT id FROM recipes WHERE name = 'Gujarati kadhi'), 'ingredient', (SELECT id FROM ingredients WHERE name = 'salt'), 1.5, 'tsp'),
    ((SELECT id FROM recipes WHERE name = 'Gujarati kadhi'), 'ingredient', (SELECT id FROM ingredients WHERE name = 'kadhi masala'), 1, 'tsp'),
    ((SELECT id FROM recipes WHERE name = 'Gujarati kadhi'), 'base', (SELECT id FROM bases WHERE name = 'tadka base'), 1, 'batch'),
    ((SELECT id FROM recipes WHERE name = 'Gujarati kadhi'), 'ingredient', (SELECT id FROM ingredients WHERE name = 'curry leaves'), 1, 'sprig'),
    ((SELECT id FROM recipes WHERE name = 'Gujarati kadhi'), 'ingredient', (SELECT id FROM ingredients WHERE name = 'cloves'), 3, 'pc'),
    ((SELECT id FROM recipes WHERE name = 'Gujarati kadhi'), 'ingredient', (SELECT id FROM ingredients WHERE name = 'methi'), 0.25, 'tsp'),

    ((SELECT id FROM recipes WHERE name = 'Gujarati dal'), 'ingredient', (SELECT id FROM ingredients WHERE name = 'tuvar dal'), 1, 'cup'),
    ((SELECT id FROM recipes WHERE name = 'Gujarati dal'), 'ingredient', (SELECT id FROM ingredients WHERE name = 'tomato'), 1, 'pc'),
    ((SELECT id FROM recipes WHERE name = 'Gujarati dal'), 'ingredient', (SELECT id FROM ingredients WHERE name = 'peanuts'), 0.25, 'cup'),
    ((SELECT id FROM recipes WHERE name = 'Gujarati dal'), 'ingredient', (SELECT id FROM ingredients WHERE name = 'turmeric'), 0.5, 'tsp'),
    ((SELECT id FROM recipes WHERE name = 'Gujarati dal'), 'ingredient', (SELECT id FROM ingredients WHERE name = 'red chilli powder'), 1, 'tsp'),
    ((SELECT id FROM recipes WHERE name = 'Gujarati dal'), 'ingredient', (SELECT id FROM ingredients WHERE name = 'jaggery'), 1, 'tbsp'),
    ((SELECT id FROM recipes WHERE name = 'Gujarati dal'), 'ingredient', (SELECT id FROM ingredients WHERE name = 'ginger'), 1, 'tbsp'),
    ((SELECT id FROM recipes WHERE name = 'Gujarati dal'), 'ingredient', (SELECT id FROM ingredients WHERE name = 'dal masala'), 1, 'tsp'),
    ((SELECT id FROM recipes WHERE name = 'Gujarati dal'), 'base', (SELECT id FROM bases WHERE name = 'tadka base'), 1, 'batch'),

    ((SELECT id FROM recipes WHERE name = 'Paneer sandwich'), 'ingredient', (SELECT id FROM ingredients WHERE name = 'bread'), 8, 'slice'),
    ((SELECT id FROM recipes WHERE name = 'Paneer sandwich'), 'ingredient', (SELECT id FROM ingredients WHERE name = 'yogurt'), 0.5, 'cup'),
    ((SELECT id FROM recipes WHERE name = 'Paneer sandwich'), 'ingredient', (SELECT id FROM ingredients WHERE name = 'paneer'), 8, 'oz'),
    ((SELECT id FROM recipes WHERE name = 'Paneer sandwich'), 'ingredient', (SELECT id FROM ingredients WHERE name = 'onion'), 0.5, 'pc'),
    ((SELECT id FROM recipes WHERE name = 'Paneer sandwich'), 'ingredient', (SELECT id FROM ingredients WHERE name = 'green pepper'), 1, 'pc'),
    ((SELECT id FROM recipes WHERE name = 'Paneer sandwich'), 'ingredient', (SELECT id FROM ingredients WHERE name = 'butter'), 2, 'tbsp'),
    ((SELECT id FROM recipes WHERE name = 'Paneer sandwich'), 'base', (SELECT id FROM bases WHERE name = 'green chutney'), 0.5, 'batch'),

    ((SELECT id FROM recipes WHERE name = 'Aloo peas sandwich'), 'ingredient', (SELECT id FROM ingredients WHERE name = 'sandwich bread'), 8, 'slice'),
    ((SELECT id FROM recipes WHERE name = 'Aloo peas sandwich'), 'ingredient', (SELECT id FROM ingredients WHERE name = 'butter'), 2, 'tbsp'),
    ((SELECT id FROM recipes WHERE name = 'Aloo peas sandwich'), 'base', (SELECT id FROM bases WHERE name = 'green chutney'), 0.5, 'batch'),
    ((SELECT id FROM recipes WHERE name = 'Aloo peas sandwich'), 'ingredient', (SELECT id FROM ingredients WHERE name = 'potato'), 3, 'pc'),
    ((SELECT id FROM recipes WHERE name = 'Aloo peas sandwich'), 'ingredient', (SELECT id FROM ingredients WHERE name = 'peas'), 1, 'cup'),
    ((SELECT id FROM recipes WHERE name = 'Aloo peas sandwich'), 'base', (SELECT id FROM bases WHERE name = 'garlic base'), 1, 'batch'),
    ((SELECT id FROM recipes WHERE name = 'Aloo peas sandwich'), 'ingredient', (SELECT id FROM ingredients WHERE name = 'turmeric'), 0.5, 'tsp'),
    ((SELECT id FROM recipes WHERE name = 'Aloo peas sandwich'), 'base', (SELECT id FROM bases WHERE name = 'tadka base'), 1, 'batch'),
    ((SELECT id FROM recipes WHERE name = 'Aloo peas sandwich'), 'ingredient', (SELECT id FROM ingredients WHERE name = 'salt'), 1, 'tsp'),
    ((SELECT id FROM recipes WHERE name = 'Aloo peas sandwich'), 'base', (SELECT id FROM bases WHERE name = 'dhana-jeera powder'), 1, 'batch'),

    ((SELECT id FROM recipes WHERE name = 'Pani puri'), 'ingredient', (SELECT id FROM ingredients WHERE name = 'puris'), 40, 'pc'),
    ((SELECT id FROM recipes WHERE name = 'Pani puri'), 'ingredient', (SELECT id FROM ingredients WHERE name = 'black chana'), 1, 'cup'),
    ((SELECT id FROM recipes WHERE name = 'Pani puri'), 'ingredient', (SELECT id FROM ingredients WHERE name = 'moong'), 1, 'cup'),
    ((SELECT id FROM recipes WHERE name = 'Pani puri'), 'ingredient', (SELECT id FROM ingredients WHERE name = 'potato'), 3, 'pc'),
    ((SELECT id FROM recipes WHERE name = 'Pani puri'), 'base', (SELECT id FROM bases WHERE name = 'green pani'), 1, 'batch'),
    ((SELECT id FROM recipes WHERE name = 'Pani puri'), 'base', (SELECT id FROM bases WHERE name = 'tamarind pani'), 1, 'batch'),
    ((SELECT id FROM recipes WHERE name = 'Pani puri'), 'ingredient', (SELECT id FROM ingredients WHERE name = 'pani puri masala'), 1, 'tbsp');

PRAGMA user_version = 4;
