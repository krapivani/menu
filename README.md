# menu

Help organize dinner menu and grocery: pick a random weekly dinner rotation
from a recipe library and turn it into a consolidated grocery list.

The whole app is a static site — a [Leptos](https://leptos.dev) single-page
app compiled to WebAssembly, with its data stored in a SQLite database that
runs *inside the browser*. There is no backend server and no cloud database,
so it deploys for free to GitHub Pages.

## Architecture

This is a Cargo workspace with two crates:

- **`shared`** — plain Rust domain types and all business logic: menu
  rotation, base expansion, and grocery-list aggregation. No I/O, so it
  builds and tests natively with `cargo test`.
- **`frontend`** — the Leptos CSR (client-side-rendered) app, built with
  [`trunk`](https://trunkrs.dev). It embeds SQLite compiled to WebAssembly
  (via the `sqlite-wasm-rs` crate) to run real SQL queries against the data
  model described below, entirely on-device.

Because `sqlite-wasm-rs`'s build script only compiles for
`wasm32-unknown-unknown`, `frontend` can only be built/checked/tested with
that target (or via `trunk`, which sets it automatically). The workspace's
root `Cargo.toml` sets `default-members = ["shared"]` so a plain
`cargo build` / `cargo test` / `cargo clippy` at the repo root only touches
`shared`.

## Navigation

The app is organised around the workflow — generate a menu, review it, shop
from it — rather than a flat row of tabs:

- **Home** is the *Generate Menu* screen: tag filters, number of days, a
  **Generate** button, and the resulting plan listed per day with per-day
  re-roll and pin/lock.
- **View grocery list** appears on Home only *after* a menu has been
  generated — the grocery list is not a destination of its own, because it
  has nothing to show without a plan.
- **Recipes** is reached from the link in the top bar. Each recipe expands to
  show its ingredient lines *as authored*: plain ingredients as themselves,
  and a base as a single named line that expands in place to reveal its
  members and their scaled quantities.
- **+ Add** offers *Add ingredient* and *Add recipe*.
- The **gear** menu holds *Manage ingredients* and *Backup & restore (JSON)*.
  Keep taking those JSON backups: the database lives only in this browser.
- **Bases** have no destination of their own. They're created and edited
  inline from the recipe editor ("Manage bases"), and browsed via the
  drill-down from any base line in a recipe.

The grocery list itself supports checking items off, the expand-bases toggle,
ad-hoc items typed by hand (milk, bin bags — persisted, and visually distinct
from recipe-derived lines), **Copy to clipboard** and a `.txt` download for
pasting into a notes app, and **Save as PDF / print**, which uses the
browser's native print dialog together with a `@media print` stylesheet
rather than bundling a PDF library into the WASM binary.

## Data model

The data model is normalized so that both single ingredients (`ginger
paste`, `salt`, `red chilli powder`, ...) and named **bases** — building
blocks of ingredients that are almost always used together
(`ginger-garlic-chilli paste`, `tadka base`, ...) — are first-class:

- `ingredients (id, name, category, default_unit, aisle)`
- `bases (id, name, description)`
- `base_members (base_id, ingredient_id, quantity, unit)` — the
  proportions of each ingredient within a base
- `recipes (id, name, tags, instructions, servings, last_used)`
- `recipe_items (recipe_id, ref_type CHECK IN ('ingredient','base'),
  ref_id, quantity, unit)` — each recipe line references *either* a single
  ingredient or a whole base

SQL migrations live in [`/migrations`](./migrations) and are applied on
first load, tracked via `PRAGMA user_version` (schema in
`0001_initial_schema.sql`, seed data — 22 ingredients, 4 bases, 15 example
recipes — in `0002_seed_data.sql`, and the `cluster` → `base` rename in
`0003_rename_clusters_to_bases.sql`). They're plain, portable SQL (no
SQLite-only syntax where a standard alternative exists) so they can be
replayed unmodified against a hosted database later (see below).

JSON exported before the rename still imports: the `clusters` key and the
`"cluster"` ref type are accepted as aliases for `bases` and `"base"`.

### Grocery list logic

Building a grocery list expands each recipe's base references into their
member ingredients, scaling each member's quantity by the amount of base
called for, then merges lines with the same ingredient name and a
compatible unit (case-insensitive). Ingredients with incompatible units stay
on separate lines rather than being silently combined. For example, a recipe
calling for `ginger-garlic-chilli paste` and another calling for plain
`ginger paste` produce a single combined `ginger paste` line. The grocery
list has a toggle to show expanded individual ingredients (the default)
or show bases as single line items instead, for people who buy the paste
pre-made. Recipe *display* is the other way round — base-first, never
flattened — so a recipe reads the way it was written.

### Rotation logic

Generating a menu picks recipes at random with no repeats within the plan,
using a least-recently-used weighting (via each recipe's `last_used`
timestamp) so the same handful of recipes don't dominate every week. It
supports tag filters (e.g. `vegetarian`, `quick`, `beef` — a recipe must
have *all* requested tags), pinned/locked days that are left untouched, and
a per-day re-roll. A deterministic seed can be passed in for tests.

## Local development

Install the Rust toolchain, the WASM target, and `trunk`:

```sh
rustup target add wasm32-unknown-unknown
cargo install trunk
```

Then, from `frontend/`:

```sh
trunk serve
```

This serves the app locally with live-reload. To build the production
bundle (as CI does), run `trunk build --release --public-url /menu/` — the
`public-url` matters because the site is served from
`https://<user>.github.io/menu/`, not the domain root.

Native-only crates:

```sh
cargo test -p shared          # unit tests for rotation + grocery logic
cargo clippy -p shared -- -D warnings
cargo fmt --all -- --check
```

## Browser-local storage and its caveats

The app keeps its SQLite database entirely client-side. On every mutation
(adding a recipe, checking off a grocery item, etc.) the whole database is
serialized to JSON and persisted to the browser's `localStorage`, and
restored from there on load. Persistence is behind a `Store` trait
(`frontend/src/store.rs`), so it's swappable without touching any UI code.

**This means your data lives only in the current browser, on the current
device.** Clearing site data (or switching browsers/devices) loses it. Use
the gear menu's **Backup & restore (JSON)** screen to back up or move your
library — do this periodically, or before clearing browser storage.

The current implementation uses `localStorage` as its storage tier. A more
durable [Origin Private File System](https://developer.mozilla.org/en-US/docs/Web/API/File_System_API/Origin_private_file_system)
(OPFS)-backed SQLite VFS (e.g. via `sqlite-wasm-vfs`'s `sahpool` VFS) is a
natural follow-up improvement, since it survives larger datasets and gives
real file-level durability, while keeping the same `Store` trait seam.

### Future: syncing with Turso

Because the schema and migrations are plain, portable SQL, they can be
replayed as-is against a hosted [Turso](https://turso.tech) (libSQL)
database, which speaks the same SQL dialect as SQLite. Multi-device sync
would mean adding a `TursoStore` that implements the same `Store` trait used
by `SqliteWasmStore` today — the UI and business logic (`shared`) wouldn't
need to change at all.

## Enabling GitHub Pages

1. Make sure the repository is public (Pages on private repos requires
   GitHub Pro/Team).
2. Go to **Settings → Pages** and set **Source** to **GitHub Actions**.
3. Push to `main` — `.github/workflows/deploy-pages.yml` builds the app with
   `trunk` and deploys it via `actions/deploy-pages`.

The site will be available at `https://<owner>.github.io/menu/`.
