# Communication

Where to ask questions, file bugs, propose features, and reach the team.

## GitHub issues

The primary coordination surface.

- **Bug reports** — use the bug template (`.github/ISSUE_TEMPLATE/bug_report.yml`). Include `zeroclaw --version`, OS, and the output of `zeroclaw doctor`.
- **Feature requests** — use the feature template (`.github/ISSUE_TEMPLATE/feature_request.yml`). Focus on user value and constraints; implementation details are for RFCs or PR discussion.
- **RFCs** — see [RFC process](./rfcs.md).
- **Questions** — if it's more "how do I…" than "this is broken", prefer Discussions (below).

Search before filing. Duplicates get consolidated; the search box is your friend.

## GitHub Discussions

For design chatter, usage questions, "does anyone else see this" posts, and conceptual conversations that aren't yet an issue. Less formal than an issue, won't get lost like a Discord message.

[github.com/zeroclaw-labs/zeroclaw/discussions](https://github.com/zeroclaw-labs/zeroclaw/discussions)

## Discord

Real-time chat. Channels:

- `#general` — the default room
- `#help` — "I can't get X working" threads
- `#dev` — in-flight development discussion
- `#releases` — announcements, release notes, breaking-change pre-warnings

[Invite link in the repo README.](https://github.com/zeroclaw-labs/zeroclaw)

**Discord is ephemeral** — anything important should also land in an issue or discussion. Discord is great for "is anyone awake", "quick question", and social chatter; it is not a durable record.

## Maintainer contacts

Core maintainers and their focus areas:

| Handle | Focus |
|---|---|
| [@singlerider](https://github.com/singlerider) | Runtime, providers, infra |
| [@WareWolf-MoonWall](https://github.com/WareWolf-MoonWall) | Governance, docs, reviewer playbook |
| [@theonlyhennygod](https://github.com/theonlyhennygod) | Channels, gateway |
| [@JordanTheJet](https://github.com/JordanTheJet) | Hardware, edge deployments |

`@`-mention sparingly — CC maintainers only when the issue genuinely needs their attention. Default to letting the team triage.

## Security issues

Do not file public issues for security vulnerabilities.

Email: `security@zeroclaw.dev`

Include:

- Affected versions
- Reproduction (minimal, please)
- Impact assessment

We aim to acknowledge within 48 hours and publish a patch + advisory within 14 days for critical issues. Coordinated disclosure is appreciated.

See `SECURITY.md` in the repo root for the full policy.

## Release feed

Subscribe to the GitHub release feed to be notified when new versions ship:

```
https://github.com/zeroclaw-labs/zeroclaw/releases.atom
```

Or watch the repo on GitHub (Watch → Custom → Releases).

Release notes are cross-posted to Discord `#releases` and the community Twitter.

## Commercial support

None offered. ZeroClaw is maintained by the community. If you're deploying at scale and want SLAs, sponsor a maintainer directly or fund a dedicated support arrangement through the core team. Reach out via `hello@zeroclaw.dev`.

## Feedback

The fastest way to reach the team with open-ended feedback is a GitHub Discussion post tagged `feedback`. For "I tried to do X and it felt wrong" write-ups — the kind of input that surfaces UX issues the maintainers can't see from the code — that's where it goes.

## Contributor recognition

Everyone who's had a PR merged appears in the contributors list on the repo. For substantial contributions — features, RFCs, significant bug fixes — your handle shows up in the release notes.

## See also

- [How to contribute](./how-to.md)
- [RFC process](./rfcs.md)
- [Philosophy](../philosophy.md) — what the project is trying to be, so you know what's in scope
