use crate::cmd::refs::{build_api, build_refs};
use crate::util::*;
use std::process::Command;

pub fn run() -> anyhow::Result<()> {
    let root = repo_root();
    require_tool("cargo", "https://rustup.rs")?;
    require_tool("mdbook", "cargo install mdbook --locked")?;
    require_tool("mdbook-xgettext", "cargo install mdbook-i18n-helpers --locked")?;
    require_tool("mdbook-gettext", "cargo install mdbook-i18n-helpers --locked")?;

    build_refs(&root)?;
    build_api(&root)?;
    build_locales(&root)?;
    assemble(&root)?;
    println!("==> Done. Open: {}", book_dir(&root).join("book/index.html").display());
    Ok(())
}

pub fn build_locales(root: &std::path::Path) -> anyhow::Result<()> {
    let book = book_dir(root);
    println!("==> Building mdBook for locales: {}", locales().join(" "));
    for locale in locales() {
        run_cmd(Command::new("mdbook")
            .args(["build", "-d", &format!("book/{locale}")])
            .env("MDBOOK_BOOK__LANGUAGE", locale)
            .current_dir(&book))?;
    }
    Ok(())
}

pub fn assemble(root: &std::path::Path) -> anyhow::Result<()> {
    println!("==> Assembling site (rustdoc + locale redirect)");
    let book = book_dir(root);
    let api_dest = book.join("book/api");
    let _ = std::fs::remove_dir_all(&api_dest);
    copy_dir_all(root.join("target/doc"), &api_dest)?;

    const INDEX_HTML: &str = "\
<!doctype html>
<meta charset=\"utf-8\">
<meta http-equiv=\"refresh\" content=\"0; url=./en/\">
<link rel=\"canonical\" href=\"./en/\">
<title>ZeroClaw Docs</title>
";
    std::fs::write(book.join("book/index.html"), INDEX_HTML)?;
    Ok(())
}
