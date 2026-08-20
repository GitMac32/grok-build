//! Tests for multi-provider support and custom endpoint configurations.

#[cfg(test)]
mod tests {
    use crate::agent::config::{Config, resolve_credentials, resolve_model_list};

    #[test]
    fn openai_compatible_custom_endpoint() {
        let raw_config: toml::Value = toml::from_str(
            r#"
            [cli]
            remote_fetch_enabled = false

            [model.gpt4-custom]
            model = "gpt-4"
            base_url = "https://api.openai.com/v1"
            api_key = "sk-proj-custom"
            context_window = 128000
            "#,
        )
        .unwrap();

        let cfg = Config::new_from_toml_cfg(&raw_config).expect("config should parse");
        let resolved = resolve_model_list(&cfg, None);
        let model = resolved.get("gpt4-custom").expect("model should exist");
        assert_eq!(model.info.base_url, "https://api.openai.com/v1");
        assert_eq!(model.info.context_window.get(), 128000);
        let creds = resolve_credentials(model, None);
        assert_eq!(creds.api_key.as_deref(), Some("sk-proj-custom"));
    }

    #[test]
    fn local_ollama_endpoint() {
        let raw_config: toml::Value = toml::from_str(
            r#"
            [cli]
            remote_fetch_enabled = false

            [model.local-llama]
            model = "llama2"
            base_url = "http://localhost:11434/v1"
            context_window = 4096
            "#,
        )
        .unwrap();

        let cfg = Config::new_from_toml_cfg(&raw_config).expect("config should parse");
        let resolved = resolve_model_list(&cfg, None);
        let model = resolved.get("local-llama").expect("model should exist");
        assert_eq!(model.info.base_url, "http://localhost:11434/v1");
        assert_eq!(model.info.context_window.get(), 4096);
        // No credentials needed for local Ollama
        let creds = resolve_credentials(model, None);
        assert_eq!(creds.api_key, None);
    }

    #[test]
    fn env_key_single_var() {
        let raw_config: toml::Value = toml::from_str(
            r#"
            [model.env-authed]
            model = "test"
            base_url = "https://api.example.com/v1"
            env_key = "TEST_API_KEY"
            context_window = 100000
            "#,
        )
        .unwrap();

        let cfg = Config::new_from_toml_cfg(&raw_config).expect("config should parse");
        let resolved = resolve_model_list(&cfg, None);
        let model = resolved.get("env-authed").expect("model should exist");
        assert_eq!(model.info.base_url, "https://api.example.com/v1");
        // Env var resolution happens at runtime, so we just verify the config parsed
        let creds = resolve_credentials(model, None);
        // Empty since TEST_API_KEY isn't set in this test
        assert_eq!(creds.api_key, None);
    }

    #[test]
    fn env_key_array_fallbacks() {
        let raw_config: toml::Value = toml::from_str(
            r#"
            [model.fallback-env]
            model = "test"
            base_url = "https://api.example.com/v1"
            env_key = ["PRIMARY_VAR", "SECONDARY_VAR", "FALLBACK_VAR"]
            context_window = 100000
            "#,
        )
        .unwrap();

        let cfg = Config::new_from_toml_cfg(&raw_config).expect("config should parse");
        let resolved = resolve_model_list(&cfg, None);
        let model = resolved.get("fallback-env").expect("model should exist");
        // The model should be configured with env_key array
        assert!(model.env_key.is_some(), "env_key should be set");
    }

    #[test]
    fn provider_with_static_key_inheritance() {
        let raw_config: toml::Value = toml::from_str(
            r#"
            [model_providers.azure]
            base_url = "https://myorg.openai.azure.com/v1"
            api_key = "azure-admin-key"
            context_window = 100000

            [model.gpt4-azure]
            model = "gpt-4"
            model_provider = "azure"
            "#,
        )
        .unwrap();

        let cfg = Config::new_from_toml_cfg(&raw_config).expect("config should parse");
        let resolved = resolve_model_list(&cfg, None);
        let model = resolved.get("gpt4-azure").expect("model should exist");
        assert_eq!(model.info.base_url, "https://myorg.openai.azure.com/v1");
        let creds = resolve_credentials(model, None);
        assert_eq!(creds.api_key.as_deref(), Some("azure-admin-key"));
    }

    #[test]
    fn model_overrides_provider_context_window() {
        let raw_config: toml::Value = toml::from_str(
            r#"
            [model_providers.gateway]
            base_url = "https://gateway.example/v1"
            api_key = "gateway-key"
            context_window = 100000

            [model.small-context]
            model = "llama"
            model_provider = "gateway"
            context_window = 4096
            "#,
        )
        .unwrap();

        let cfg = Config::new_from_toml_cfg(&raw_config).expect("config should parse");
        let resolved = resolve_model_list(&cfg, None);
        let model = resolved.get("small-context").expect("model should exist");
        // Model's context_window should override provider's
        assert_eq!(model.info.context_window.get(), 4096);
        assert_eq!(model.info.base_url, "https://gateway.example/v1");
    }

    #[test]
    fn multiple_models_same_provider() {
        let raw_config: toml::Value = toml::from_str(
            r#"
            [model_providers.ollama]
            base_url = "http://localhost:11434/v1"
            context_window = 4096

            [model.llama2]
            model = "llama2"
            model_provider = "ollama"

            [model.mistral]
            model = "mistral"
            model_provider = "ollama"
            context_window = 8192
            "#,
        )
        .unwrap();

        let cfg = Config::new_from_toml_cfg(&raw_config).expect("config should parse");
        let resolved = resolve_model_list(&cfg, None);

        let llama = resolved.get("llama2").expect("llama2 should exist");
        assert_eq!(llama.info.base_url, "http://localhost:11434/v1");
        assert_eq!(llama.info.context_window.get(), 4096);

        let mistral = resolved.get("mistral").expect("mistral should exist");
        assert_eq!(mistral.info.base_url, "http://localhost:11434/v1");
        // Mistral overrides context_window
        assert_eq!(mistral.info.context_window.get(), 8192);
    }

    #[test]
    fn custom_endpoint_no_auth() {
        let raw_config: toml::Value = toml::from_str(
            r#"
            [cli]
            remote_fetch_enabled = false

            [model.internal-llm]
            model = "internal"
            base_url = "https://internal.corp.example/v1"
            context_window = 64000
            "#,
        )
        .unwrap();

        let cfg = Config::new_from_toml_cfg(&raw_config).expect("config should parse");
        let resolved = resolve_model_list(&cfg, None);
        let model = resolved.get("internal-llm").expect("model should exist");
        assert_eq!(model.info.base_url, "https://internal.corp.example/v1");
        let creds = resolve_credentials(model, Some("session-token"));
        // Should NOT include session token for custom endpoint
        assert_eq!(creds.api_key, None, "session token must not leak to custom endpoint");
    }
}
