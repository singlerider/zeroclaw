use std::path::{Path, PathBuf};
use std::process::Command;

pub fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("xtask lives one level below repo root")
        .to_owned()
}

pub fn book_dir(root: &Path) -> PathBuf {
    root.join("docs/book")
}

pub fn ref_dir(root: &Path) -> PathBuf {
    root.join("docs/book/src/reference")
}

pub fn po_dir(root: &Path) -> PathBuf {
    root.join("docs/book/po")
}

pub fn pot_file(root: &Path) -> PathBuf {
    root.join("docs/book/po/messages.pot")
}

pub fn locales() -> &'static [&'static str] {
    &["en", "ja"]
}

pub fn require_tool(cmd: &str, install_hint: &str) -> anyhow::Result<()> {
    let found = std::env::var_os("PATH")
        .map(|paths| {
            std::env::split_paths(&paths).any(|dir| {
                let candidate = dir.join(cmd);
                candidate.is_file()
                    || dir.join(format!("{cmd}.exe")).is_file()
            })
        })
        .unwrap_or(false);
    if !found {
        anyhow::bail!("'{}' not found on PATH\n  install: {}", cmd, install_hint);
    }
    Ok(())
}

pub fn run_cmd(cmd: &mut Command) -> anyhow::Result<()> {
    let status = cmd.status()?;
    if !status.success() {
        anyhow::bail!("command failed: {:?}", cmd.get_program());
    }
    Ok(())
}

pub fn copy_dir_all(src: impl AsRef<Path>, dst: impl AsRef<Path>) -> anyhow::Result<()> {
    std::fs::create_dir_all(&dst)?;
    for entry in std::fs::read_dir(&src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        if ty.is_dir() {
            copy_dir_all(entry.path(), dst.as_ref().join(entry.file_name()))?;
        } else {
            std::fs::copy(entry.path(), dst.as_ref().join(entry.file_name()))?;
        }
    }
    Ok(())
}
