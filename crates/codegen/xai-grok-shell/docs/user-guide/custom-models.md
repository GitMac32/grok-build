# Custom Models and Providers

Grok Build supports using OpenAI-compatible API providers, local models, and custom endpoints without being locked into xAI. This guide shows how to configure alternative LLM providers via your config file.

## Quick Start

### OpenAI API

```toml
# ~/.grok/config.toml
[model.gpt-4o]
model = "gpt-4o"
base_url = "https://api.openai.com/v1"
api_key = "sk-proj-..."
context_window = 128000
```

```bash
XAI_API_KEY= grok -m gpt-4o
```

### Ollama (Local)

```toml
# ~/.grok/config.toml
[model.local-llama]
model = "llama2"
base_url = "http://localhost:11434/v1"
context_window = 4096
```

```bash
# Start Ollama
ollama serve

# In another terminal
grok -m local-llama
```

### Using Environment Variables for API Keys

```toml
# ~/.grok/config.toml
[model.claude-via-proxy]
model = "claude-3-sonnet"
base_url = "https://litellm.example.com/v1"
env_key = "LITELLM_API_KEY"  # reads from this env var
context_window = 200000
```

```bash
export LITELLM_API_KEY="sk-..."
grok -m claude-via-proxy
```

### Multiple Environment Variable Fallbacks

If multiple env vars might hold the key, specify them in order:

```toml
[model.flexible-auth]
model = "my-model"
base_url = "https://api.example.com/v1"
env_key = ["MY_MODEL_KEY", "FALLBACK_KEY", "THIRD_OPTION"]
# Tries MY_MODEL_KEY first, then FALLBACK_KEY, then THIRD_OPTION
context_window = 256000
```

## Model Providers (Reusable Endpoint Definitions)

For multiple models sharing the same endpoint, use `[model_providers.*]` to avoid repetition:

```toml
# Define a provider once
[model_providers.ollama]
base_url = "http://localhost:11434/v1"
context_window = 4096

# Use it for multiple models
[model.ollama-llama2]
model = "llama2"
model_provider = "ollama"

[model.ollama-neural]
model = "neural-chat"
model_provider = "ollama"
```

### Provider with Static API Key

```toml
[model_providers.azure]
base_url = "https://myorg.openai.azure.com/v1"
api_key = "abc123key"
context_window = 100000

[model.gpt4-azure]
model = "gpt-4"
model_provider = "azure"
```

### Provider with Command-Based Auth (Rotating Tokens)

For providers that require fresh tokens (e.g., OAuth, OIDC):

```toml
[model_providers.corp-gateway]
base_url = "https://llm.corp.example/v1"
context_window = 200000

[model_providers.corp-gateway.auth]
command = "/usr/local/bin/get-corp-token"
token_ttl_secs = 3600
timeout_secs = 10

[model.corp-llm]
model = "internal-model"
model_provider = "corp-gateway"
```

Your command should output:
- **stdout**: a bare token or JSON `{"access_token": "...", "expires_in": 3600}`
- **stderr**: status messages or errors (shown to user)

Example helper:

```bash
#!/bin/bash
# /usr/local/bin/get-corp-token
echo "Getting token from corp auth..." >&2
TOKEN=$(curl -s https://auth.corp.example/token | jq -r .access_token)
echo "$TOKEN"
```

## Disable Remote Model Fetching

By default, Grok fetches the latest model list from your configured endpoint. To use only local config definitions:

```toml
# ~/.grok/config.toml
[cli]
remote_fetch_enabled = false
```

Now only models you explicitly define in `[model.*]` sections are available.

## API Backends

Most OpenAI-compatible endpoints use `chat_completions` (default). Some support advanced backends:

```toml
# Responses API (xAI, some custom backends)
[model_providers.xai]
base_url = "https://api.x.ai/v1"
api_backend = "responses"

# Chat Completions (OpenAI-standard)
[model_providers.openai]
base_url = "https://api.openai.com/v1"
api_backend = "chat_completions"

# Messages API (Anthropic)
[model_providers.anthropic]
base_url = "https://api.anthropic.com/v1"
api_backend = "messages"
```

## Model-Specific Overrides

Models inherit from their provider, but can override:

```toml
[model_providers.gateway]
base_url = "https://gateway.example/v1"
api_key = "default-key"
context_window = 100000
temperature = 0.7

# Override just the key and temperature for this model
[model.custom-variant]
model = "llama2"
model_provider = "gateway"
api_key = "model-specific-key"  # shadows provider's key
temperature = 0.9
context_window = 50000  # can also override context window
```

## Example Configurations

### All Local (No Remote)

```toml
[cli]
remote_fetch_enabled = false  # don't fetch from xAI

[model.ollama]
model = "llama2"
base_url = "http://localhost:11434/v1"
context_window = 4096

[model.mistral]
model = "mistral"
base_url = "http://localhost:11434/v1"
context_window = 8192
```

### Multi-Provider Setup

```toml
[cli]
remote_fetch_enabled = false

# Local fallback
[model_providers.local]
base_url = "http://localhost:11434/v1"

[model.local-llama]
model = "llama2"
model_provider = "local"
context_window = 4096

# Cloud backup
[model_providers.openai]
base_url = "https://api.openai.com/v1"

[model.openai-gpt4]
model = "gpt-4"
model_provider = "openai"
api_key = "sk-proj-..."
context_window = 128000

# Custom gateway
[model_providers.corp]
base_url = "https://llm.corp.example/v1"
auth_provider = "corp-token-helper"

[model.corp-model]
model = "internal-llm"
model_provider = "corp"
context_window = 64000
```

### Using env_key with Multiple Fallbacks

```toml
[model_providers.flexible]
base_url = "https://api.example.com/v1"

[model.robust-model]
model = "v1"
model_provider = "flexible"
env_key = ["PRIMARY_KEY", "SECONDARY_KEY", "FALLBACK_KEY"]
context_window = 200000
```

Grok will try each environment variable in order, using the first one that is set.

## Troubleshooting

### "Model not found"

If you defined a model in config but it's not appearing:

1. Check that `remote_fetch_enabled = false` if you want only local models
2. Ensure the `[model.*]` section is properly formatted
3. Run `grok inspect` to see all loaded models

### Authentication failures

- **Static key issues**: Verify the `api_key` string has no extra spaces
- **Env var issues**: Confirm the variable is exported: `export MY_KEY=value && grok ...`
- **Command-based auth**: Check stderr from your auth command: `grok -p "test" --debug-file /tmp/grok.log`

### Context window mismatch

If the model truncates input unexpectedly, ensure `context_window` matches the actual model capacity:

```bash
# Check what Grok sees
grok inspect models
```

### Certificate errors with self-signed endpoints

For internal/development endpoints with self-signed certs:

```bash
RUSTLS_CERTIFICATE_VALIDATION=none grok -m your-model
```

## See Also

- [Configuration Guide](./03-configuration.md) — Full config file reference
- [Authentication Guide](./02-authentication.md) — Advanced auth provider setup
