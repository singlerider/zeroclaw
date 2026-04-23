#!/usr/bin/env python3
"""Fill empty and fuzzy .po file entries with AI translations."""

import argparse
import json
import sys

try:
    import polib
except ImportError:
    print("error: polib not installed — run: pip install polib", file=sys.stderr)
    sys.exit(1)

try:
    import anthropic
except ImportError:
    print("error: anthropic not installed — run: pip install anthropic", file=sys.stderr)
    sys.exit(1)


LOCALE_NAMES = {
    "ja": "Japanese",
    "ko": "Korean",
    "zh": "Simplified Chinese",
    "zh-TW": "Traditional Chinese",
    "fr": "French",
    "de": "German",
    "es": "Spanish",
    "pt": "Portuguese",
    "ru": "Russian",
}

MODEL = "claude-haiku-4-5-20251001"
CHUNK_SIZE = 50


def needs_translation(entry):
    return "fuzzy" in entry.flags or not entry.msgstr or entry.msgstr.strip() == ""


def is_code_only(text):
    stripped = text.strip()
    return stripped.startswith("`") and stripped.endswith("`") and stripped.count("`") == 2


def translate_batch(client, numbered_pairs, locale_name):
    """Translate a list of (number, msgid) pairs. Returns {str(number): translation}."""
    items = "\n".join(f'{n}. {json.dumps(msgid)}' for n, msgid in numbered_pairs)

    prompt = f"""You are translating technical documentation strings for ZeroClaw, an AI agent/bot framework, into {locale_name}.

Rules:
- Preserve all markdown formatting exactly (**, *, `, [](), etc.)
- Do NOT translate content inside backticks
- Preserve any HTML entities or special characters
- Keep technical terms in English (ZeroClaw, Matrix, Mattermost, LINE, MCP, API, CLI, etc.)
- Return ONLY a JSON object mapping number (as string key) to translated string, no other text

Strings to translate:
{items}

Respond with a JSON object like: {{"1": "translated text", "2": "another translation"}}"""

    message = client.messages.create(
        model=MODEL,
        max_tokens=4096,
        messages=[{"role": "user", "content": prompt}],
    )

    response_text = message.content[0].text.strip()
    if response_text.startswith("```"):
        lines = response_text.split("\n")
        response_text = "\n".join(lines[1:-1])

    return json.loads(response_text)


def main():
    parser = argparse.ArgumentParser(description="AI-fill empty/fuzzy .po entries")
    parser.add_argument("--po", required=True, help="Path to .po file")
    parser.add_argument("--locale", required=True, help="Target locale code (e.g. ja)")
    args = parser.parse_args()

    locale_name = LOCALE_NAMES.get(args.locale, args.locale)

    po = polib.pofile(args.po)

    to_translate = [
        (i, entry)
        for i, entry in enumerate(po)
        if not entry.obsolete and needs_translation(entry) and not is_code_only(entry.msgid)
    ]

    if not to_translate:
        print(f"==> {args.locale}: nothing to translate")
        return

    print(f"==> {args.locale}: translating {len(to_translate)} entries via AI ({MODEL})")

    client = anthropic.Anthropic()

    translated_count = 0
    for chunk_start in range(0, len(to_translate), CHUNK_SIZE):
        chunk = to_translate[chunk_start:chunk_start + CHUNK_SIZE]
        numbered = [(j + 1, entry.msgid) for j, (_, entry) in enumerate(chunk)]

        try:
            results = translate_batch(client, numbered, locale_name)
        except Exception as exc:
            print(f"warning: translation chunk failed ({exc}); skipping", file=sys.stderr)
            continue

        for j, (orig_idx, entry) in enumerate(chunk):
            key = str(j + 1)
            if key not in results:
                continue
            entry.msgstr = results[key]
            if "fuzzy" in entry.flags:
                entry.flags.remove("fuzzy")
            translated_count += 1

    po.save(args.po)
    print(f"==> {args.locale}: wrote {translated_count} translations to {args.po}")


if __name__ == "__main__":
    main()
