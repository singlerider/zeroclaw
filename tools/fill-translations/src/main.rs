use clap::Parser;
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Parser)]
#[command(about = "Fill empty/fuzzy .po entries via Anthropic API")]
struct Args {
    #[arg(long)]
    po: PathBuf,
    #[arg(long)]
    locale: String,
    /// Re-translate all entries, not just empty/fuzzy ones
    #[arg(long)]
    force: bool,
    /// Entries per API call
    #[arg(long, default_value = "50")]
    batch: usize,
    /// Model override (env FILL_MODEL also works)
    #[arg(long)]
    model: Option<String>,
}

/// A parsed .po entry, carrying line positions so we can rewrite in place.
struct Entry {
    /// 0-based line index of the `msgstr` keyword line
    msgstr_line: usize,
    /// 0-based line index of the `#, fuzzy` flag line, if present
    fuzzy_line: Option<usize>,
    /// Decoded msgid text (po string escapes resolved, concatenated)
    msgid: String,
    /// Decoded msgstr text
    msgstr: String,
}

/// Decode a run of po quoted-string lines into a plain Rust String.
/// Each line looks like `"some text\n"` — strip outer quotes, unescape.
fn decode_po_string(lines: &[String]) -> String {
    let mut out = String::new();
    for line in lines {
        let inner = line.trim();
        if inner.starts_with('"') && inner.ends_with('"') && inner.len() >= 2 {
            let s = &inner[1..inner.len() - 1];
            let mut chars = s.chars().peekable();
            while let Some(c) = chars.next() {
                if c == '\\' {
                    match chars.next() {
                        Some('n') => out.push('\n'),
                        Some('t') => out.push('\t'),
                        Some('\\') => out.push('\\'),
                        Some('"') => out.push('"'),
                        Some(other) => { out.push('\\'); out.push(other); }
                        None => out.push('\\'),
                    }
                } else {
                    out.push(c);
                }
            }
        }
    }
    out
}

/// Replace invalid JSON escape sequences (e.g. `\[`, `\(`) with their literal characters.
fn sanitize_json_escapes(s: &str) -> String {
    let valid = ['\"', '\\', '/', 'b', 'f', 'n', 'r', 't', 'u'];
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\\' {
            match chars.peek() {
                Some(&next) if valid.contains(&next) => {
                    out.push(c);
                }
                Some(_) => {
                    // Invalid escape — drop the backslash, keep the character
                }
                None => { out.push(c); }
            }
        } else {
            out.push(c);
        }
    }
    out
}

/// Encode a plain string into a single-line po `msgstr "..."` value.
fn encode_po_string(s: &str) -> String {
    let mut out = String::new();
    for c in s.chars() {
        match c {
            '"'  => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\t' => out.push_str("\\t"),
            other => out.push(other),
        }
    }
    out
}

fn commit_entry(
    entries: &mut Vec<Entry>,
    fuzzy_line: Option<usize>,
    msgstr_line_idx: Option<usize>,
    msgid_lines: &[String],
    msgstr_lines: &[String],
) {
    let Some(ms_line) = msgstr_line_idx else { return };
    let msgid = decode_po_string(msgid_lines);
    let msgstr = decode_po_string(msgstr_lines);
    if msgid.is_empty() {
        return; // header entry
    }
    entries.push(Entry { msgstr_line: ms_line, fuzzy_line, msgid, msgstr });
}

fn parse_po(lines: &[String]) -> Vec<Entry> {
    let mut entries = Vec::new();
    let mut fuzzy_line: Option<usize> = None;
    let mut in_msgid = false;
    let mut in_msgstr = false;
    let mut msgid_lines: Vec<String> = Vec::new();
    let mut msgstr_lines: Vec<String> = Vec::new();
    let mut msgstr_line_idx: Option<usize> = None;

    for (idx, line) in lines.iter().enumerate() {
        let trimmed = line.trim_end();

        if trimmed.starts_with("#,") && trimmed.contains("fuzzy") {
            commit_entry(&mut entries, fuzzy_line, msgstr_line_idx, &msgid_lines, &msgstr_lines);
            fuzzy_line = Some(idx);
            in_msgid = false;
            in_msgstr = false;
            msgid_lines.clear();
            msgstr_lines.clear();
            msgstr_line_idx = None;
            continue;
        }

        if trimmed.starts_with("msgid ") {
            if msgstr_line_idx.is_some() {
                commit_entry(&mut entries, fuzzy_line, msgstr_line_idx, &msgid_lines, &msgstr_lines);
                fuzzy_line = None;
                msgid_lines.clear();
                msgstr_lines.clear();
                msgstr_line_idx = None;
            }
            in_msgid = true;
            in_msgstr = false;
            msgid_lines.clear();
            msgid_lines.push(trimmed[6..].to_string());
            continue;
        }

        if trimmed.starts_with("msgstr ") {
            in_msgid = false;
            in_msgstr = true;
            msgstr_lines.clear();
            msgstr_line_idx = Some(idx);
            msgstr_lines.push(trimmed[7..].to_string());
            continue;
        }

        if trimmed.starts_with('"') {
            if in_msgid  { msgid_lines.push(trimmed.to_string()); }
            if in_msgstr { msgstr_lines.push(trimmed.to_string()); }
            continue;
        }

        if trimmed.is_empty() || trimmed.starts_with('#') {
            in_msgid = false;
            in_msgstr = false;
        }
    }
    commit_entry(&mut entries, fuzzy_line, msgstr_line_idx, &msgid_lines, &msgstr_lines);
    entries
}

fn write_po(
    lines: &[String],
    raw: &str,
    translations: &HashMap<usize, String>,
    translated_entries: &[&Entry],
    to_accept: &[&Entry],
    path: &std::path::Path,
) -> anyhow::Result<()> {
    // Remove fuzzy flags for entries we translated + entries accepted as-is
    let fuzzy_lines_to_remove: std::collections::HashSet<usize> = translated_entries
        .iter()
        .filter(|e| e.fuzzy_line.is_some() && translations.contains_key(&e.msgstr_line))
        .chain(to_accept.iter())
        .filter_map(|e| e.fuzzy_line)
        .collect();

    let mut output_lines: Vec<String> = Vec::with_capacity(lines.len());
    let mut i = 0;
    while i < lines.len() {
        if fuzzy_lines_to_remove.contains(&i) {
            i += 1;
            continue;
        }
        if let Some(translated) = translations.get(&i) {
            output_lines.push(format!("msgstr \"{}\"", encode_po_string(translated)));
            i += 1;
            while i < lines.len() && lines[i].trim_start().starts_with('"') {
                i += 1;
            }
            continue;
        }
        output_lines.push(lines[i].clone());
        i += 1;
    }

    let mut out = output_lines.join("\n");
    if raw.ends_with('\n') {
        out.push('\n');
    }
    std::fs::write(path, out)?;
    Ok(())
}

async fn translate_batch(
    client: &reqwest::Client,
    api_key: &str,
    model: &str,
    locale: &str,
    batch: &[&str],
) -> anyhow::Result<Vec<String>> {
    let numbered: String = batch
        .iter()
        .enumerate()
        .map(|(i, s)| format!("{}. {}", i + 1, s))
        .collect::<Vec<_>>()
        .join("\n");

    let prompt = format!(
        "Translate these English documentation strings to locale '{locale}'.\n\
         Return ONLY a JSON array of translated strings in the same order.\n\
         No explanation. Preserve backticks, bold (**), and code spans exactly.\n\
         If a string is already in the target language or is a code literal, return it unchanged.\n\n\
         {numbered}"
    );

    let body = serde_json::json!({
        "model": model,
        "max_tokens": 8192,
        "messages": [{"role": "user", "content": prompt}]
    });

    let (auth_name, auth_value) = if api_key.starts_with("sk-ant-oat") {
        ("Authorization", format!("Bearer {api_key}"))
    } else {
        ("x-api-key", api_key.to_string())
    };

    let resp = client
        .post("https://api.anthropic.com/v1/messages")
        .header(auth_name, auth_value)
        .header("anthropic-version", "2023-06-01")
        .json(&body)
        .send()
        .await?
        .error_for_status()?
        .json::<serde_json::Value>()
        .await?;

    let text = resp["content"][0]["text"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("no text in response"))?;

    // Extract the JSON array — find first '[' and last ']'
    let start = text.find('[').ok_or_else(|| anyhow::anyhow!("no JSON array in response"))?;
    let end   = text.rfind(']').ok_or_else(|| anyhow::anyhow!("no closing ] in response"))?;
    let raw_json = &text[start..=end];
    let arr: Vec<String> = serde_json::from_str(raw_json)
        .or_else(|_| serde_json::from_str(&sanitize_json_escapes(raw_json)))?;
    Ok(arr)
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    let api_key = std::env::var("ANTHROPIC_API_KEY")
        .map_err(|_| anyhow::anyhow!("ANTHROPIC_API_KEY not set"))?;

    let model = args
        .model
        .or_else(|| std::env::var("FILL_MODEL").ok())
        .unwrap_or_else(|| "claude-haiku-4-5-20251001".to_string());

    let raw = std::fs::read_to_string(&args.po)?;
    let lines: Vec<String> = raw.lines().map(str::to_owned).collect();

    let entries = parse_po(&lines);

    let mut translations: HashMap<usize, String> = HashMap::new();

    // Repair entries where msgid ends with \n but msgstr doesn't — corrupted by
    // interrupted runs. Pre-populate into translations so write_po fixes them inline.
    let mut repaired = 0;
    for entry in &entries {
        if !entry.msgstr.is_empty()
            && entry.msgid.ends_with('\n')
            && !entry.msgstr.ends_with('\n')
        {
            translations.insert(entry.msgstr_line, format!("{}\n", entry.msgstr));
            repaired += 1;
        }
    }
    if repaired > 0 {
        println!("==> Repairing {repaired} entries missing trailing \\n");
    }

    // Entries with empty msgstr need AI translation.
    // Fuzzy entries already have a translation — accept it as-is, just drop the flag.
    // --force retranslates everything regardless.
    let to_translate: Vec<&Entry> = entries
        .iter()
        .filter(|e| args.force || e.msgstr.is_empty())
        .collect();

    let to_accept: Vec<&Entry> = entries
        .iter()
        .filter(|e| !args.force && e.fuzzy_line.is_some() && !e.msgstr.is_empty())
        .collect();

    if to_translate.is_empty() && to_accept.is_empty() && repaired == 0 {
        println!("Nothing to translate.");
        return Ok(());
    }

    println!(
        "==> {} to translate, {} fuzzy accepted as-is, model={model}",
        to_translate.len(),
        to_accept.len()
    );

    let client = reqwest::Client::new();
    let total = to_translate.len();
    let total_chunks = total.div_ceil(args.batch).max(1);

    for (chunk_idx, chunk) in to_translate.chunks(args.batch).enumerate() {
        let msgids: Vec<&str> = chunk.iter().map(|e| e.msgid.as_str()).collect();
        println!("==> Chunk {}/{total_chunks} ({} entries)", chunk_idx + 1, chunk.len());

        match translate_batch(&client, &api_key, &model, &args.locale, &msgids).await {
            Ok(translated) => {
                for (entry, text) in chunk.iter().zip(translated.iter()) {
                    // If msgid ends with \n, msgstr must too — gettext requires it.
                    let text = if entry.msgid.ends_with('\n') && !text.ends_with('\n') {
                        format!("{text}\n")
                    } else {
                        text.clone()
                    };
                    translations.insert(entry.msgstr_line, text);
                }
                write_po(&lines, &raw, &translations, &to_translate, &to_accept, &args.po)?;
            }
            Err(e) => {
                eprintln!("  warning: chunk {} failed: {e}", chunk_idx + 1);
            }
        }
    }

    // Final write — handles to_accept fuzzy removals even when to_translate is empty
    write_po(&lines, &raw, &translations, &to_translate, &to_accept, &args.po)?;
    println!("==> Done: {}/{total} entries translated.", translations.len());
    Ok(())
}
