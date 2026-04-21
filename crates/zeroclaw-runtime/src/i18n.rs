//! Fluent-based i18n for tool descriptions.
//!
//! English descriptions are embedded via `include_str!` at compile time.
//! Non-English locales are loaded from disk and override English per-key.

use fluent::{FluentBundle, FluentResource};
use std::collections::HashMap;
use std::sync::OnceLock;

static DESCRIPTIONS: OnceLock<HashMap<String, String>> = OnceLock::new();

/// Initialize with a specific locale. No-op after first call.
pub fn init(locale: &str) {
    DESCRIPTIONS.get_or_init(|| load_descriptions(locale));
}

/// Get a tool description by tool name (e.g. "shell", "file_read").
pub fn get_tool_description(tool_name: &str) -> Option<&'static str> {
    let map = DESCRIPTIONS.get_or_init(|| load_descriptions(&detect_locale()));
    let key = format!("tool-{}", tool_name.replace('_', "-"));
    map.get(&key).map(String::as_str)
}

fn load_descriptions(locale: &str) -> HashMap<String, String> {
    let mut map = format_ftl_messages(include_str!("../locales/en/tools.ftl"), "en");
    if locale != "en"
        && let Some(locale_ftl) = load_ftl_from_disk(locale, "tools.ftl")
    {
        map.extend(format_ftl_messages(&locale_ftl, locale));
    }
    map
}

fn format_ftl_messages(ftl_source: &str, locale: &str) -> HashMap<String, String> {
    let resource =
        FluentResource::try_new(ftl_source.to_string()).unwrap_or_else(|(resource, _)| resource);
    let language_identifier = locale.parse().unwrap_or_else(|_| "en".parse().unwrap());
    let mut bundle = FluentBundle::new(vec![language_identifier]);
    let _ = bundle.add_resource(resource);

    let mut map = HashMap::new();
    for line in ftl_source.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') || trimmed.starts_with('-') {
            continue;
        }
        if let Some(identifier) = trimmed.split(" =").next()
            && let Some(message) = bundle.get_message(identifier)
            && let Some(pattern) = message.value()
        {
            let mut errors = vec![];
            let value = bundle.format_pattern(pattern, None, &mut errors);
            map.insert(identifier.to_string(), value.into_owned());
        }
    }
    map
}

fn load_ftl_from_disk(locale: &str, filename: &str) -> Option<String> {
    let search_paths = [
        directories::BaseDirs::new().map(|base| {
            base.config_dir()
                .join("zeroclaw/locales")
                .join(locale)
                .join(filename)
        }),
        std::env::current_exe().ok().and_then(|exe| {
            exe.parent()
                .map(|p| p.join("locales").join(locale).join(filename))
        }),
    ];
    for path in search_paths.into_iter().flatten() {
        if let Ok(content) = std::fs::read_to_string(&path) {
            tracing::debug!(path = %path.display(), "loaded locale FTL from disk");
            return Some(content);
        }
    }
    None
}

/// Detect locale: ZEROCLAW_LOCALE → LANG → LC_ALL → "en".
pub fn detect_locale() -> String {
    if let Ok(val) = std::env::var("ZEROCLAW_LOCALE") {
        let trimmed = val.trim().to_string();
        if !trimmed.is_empty() {
            return normalize_locale(&trimmed);
        }
    }
    for var in &["LANG", "LC_ALL"] {
        if let Ok(val) = std::env::var(var) {
            let locale = normalize_locale(&val);
            if locale != "C" && locale != "POSIX" && !locale.is_empty() {
                return locale;
            }
        }
    }
    "en".to_string()
}

/// Normalize "zh_CN.UTF-8" → "zh-CN".
pub fn normalize_locale(raw: &str) -> String {
    raw.split('.').next().unwrap_or(raw).replace('_', "-")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn english_descriptions_are_embedded() {
        let map = format_ftl_messages(include_str!("../locales/en/tools.ftl"), "en");
        assert!(map.contains_key("tool-shell"));
        assert!(map.contains_key("tool-file-read"));
        assert!(!map.contains_key("tool-nonexistent"));
    }

    #[test]
    fn unknown_locale_falls_back_to_english() {
        let map = load_descriptions("xx-FAKE");
        assert!(map.contains_key("tool-shell"));
    }

    #[test]
    fn normalize_locale_strips_encoding() {
        assert_eq!(normalize_locale("en_US.UTF-8"), "en-US");
        assert_eq!(normalize_locale("zh_CN.utf8"), "zh-CN");
        assert_eq!(normalize_locale("fr"), "fr");
    }

    #[test]
    fn detect_locale_from_env() {
        let saved = std::env::var("ZEROCLAW_LOCALE").ok();
        // SAFETY: test-only, single-threaded test runner.
        unsafe { std::env::set_var("ZEROCLAW_LOCALE", "ja-JP") };
        assert_eq!(detect_locale(), "ja-JP");
        match saved {
            // SAFETY: test-only, single-threaded test runner.
            Some(v) => unsafe { std::env::set_var("ZEROCLAW_LOCALE", v) },
            // SAFETY: test-only, single-threaded test runner.
            None => unsafe { std::env::remove_var("ZEROCLAW_LOCALE") },
        }
    }
}
