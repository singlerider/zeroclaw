use crate::cmd::refs::{build_api, build_refs};
use crate::util::*;
use std::process::{Command, Stdio};
use std::sync::{Arc, atomic::{AtomicBool, Ordering}};

const PORT: u16 = 3000;

pub fn run(locale: &str) -> anyhow::Result<()> {
    let root = repo_root();
    require_tool("cargo", "https://rustup.rs")?;
    require_tool("mdbook", "cargo install mdbook --locked")?;
    require_tool("mdbook-xgettext", "cargo install mdbook-i18n-helpers --locked")?;
    require_tool("mdbook-gettext", "cargo install mdbook-i18n-helpers --locked")?;
    require_tool("python3", "https://python.org")?;

    let ref_dir = ref_dir(&root);
    if !ref_dir.join("cli.md").exists() || !ref_dir.join("config.md").exists() {
        build_refs(&root)?;
    }
    if !root.join("target/doc").exists() {
        build_api(&root)?;
    }

    let book = book_dir(&root);
    let out_dir = book.join("book");

    // Initial build
    run_cmd(Command::new("mdbook")
        .args(["build", "-d", "book"])
        .env("MDBOOK_BOOK__LANGUAGE", locale)
        .current_dir(&book))?;
    let api_dest = out_dir.join("api");
    let _ = std::fs::remove_dir_all(&api_dest);
    copy_dir_all(root.join("target/doc"), &api_dest)?;

    // Watch for source changes in background (no built-in HTTP server)
    let mut watch = Command::new("mdbook")
        .args(["watch", "-d", "book"])
        .env("MDBOOK_BOOK__LANGUAGE", locale)
        .current_dir(&book)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()?;

    // Re-copy api/ whenever mdbook's clean-on-rebuild removes it
    let running = Arc::new(AtomicBool::new(true));
    let running_clone = running.clone();
    let root_clone = root.clone();
    let out_dir_clone = out_dir.clone();
    std::thread::spawn(move || {
        while running_clone.load(Ordering::Relaxed) {
            std::thread::sleep(std::time::Duration::from_secs(1));
            if out_dir_clone.exists()
                && !out_dir_clone.join("api").exists()
                && root_clone.join("target/doc").exists()
            {
                let _ = copy_dir_all(root_clone.join("target/doc"), out_dir_clone.join("api"));
            }
        }
    });

    let url = format!("http://localhost:{PORT}");
    println!("==> Serving locale '{locale}' at {url}");
    println!("    API reference: {url}/api/index.html");
    println!("    (live-reload active — edit docs/book/src/ to trigger rebuild)");

    let _ = Command::new("xdg-open").arg(&url).spawn()
        .or_else(|_| Command::new("open").arg(&url).spawn());

    // python3 owns the terminal — blocks until Ctrl-C
    let _ = Command::new("python3")
        .args(["-m", "http.server", &PORT.to_string(), "--directory"])
        .arg(&out_dir)
        .status();

    running.store(false, Ordering::Relaxed);
    let _ = watch.kill();
    let _ = watch.wait();
    Ok(())
}
