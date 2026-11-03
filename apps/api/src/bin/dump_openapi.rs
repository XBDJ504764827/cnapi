//! Writes the generated OpenAPI document to disk.
//!
//! Used by `pnpm gen:openapi` so the TypeScript client can be regenerated
//! without running the server:
//!
//! ```text
//! cargo run -p cngokz-api --bin dump-openapi -- docs/openapi.json
//! ```

use std::path::PathBuf;

use utoipa::OpenApi;

use cngokz_api::routes::openapi::ApiDoc;

fn main() -> anyhow::Result<()> {
    let path = std::env::args()
        .nth(1)
        .map_or_else(|| PathBuf::from("docs/openapi.json"), PathBuf::from);

    if let Some(parent) = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        std::fs::create_dir_all(parent)?;
    }

    let document = ApiDoc::openapi().to_pretty_json()?;
    std::fs::write(&path, format!("{document}\n"))?;
    println!("OpenAPI document written to {}", path.display());

    Ok(())
}
