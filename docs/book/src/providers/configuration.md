# Provider Configuration

Every model provider is declared under `[providers.models.<name>]` in `~/.zeroclaw/config.toml`. The `<name>` is your own alias — it's how you reference the provider elsewhere in config (`default_model = "claude"`, `fallback_providers = ["claude", "local"]`, etc.).

## Minimum shape

```toml
[providers.models.<name>]
kind = "<provider-kind>"   # required — selects the implementation
model = "<model-id>"       # required — passed to the provider
```

Almost every provider also takes:

```toml
api_key = "..."            # or an env-var placeholder
base_url = "https://..."   # for OpenAI-compatible or self-hosted endpoints
```

## Kinds

| `kind` | Implementation | Notes |
|---|---|---|
| `anthropic` | `crates/zeroclaw-providers/src/anthropic.rs` | Accepts OAuth tokens (`sk-ant-oat*`) or API keys |
| `openai` | `openai.rs` | GPT, o-series |
| `ollama` | `ollama.rs` | Native `/api/chat`. Supports structured output via `format` |
| `openai-compatible` | `compatible.rs` | One impl for ~20 providers; set `base_url` and optionally `api_key` |
| `bedrock` | `bedrock.rs` | Uses AWS credentials chain (env, IAM role, profile) |
| `gemini` | `gemini.rs` | |
| `gemini-cli` | `gemini_cli.rs` | Shells out to `gemini` CLI; no API key needed |
| `azure-openai` | `azure_openai.rs` | Takes `base_url` + `api_version` + `deployment` |
| `copilot` | `copilot.rs` | OAuth flow built in |
| `openrouter` | `openrouter.rs` | Multi-vendor routing layer |
| `claude-code` | `claude_code.rs` | Delegates to a Claude Code session via MCP |
| `telnyx` | `telnyx.rs` | Voice AI via Telnyx |
| `kilocli` | `kilocli.rs` | Local KiloCLI inference |
| `reliable` | `reliable.rs` | Fallback-chain wrapper — see [Fallback & routing](./fallback-and-routing.md) |
| `router` | `router.rs` | Task-hint router — see [Fallback & routing](./fallback-and-routing.md) |

## Credentials

Four ways to supply credentials, in resolution order:

1. **Inline `api_key = "..."`** in the config entry (fine for dev, risky for checked-in configs)
2. **Config-level secrets store** — encrypted at `~/.zeroclaw/secrets` via a local key file
3. **Provider-specific env var** — `ANTHROPIC_API_KEY`, `ANTHROPIC_OAUTH_TOKEN`, `OPENAI_API_KEY`, `OPENROUTER_API_KEY`, `GROQ_API_KEY`, etc.
4. **Generic fallback** — `ZEROCLAW_API_KEY`, `API_KEY`

The onboarding wizard writes credentials to the secrets store by default. Config files you commit should use neither inline keys nor `env_passthrough` entries that leak user keys.

## OAuth and subscription auth

Several providers support OAuth / subscription-style tokens instead of raw API keys:

- **Anthropic** — `sk-ant-oat-*` OAuth tokens work anywhere an API key does. No cost if you're on a Pro/Team plan.
- **GitHub Copilot** — authenticate via the onboarding wizard (`zeroclaw onboard`) which handles the OAuth flow. The token is stored in the secrets backend.
- **Gemini CLI** — uses the `gemini` CLI's existing auth.
- **Claude Code** — uses your Claude Code login.

## Container-friendly overrides

The onboarding wizard detects Docker/Podman/Kubernetes and rewrites `localhost` to container-appropriate hostnames:

```toml
[providers.models.local]
kind = "ollama"
base_url = "http://host.docker.internal:11434"   # was "http://localhost:11434" on host
```

You can also force this manually at runtime:

```bash
ZEROCLAW_OLLAMA_BASE_URL=http://ollama:11434 zeroclaw agent
```

## Per-provider knobs

Beyond the universal fields, some providers accept extras. Highlights:

### Ollama

```toml
[providers.models.local]
kind = "ollama"
base_url = "http://localhost:11434"
model = "qwen3.6:35b-a3b"
think = false                # disable reasoning mode for faster output
reasoning_effort = "none"    # same intent, passed as a top-level field
options = { temperature = 0, num_ctx = 32768 }
```

### OpenAI-compatible

```toml
[providers.models.groq]
kind = "openai-compatible"
base_url = "https://api.groq.com/openai"
model = "llama-3.3-70b-versatile"
api_key = "gsk_..."
# Optional — supplies SSE tool-call streaming hints the endpoint understands
native_tool_streaming = true
```

### Azure OpenAI

```toml
[providers.models.azure]
kind = "azure-openai"
base_url = "https://my-resource.openai.azure.com"
deployment = "gpt-4o"
api_version = "2024-10-01-preview"
api_key = "..."
```

## Setting the fallback provider

`providers.fallback` names which `[providers.models.<name>]` entry the agent uses when no other routing rule applies. Onboarding configures your provider credentials and model but does **not** set this field — you set it manually after deciding which provider should be the default:

```toml
[providers]
fallback = "claude"   # must match a key under [providers.models.*]
```

Or from the command line:

```bash
zeroclaw config set providers.fallback claude
```

Until `providers.fallback` is set the agent will not know which provider to use and will error at startup. Configure at least one provider under `[providers.models.*]` and then set `providers.fallback` to its name.

Channels, tools, and SOPs can override the fallback on a per-request basis.

## See also

- [Overview](./overview.md)
- [Provider catalog](./catalog.md) — concrete config examples for every provider
- [Streaming](./streaming.md)
- [Fallback & routing](./fallback-and-routing.md)
