//! Build script.
//!
//! `sqlx::migrate!("./migrations")` embeds the migration files at compile time.
//! Cargo does not track newly added files referenced only by that macro, so this
//! script makes any change inside `migrations/` (add, edit, remove) trigger a
//! rebuild. Without it a running `cargo run` would silently keep using an older
//! embedded migration set.

fn main() {
    println!("cargo:rerun-if-changed=migrations");
}
