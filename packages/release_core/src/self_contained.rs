//! Self-contained publishing (Phase 2 application).
//!
//! Before publishing to a private/primary registry, the internal (workspace) dependencies
//! of the crates being released are rewritten to `registry = "<name>"` so the published
//! crates resolve those deps from that registry. The edit is applied to the manifests in
//! the (ephemeral CI) checkout; a [`ManifestBackup`] restores the originals on scope exit,
//! so the working tree is always left unchanged even if publishing fails.

use std::collections::HashSet;

use anyhow::Context as _;
use cargo_metadata::camino::{Utf8Path, Utf8PathBuf};
use cargo_utils::LocalManifest;

/// Rewrite the internal dependencies of each manifest to `registry`, in place.
///
/// `internal_deps` is the set of workspace package names being published together — any
/// dependency naming one of them gets `registry = "<registry>"` added.
pub(crate) fn apply_self_containment(
  manifest_paths: &[&Utf8Path],
  internal_deps: &HashSet<&str>,
  registry: &str,
) -> anyhow::Result<()> {
  for path in manifest_paths {
    let mut manifest =
      LocalManifest::try_new(path).with_context(|| format!("failed to open manifest {path}"))?;
    manifest.set_dependencies_registry(internal_deps, registry);
    manifest
      .write()
      .with_context(|| format!("failed to write manifest {path}"))?;
  }
  Ok(())
}

/// Snapshot of manifest file contents that restores the originals when dropped.
///
/// Used to guarantee the checkout is returned to its original state after a self-contained
/// publish, regardless of success, error, or panic.
pub(crate) struct ManifestBackup {
  entries: Vec<(Utf8PathBuf, Vec<u8>)>,
}

impl ManifestBackup {
  /// Read and remember the current contents of each manifest.
  pub(crate) fn capture(manifest_paths: &[&Utf8Path]) -> anyhow::Result<Self> {
    let mut entries = Vec::with_capacity(manifest_paths.len());
    for path in manifest_paths {
      let bytes = fs_err::read(path.as_std_path())
        .with_context(|| format!("failed to read manifest {path}"))?;
      entries.push(((*path).to_owned(), bytes));
    }
    Ok(Self { entries })
  }
}

impl Drop for ManifestBackup {
  fn drop(&mut self) {
    for (path, bytes) in &self.entries {
      if let Err(e) = fs_err::write(path.as_std_path(), bytes) {
        tracing::warn!("failed to restore manifest {path} after self-contained publish: {e}");
      }
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  fn write(dir: &Utf8Path, name: &str, contents: &str) -> Utf8PathBuf {
    let path = dir.join(name);
    fs_err::write(path.as_std_path(), contents).unwrap();
    path
  }

  #[test]
  fn applies_registry_and_restores_on_drop() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = Utf8Path::from_path(tmp.path()).unwrap();
    let original = "[package]\nname = \"app\"\nversion = \"0.0.1\"\n\n[dependencies]\ncore = { path = \"../core\", version = \"0.0.1\" }\nserde = \"1\"\n";
    let manifest = write(dir, "Cargo.toml", original);
    let paths = [manifest.as_path()];
    let internal: HashSet<&str> = ["core"].into_iter().collect();

    {
      let _backup = ManifestBackup::capture(&paths).unwrap();
      apply_self_containment(&paths, &internal, "primary").unwrap();
      let rewritten = fs_err::read_to_string(manifest.as_std_path()).unwrap();
      assert!(
        rewritten.contains(r#"registry = "primary""#),
        "rewritten:\n{rewritten}"
      );
      // Non-internal dep untouched.
      assert!(rewritten.contains(r#"serde = "1""#));
    }

    // After the backup guard drops, the manifest is byte-for-byte the original.
    let restored = fs_err::read_to_string(manifest.as_std_path()).unwrap();
    assert_eq!(restored, original);
  }
}
