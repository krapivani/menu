-- Migration 2: seed data
-- Individual ingredients, named clusters, and ~15 example recipes so the app
-- is usable on first load.

INSERT INTO ingredients (name, category, default_unit, aisle) VALUES
    ('ginger paste', 'spice', 'tsp', 'produce'),
    ('garlic paste', 'spice', 'tsp', 'produce'),
    ('green chilli', 'produce', 'pc', 'produce'),
    ('salt', 'spice', 'tsp', 'pantry'),
    ('red chilli powder', 'spice', 'tsp', 'pantry'),
    ('turmeric', 'spice', 'tsp', 'pantry'),
    ('cumin seeds', 'spice', 'tsp', 'pantry'),
    ('garam masala', 'spice', 'tsp', 'pantry'),
    ('onion', 'produce', 'pc', 'produce'),
    ('tomato', 'produce', 'pc', 'produce'),
    ('coriander leaves', 'produce', 'bunch', 'produce'),
    ('yogurt', 'dairy', 'cup', 'dairy'),
    ('basmati rice', 'pantry', 'cup', 'pantry'),
    ('chicken thighs', 'meat', 'lb', 'meat'),
    ('paneer', 'dairy', 'oz', 'dairy'),
    ('ghee', 'dairy', 'tbsp', 'pantry'),
    ('mustard seeds', 'spice', 'tsp', 'pantry'),
    ('curry leaves', 'produce', 'sprig', 'produce'),
    ('black pepper', 'spice', 'tsp', 'pantry'),
    ('carrot', 'produce', 'pc', 'produce'),
    ('celery', 'produce', 'stalk', 'produce'),
    ('ground beef', 'meat', 'lb', 'meat');

-- Named clusters of ingredients that are almost always used together.
INSERT INTO ingredient_clusters (name, description) VALUES
    ('ginger-garlic-chilli paste', 'The base aromatics for most Indian curries.'),
    ('tadka base', 'The tempering (tadka) used to finish South Indian dishes.'),
    ('salt & pepper', 'Basic seasoning pair.'),
    ('mirepoix', 'Onion, carrot, and celery aromatic base.');

INSERT INTO cluster_members (cluster_id, ingredient_id, quantity, unit) VALUES
    (1, (SELECT id FROM ingredients WHERE name = 'ginger paste'), 1, 'tbsp'),
    (1, (SELECT id FROM ingredients WHERE name = 'garlic paste'), 1, 'tbsp'),
    (1, (SELECT id FROM ingredients WHERE name = 'green chilli'), 1, 'pc'),
    (2, (SELECT id FROM ingredients WHERE name = 'mustard seeds'), 1, 'tsp'),
    (2, (SELECT id FROM ingredients WHERE name = 'cumin seeds'), 1, 'tsp'),
    (2, (SELECT id FROM ingredients WHERE name = 'curry leaves'), 1, 'sprig'),
    (3, (SELECT id FROM ingredients WHERE name = 'salt'), 1, 'tsp'),
    (3, (SELECT id FROM ingredients WHERE name = 'black pepper'), 1, 'tsp'),
    (4, (SELECT id FROM ingredients WHERE name = 'onion'), 1, 'pc'),
    (4, (SELECT id FROM ingredients WHERE name = 'carrot'), 1, 'pc'),
    (4, (SELECT id FROM ingredients WHERE name = 'celery'), 1, 'stalk');

INSERT INTO recipes (name, tags, instructions, servings) VALUES
    ('Butter Chicken', 'chicken', 'Simmer chicken thighs in a spiced tomato-yogurt gravy finished with ghee.', 4),
    ('Chana Masala', 'vegetarian,quick', 'Simmer chickpeas in a spiced tomato-onion gravy.', 4),
    ('Paneer Tikka Masala', 'vegetarian', 'Simmer seared paneer cubes in a spiced tomato-yogurt gravy.', 4),
    ('South Indian Lemon Rice', 'vegetarian,quick', 'Toss cooked rice with a mustard-seed tadka and lemon.', 4),
    ('Chicken Biryani', 'chicken', 'Layer spiced chicken with basmati rice and slow-cook.', 6),
    ('Tomato Rasam', 'vegetarian,quick', 'Simmer tomatoes with tamarind and a tadka into a tangy soup.', 4),
    ('Paneer Bhurji', 'vegetarian,quick', 'Scramble crumbled paneer with onion, tomato, and spices.', 4),
    ('Beef Keema', 'beef', 'Brown ground beef with onion and spices into a dry mince curry.', 4),
    ('Curry Leaves Tempered Yogurt Rice', 'vegetarian,quick', 'Fold a curry-leaf tadka into yogurt rice.', 4),
    ('Mirepoix Chicken Stew', 'chicken', 'Braise chicken thighs with mirepoix in a light broth.', 4),
    ('Garlic Ginger Fried Rice', 'vegetarian,quick', 'Stir-fry rice with the ginger-garlic-chilli base and vegetables.', 4),
    ('Chicken Curry', 'chicken', 'Simmer chicken thighs in a classic onion-tomato masala.', 4),
    ('Beef and Mirepoix Stew', 'beef', 'Braise ground beef with mirepoix and tomato.', 4),
    ('Vegetable Pulao', 'vegetarian', 'Cook basmati rice with mirepoix and a tadka finish.', 4),
    ('Paneer Curry Leaves Stir Fry', 'vegetarian,quick', 'Stir-fry paneer with onion and a curry-leaf tadka.', 4);

INSERT INTO recipe_items (recipe_id, ref_type, ref_id, quantity, unit) VALUES
    -- Butter Chicken
    (1, 'ingredient', (SELECT id FROM ingredients WHERE name = 'chicken thighs'), 1.5, 'lb'),
    (1, 'cluster', 1, 2, 'batch'),
    (1, 'ingredient', (SELECT id FROM ingredients WHERE name = 'tomato'), 3, 'pc'),
    (1, 'ingredient', (SELECT id FROM ingredients WHERE name = 'yogurt'), 0.5, 'cup'),
    (1, 'ingredient', (SELECT id FROM ingredients WHERE name = 'garam masala'), 1, 'tsp'),
    (1, 'ingredient', (SELECT id FROM ingredients WHERE name = 'red chilli powder'), 1, 'tsp'),
    (1, 'ingredient', (SELECT id FROM ingredients WHERE name = 'turmeric'), 0.5, 'tsp'),
    (1, 'ingredient', (SELECT id FROM ingredients WHERE name = 'salt'), 1, 'tsp'),
    (1, 'ingredient', (SELECT id FROM ingredients WHERE name = 'ghee'), 2, 'tbsp'),
    (1, 'ingredient', (SELECT id FROM ingredients WHERE name = 'onion'), 1, 'pc'),
    -- Chana Masala
    (2, 'ingredient', (SELECT id FROM ingredients WHERE name = 'onion'), 1, 'pc'),
    (2, 'ingredient', (SELECT id FROM ingredients WHERE name = 'tomato'), 2, 'pc'),
    (2, 'cluster', 1, 1, 'batch'),
    (2, 'ingredient', (SELECT id FROM ingredients WHERE name = 'turmeric'), 0.5, 'tsp'),
    (2, 'ingredient', (SELECT id FROM ingredients WHERE name = 'red chilli powder'), 1, 'tsp'),
    (2, 'ingredient', (SELECT id FROM ingredients WHERE name = 'garam masala'), 1, 'tsp'),
    (2, 'ingredient', (SELECT id FROM ingredients WHERE name = 'salt'), 1, 'tsp'),
    (2, 'ingredient', (SELECT id FROM ingredients WHERE name = 'coriander leaves'), 1, 'bunch'),
    -- Paneer Tikka Masala
    (3, 'ingredient', (SELECT id FROM ingredients WHERE name = 'paneer'), 8, 'oz'),
    (3, 'cluster', 1, 1, 'batch'),
    (3, 'ingredient', (SELECT id FROM ingredients WHERE name = 'tomato'), 2, 'pc'),
    (3, 'ingredient', (SELECT id FROM ingredients WHERE name = 'yogurt'), 0.25, 'cup'),
    (3, 'ingredient', (SELECT id FROM ingredients WHERE name = 'garam masala'), 1, 'tsp'),
    (3, 'ingredient', (SELECT id FROM ingredients WHERE name = 'red chilli powder'), 1, 'tsp'),
    (3, 'ingredient', (SELECT id FROM ingredients WHERE name = 'ghee'), 1, 'tbsp'),
    -- South Indian Lemon Rice
    (4, 'ingredient', (SELECT id FROM ingredients WHERE name = 'basmati rice'), 2, 'cup'),
    (4, 'cluster', 2, 1, 'batch'),
    (4, 'ingredient', (SELECT id FROM ingredients WHERE name = 'turmeric'), 0.5, 'tsp'),
    (4, 'ingredient', (SELECT id FROM ingredients WHERE name = 'salt'), 1, 'tsp'),
    -- Chicken Biryani
    (5, 'ingredient', (SELECT id FROM ingredients WHERE name = 'chicken thighs'), 2, 'lb'),
    (5, 'ingredient', (SELECT id FROM ingredients WHERE name = 'basmati rice'), 3, 'cup'),
    (5, 'cluster', 1, 2, 'batch'),
    (5, 'ingredient', (SELECT id FROM ingredients WHERE name = 'onion'), 2, 'pc'),
    (5, 'ingredient', (SELECT id FROM ingredients WHERE name = 'yogurt'), 0.5, 'cup'),
    (5, 'ingredient', (SELECT id FROM ingredients WHERE name = 'garam masala'), 2, 'tsp'),
    (5, 'ingredient', (SELECT id FROM ingredients WHERE name = 'salt'), 2, 'tsp'),
    -- Tomato Rasam
    (6, 'ingredient', (SELECT id FROM ingredients WHERE name = 'tomato'), 3, 'pc'),
    (6, 'cluster', 2, 1, 'batch'),
    (6, 'ingredient', (SELECT id FROM ingredients WHERE name = 'red chilli powder'), 1, 'tsp'),
    (6, 'ingredient', (SELECT id FROM ingredients WHERE name = 'salt'), 1, 'tsp'),
    (6, 'ingredient', (SELECT id FROM ingredients WHERE name = 'coriander leaves'), 1, 'bunch'),
    -- Paneer Bhurji
    (7, 'ingredient', (SELECT id FROM ingredients WHERE name = 'paneer'), 8, 'oz'),
    (7, 'ingredient', (SELECT id FROM ingredients WHERE name = 'onion'), 1, 'pc'),
    (7, 'ingredient', (SELECT id FROM ingredients WHERE name = 'tomato'), 1, 'pc'),
    (7, 'cluster', 1, 1, 'batch'),
    (7, 'ingredient', (SELECT id FROM ingredients WHERE name = 'turmeric'), 0.5, 'tsp'),
    (7, 'ingredient', (SELECT id FROM ingredients WHERE name = 'garam masala'), 1, 'tsp'),
    -- Beef Keema
    (8, 'ingredient', (SELECT id FROM ingredients WHERE name = 'ground beef'), 1, 'lb'),
    (8, 'ingredient', (SELECT id FROM ingredients WHERE name = 'onion'), 1, 'pc'),
    (8, 'cluster', 1, 1, 'batch'),
    (8, 'ingredient', (SELECT id FROM ingredients WHERE name = 'tomato'), 2, 'pc'),
    (8, 'ingredient', (SELECT id FROM ingredients WHERE name = 'garam masala'), 1, 'tsp'),
    (8, 'ingredient', (SELECT id FROM ingredients WHERE name = 'red chilli powder'), 1, 'tsp'),
    (8, 'ingredient', (SELECT id FROM ingredients WHERE name = 'salt'), 1, 'tsp'),
    -- Curry Leaves Tempered Yogurt Rice
    (9, 'ingredient', (SELECT id FROM ingredients WHERE name = 'basmati rice'), 2, 'cup'),
    (9, 'ingredient', (SELECT id FROM ingredients WHERE name = 'yogurt'), 1, 'cup'),
    (9, 'cluster', 2, 1, 'batch'),
    (9, 'ingredient', (SELECT id FROM ingredients WHERE name = 'salt'), 1, 'tsp'),
    -- Mirepoix Chicken Stew
    (10, 'ingredient', (SELECT id FROM ingredients WHERE name = 'chicken thighs'), 1.5, 'lb'),
    (10, 'cluster', 4, 1, 'batch'),
    (10, 'cluster', 3, 1, 'batch'),
    (10, 'ingredient', (SELECT id FROM ingredients WHERE name = 'tomato'), 1, 'pc'),
    -- Garlic Ginger Fried Rice
    (11, 'ingredient', (SELECT id FROM ingredients WHERE name = 'basmati rice'), 2, 'cup'),
    (11, 'cluster', 1, 1, 'batch'),
    (11, 'ingredient', (SELECT id FROM ingredients WHERE name = 'onion'), 1, 'pc'),
    (11, 'cluster', 3, 1, 'batch'),
    -- Chicken Curry
    (12, 'ingredient', (SELECT id FROM ingredients WHERE name = 'chicken thighs'), 2, 'lb'),
    (12, 'ingredient', (SELECT id FROM ingredients WHERE name = 'onion'), 2, 'pc'),
    (12, 'ingredient', (SELECT id FROM ingredients WHERE name = 'tomato'), 2, 'pc'),
    (12, 'cluster', 1, 2, 'batch'),
    (12, 'ingredient', (SELECT id FROM ingredients WHERE name = 'turmeric'), 0.5, 'tsp'),
    (12, 'ingredient', (SELECT id FROM ingredients WHERE name = 'red chilli powder'), 1, 'tsp'),
    (12, 'ingredient', (SELECT id FROM ingredients WHERE name = 'garam masala'), 1, 'tsp'),
    (12, 'ingredient', (SELECT id FROM ingredients WHERE name = 'salt'), 1, 'tsp'),
    -- Beef and Mirepoix Stew
    (13, 'ingredient', (SELECT id FROM ingredients WHERE name = 'ground beef'), 1.5, 'lb'),
    (13, 'cluster', 4, 2, 'batch'),
    (13, 'ingredient', (SELECT id FROM ingredients WHERE name = 'tomato'), 2, 'pc'),
    (13, 'cluster', 3, 1, 'batch'),
    -- Vegetable Pulao
    (14, 'ingredient', (SELECT id FROM ingredients WHERE name = 'basmati rice'), 2, 'cup'),
    (14, 'cluster', 4, 1, 'batch'),
    (14, 'cluster', 2, 1, 'batch'),
    (14, 'ingredient', (SELECT id FROM ingredients WHERE name = 'salt'), 1, 'tsp'),
    -- Paneer Curry Leaves Stir Fry
    (15, 'ingredient', (SELECT id FROM ingredients WHERE name = 'paneer'), 8, 'oz'),
    (15, 'cluster', 2, 1, 'batch'),
    (15, 'ingredient', (SELECT id FROM ingredients WHERE name = 'onion'), 1, 'pc'),
    (15, 'ingredient', (SELECT id FROM ingredients WHERE name = 'salt'), 1, 'tsp');

PRAGMA user_version = 2;
