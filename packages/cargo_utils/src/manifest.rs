use anyhow::Context;

use crate::DepTable;
use std::collections::HashSet;
use std::str;

/// A Cargo manifest
#[derive(Debug, Clone)]
pub struct Manifest {
  /// Manifest contents as TOML data
  pub data: toml_edit::DocumentMut,
}

impl Manifest {
  /// Get all sections in the manifest that exist and might contain dependencies.
  /// The returned items are always `Table` or `InlineTable`.
  pub(crate) fn get_sections(&self) -> Vec<(DepTable, toml_edit::Item)> {
    let mut sections = Vec::new();

    for table in DepTable::KINDS {
      let dependency_type = table.kind_table();
      // Dependencies can be in the three standard sections...
      if self
        .data
        .get(dependency_type)
        .map(|t| t.is_table_like())
        .unwrap_or(false)
      {
        sections.push((table.clone(), self.data[dependency_type].clone()));
      }

      // ... and in `target.<target>.(build-/dev-)dependencies`.
      let target_sections = self
        .data
        .as_table()
        .get("target")
        .and_then(toml_edit::Item::as_table_like)
        .into_iter()
        .flat_map(toml_edit::TableLike::iter)
        .filter_map(|(target_name, target_table)| {
          let dependency_table = target_table.get(dependency_type)?;
          dependency_table.as_table_like().map(|_| {
            (
              table.clone().set_target(target_name),
              dependency_table.clone(),
            )
          })
        });

      sections.extend(target_sections);
    }

    sections
  }

  /// Rewrite the given internal dependencies so they resolve from `registry`, by adding
  /// `registry = "<registry>"` to each matching dependency across all dependency tables
  /// (normal/dev/build, `target.<t>.*`, and `[workspace.dependencies]`). A bare version-string
  /// dependency is expanded to a table. Entries inheriting via `workspace = true` are skipped
  /// so the marker is set once, on the `[workspace.dependencies]` entry.
  ///
  /// This produces the "self-contained" manifest form used when publishing to a private registry.
  pub fn set_dependencies_registry(&mut self, internal_deps: &HashSet<&str>, registry: &str) {
    self.for_each_dependency_table_mut(|table| {
      apply_dependency_registry(table, internal_deps, Some(registry));
    });
  }

  /// Inverse of [`set_dependencies_registry`]: remove any `registry` marker from the given
  /// internal dependencies, producing the plain manifest form crates.io accepts.
  pub fn strip_dependencies_registry(&mut self, internal_deps: &HashSet<&str>) {
    self.for_each_dependency_table_mut(|table| {
      apply_dependency_registry(table, internal_deps, None);
    });
  }

  /// Invoke `f` for every dependency table in the manifest: the standard
  /// `[dependencies]`/`[dev-dependencies]`/`[build-dependencies]`, their
  /// `[target.<t>.*]` variants, and `[workspace.dependencies]`.
  fn for_each_dependency_table_mut(&mut self, mut f: impl FnMut(&mut dyn toml_edit::TableLike)) {
    let is_dep_kind = |key: &str| DepTable::KINDS.iter().any(|kind| kind.kind_table() == key);
    for (k, v) in self.data.as_table_mut().iter_mut() {
      let key = k.get();
      if is_dep_kind(key)
        && let Some(t) = v.as_table_like_mut()
      {
        f(t);
      } else if key == "workspace"
        && let Some(ws) = v.as_table_like_mut()
      {
        for (wk, wv) in ws.iter_mut() {
          if wk.get() == "dependencies"
            && let Some(t) = wv.as_table_like_mut()
          {
            f(t);
          }
        }
      } else if key == "target"
        && let Some(targets) = v.as_table_like_mut()
      {
        for (_, tv) in targets.iter_mut() {
          if let Some(target_table) = tv.as_table_like_mut() {
            for (dk, dv) in target_table.iter_mut() {
              if is_dep_kind(dk.get())
                && let Some(t) = dv.as_table_like_mut()
              {
                f(t);
              }
            }
          }
        }
      }
    }
  }
}

/// Effective package name of a dependency entry: the `package` override, else the key.
fn dependency_package_name<'a>(key: &'a str, item: &'a toml_edit::Item) -> &'a str {
  item
    .as_table_like()
    .and_then(|t| t.get("package"))
    .and_then(toml_edit::Item::as_str)
    .unwrap_or(key)
}

/// `true` if the dependency inherits from `[workspace.dependencies]` via `workspace = true`.
fn dependency_inherits_workspace(item: &toml_edit::Item) -> bool {
  item
    .as_table_like()
    .and_then(|t| t.get("workspace"))
    .and_then(toml_edit::Item::as_bool)
    .unwrap_or(false)
}

/// Add or remove the `registry` marker on internal dependencies within one table.
fn apply_dependency_registry(
  table: &mut dyn toml_edit::TableLike,
  internal_deps: &HashSet<&str>,
  registry: Option<&str>,
) {
  for (key, item) in table.iter_mut() {
    let matches = {
      let pkg = dependency_package_name(key.get(), item);
      internal_deps.contains(pkg) && !dependency_inherits_workspace(item)
    };
    if !matches {
      continue;
    }
    match registry {
      Some(reg) => set_item_registry(item, reg),
      None => {
        if let Some(t) = item.as_table_like_mut() {
          t.remove("registry");
        }
      }
    }
  }
}

/// Ensure `item` is a table and set its `registry` key, expanding a bare version string.
fn set_item_registry(item: &mut toml_edit::Item, registry: &str) {
  if let toml_edit::Item::Value(toml_edit::Value::String(s)) = item {
    let version = s.value().to_owned();
    let mut inline = toml_edit::InlineTable::new();
    inline.insert("version", toml_edit::Value::from(version));
    *item = toml_edit::Item::Value(toml_edit::Value::InlineTable(inline));
  }
  if let Some(t) = item.as_table_like_mut() {
    t.insert("registry", toml_edit::value(registry));
  }
}

impl str::FromStr for Manifest {
  type Err = anyhow::Error;

  /// Read manifest data from string
  fn from_str(input: &str) -> ::std::result::Result<Self, Self::Err> {
    let d: toml_edit::DocumentMut = input.parse().context("Manifest not valid TOML")?;

    Ok(Self { data: d })
  }
}

impl std::fmt::Display for Manifest {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let s = self.data.to_string();
    s.fmt(f)
  }
}

#[cfg(test)]
mod registry_tests {
  use super::*;

  fn internal<'a>(names: &[&'a str]) -> HashSet<&'a str> {
    names.iter().copied().collect()
  }

  fn get<'a>(item: &'a toml_edit::Item, key: &str) -> Option<&'a toml_edit::Item> {
    item.as_table_like().and_then(|t| t.get(key))
  }

  #[test]
  fn set_registry_on_table_bare_and_target_deps() {
    let input = r#"
[dependencies]
core = { path = "../core", version = "0.0.1" }
bare = "1.0"
external = "2.0"

[dev-dependencies]
core = { path = "../core" }

[target.'cfg(unix)'.dependencies]
core = { path = "../core", version = "0.0.1" }
"#;
    let mut m: Manifest = input.parse().unwrap();
    m.set_dependencies_registry(&internal(&["core", "bare"]), "primary");

    let deps = m.data.get("dependencies").unwrap();
    assert_eq!(
      get(get(deps, "core").unwrap(), "registry")
        .unwrap()
        .as_str(),
      Some("primary")
    );
    // Bare version string expanded to a table with version + registry.
    let bare = get(deps, "bare").unwrap();
    assert_eq!(get(bare, "version").unwrap().as_str(), Some("1.0"));
    assert_eq!(get(bare, "registry").unwrap().as_str(), Some("primary"));
    // Non-internal dependency left untouched.
    assert_eq!(get(deps, "external").unwrap().as_str(), Some("2.0"));
    // dev-dependency also rewritten.
    let dev = m.data.get("dev-dependencies").unwrap();
    assert_eq!(
      get(get(dev, "core").unwrap(), "registry").unwrap().as_str(),
      Some("primary")
    );
  }

  #[test]
  fn skips_workspace_inherited_sets_workspace_table() {
    let input = r#"
[dependencies]
core = { workspace = true }

[workspace.dependencies]
core = { path = "core", version = "0.0.1" }
"#;
    let mut m: Manifest = input.parse().unwrap();
    m.set_dependencies_registry(&internal(&["core"]), "primary");

    // Member entry inheriting via workspace = true is left alone.
    let member = get(m.data.get("dependencies").unwrap(), "core").unwrap();
    assert!(get(member, "registry").is_none());
    assert_eq!(get(member, "workspace").unwrap().as_bool(), Some(true));

    // The workspace.dependencies entry carries the registry.
    let ws = get(m.data.get("workspace").unwrap(), "dependencies").unwrap();
    assert_eq!(
      get(get(ws, "core").unwrap(), "registry").unwrap().as_str(),
      Some("primary")
    );
  }

  #[test]
  fn strip_is_inverse_of_set() {
    let input = "[dependencies]\ncore = { path = \"../core\", version = \"0.0.1\" }\n";
    let mut m: Manifest = input.parse().unwrap();
    let names = internal(&["core"]);
    m.set_dependencies_registry(&names, "primary");
    assert!(m.to_string().contains("registry"));
    m.strip_dependencies_registry(&names);
    let core = get(m.data.get("dependencies").unwrap(), "core").unwrap();
    assert!(get(core, "registry").is_none());
    // The rest of the dependency is preserved.
    assert_eq!(get(core, "version").unwrap().as_str(), Some("0.0.1"));
  }

  #[test]
  fn matches_renamed_dependency_by_package() {
    let input = "[dependencies]\ncore_alias = { path = \"../core\", version = \"0.0.1\", package = \"core\" }\n";
    let mut m: Manifest = input.parse().unwrap();
    m.set_dependencies_registry(&internal(&["core"]), "primary");
    let dep = get(m.data.get("dependencies").unwrap(), "core_alias").unwrap();
    assert_eq!(get(dep, "registry").unwrap().as_str(), Some("primary"));
  }
}
