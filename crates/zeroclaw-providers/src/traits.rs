// Data types re-exported from zeroclaw-types for incremental compilation.
pub use zeroclaw_types::provider::{
    ChatMessage, ChatRequest, ChatResponse, ConversationMessage, ProviderCapabilities,
    StreamChunk, StreamEvent, StreamOptions, ToolCall, ToolResultMessage, ToolsPayload,
    TokenUsage,
};
pub use zeroclaw_types::tool::ToolSpec;

use async_trait::async_trait;
use futures_util::{StreamExt, stream};
use std::fmt::Write;

/// Result type for streaming operations.
pub type StreamResult<T> = std::result::Result<T, StreamError>;

/// Errors that can occur during streaming.
#[derive(Debug, thiserror::Error)]
pub enum StreamError {
    #[error("HTTP error: {0}")]
    Http(reqwest::Error),

    #[error("JSON parse error: {0}")]
    Json(serde_json::Error),

    #[error("Invalid SSE format: {0}")]
    InvalidSse(String),

    #[error("Provider error: {0}")]
    Provider(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

/// Structured error returned when a requested capability is not supported.
#[derive(Debug, Clone, thiserror::Error)]
#[error("provider_capability_error provider={provider} capability={capability} message={message}")]
pub struct ProviderCapabilityError {
    pub provider: String,
    pub capability: String,
    pub message: String,
}

#[async_trait]
pub trait Provider: Send + Sync {
    /// Query provider capabilities.
    ///
    /// Default implementation returns minimal capabilities (no native tool calling).
    /// Providers should override this to declare their actual capabilities.
    fn capabilities(&self) -> ProviderCapabilities {
        ProviderCapabilities::default()
    }

    /// Convert tool specifications to provider-native format.
    ///
    /// Default implementation returns `PromptGuided` payload, which injects
    /// tool documentation into the system prompt as text. Providers with
    /// native tool calling support should override this to return their
    /// specific format (Gemini, Anthropic, OpenAI).
    fn convert_tools(&self, tools: &[ToolSpec]) -> ToolsPayload {
        ToolsPayload::PromptGuided {
            instructions: build_tool_instructions_text(tools),
        }
    }

    /// Simple one-shot chat (single user message, no explicit system prompt).
    ///
    /// This is the preferred API for non-agentic direct interactions.
    async fn simple_chat(
        &self,
        message: &str,
        model: &str,
        temperature: f64,
    ) -> anyhow::Result<String> {
        self.chat_with_system(None, message, model, temperature)
            .await
    }

    /// One-shot chat with optional system prompt.
    ///
    /// Kept for compatibility and advanced one-shot prompting.
    async fn chat_with_system(
        &self,
        system_prompt: Option<&str>,
        message: &str,
        model: &str,
        temperature: f64,
    ) -> anyhow::Result<String>;

    /// Multi-turn conversation. Default implementation extracts the last user
    /// message and delegates to `chat_with_system`.
    async fn chat_with_history(
        &self,
        messages: &[ChatMessage],
        model: &str,
        temperature: f64,
    ) -> anyhow::Result<String> {
        let system = messages
            .iter()
            .find(|m| m.role == "system")
            .map(|m| m.content.as_str());
        let last_user = messages
            .iter()
            .rfind(|m| m.role == "user")
            .map(|m| m.content.as_str())
            .unwrap_or("");
        self.chat_with_system(system, last_user, model, temperature)
            .await
    }

    /// Structured chat API for agent loop callers.
    async fn chat(
        &self,
        request: ChatRequest<'_>,
        model: &str,
        temperature: f64,
    ) -> anyhow::Result<ChatResponse> {
        // If tools are provided but provider doesn't support native tools,
        // inject tool instructions into system prompt as fallback.
        if let Some(tools) = request.tools {
            if !tools.is_empty() && !self.supports_native_tools() {
                let tool_instructions = match self.convert_tools(tools) {
                    ToolsPayload::PromptGuided { instructions } => instructions,
                    payload => {
                        anyhow::bail!(
                            "Provider returned non-prompt-guided tools payload ({payload:?}) while supports_native_tools() is false"
                        )
                    }
                };
                let mut modified_messages = request.messages.to_vec();

                // Inject tool instructions into an existing system message.
                // If none exists, prepend one to the conversation.
                if let Some(system_message) =
                    modified_messages.iter_mut().find(|m| m.role == "system")
                {
                    if !system_message.content.is_empty() {
                        system_message.content.push_str("\n\n");
                    }
                    system_message.content.push_str(&tool_instructions);
                } else {
                    modified_messages.insert(0, ChatMessage::system(tool_instructions));
                }

                let text = self
                    .chat_with_history(&modified_messages, model, temperature)
                    .await?;
                return Ok(ChatResponse {
                    text: Some(text),
                    tool_calls: Vec::new(),
                    usage: None,
                    reasoning_content: None,
                });
            }
        }

        let text = self
            .chat_with_history(request.messages, model, temperature)
            .await?;
        Ok(ChatResponse {
            text: Some(text),
            tool_calls: Vec::new(),
            usage: None,
            reasoning_content: None,
        })
    }

    /// Whether provider supports native tool calls over API.
    fn supports_native_tools(&self) -> bool {
        self.capabilities().native_tool_calling
    }

    /// Whether provider supports multimodal vision input.
    fn supports_vision(&self) -> bool {
        self.capabilities().vision
    }

    /// Warm up the HTTP connection pool (TLS handshake, DNS, HTTP/2 setup).
    /// Default implementation is a no-op; providers with HTTP clients should override.
    async fn warmup(&self) -> anyhow::Result<()> {
        Ok(())
    }

    /// Chat with tool definitions for native function calling support.
    /// The default implementation falls back to chat_with_history and returns
    /// an empty tool_calls vector (prompt-based tool use only).
    async fn chat_with_tools(
        &self,
        messages: &[ChatMessage],
        _tools: &[serde_json::Value],
        model: &str,
        temperature: f64,
    ) -> anyhow::Result<ChatResponse> {
        let text = self.chat_with_history(messages, model, temperature).await?;
        Ok(ChatResponse {
            text: Some(text),
            tool_calls: Vec::new(),
            usage: None,
            reasoning_content: None,
        })
    }

    /// Whether provider supports streaming responses.
    /// Default implementation returns false.
    fn supports_streaming(&self) -> bool {
        false
    }

    /// Whether provider can emit structured tool-call stream events.
    ///
    /// Providers should return true only when `stream_chat(...)` can produce
    /// `StreamEvent::ToolCall` for native tool-calling requests.
    fn supports_streaming_tool_events(&self) -> bool {
        false
    }

    /// Streaming chat with optional system prompt.
    /// Returns an async stream of text chunks.
    /// Default implementation falls back to non-streaming chat.
    fn stream_chat_with_system(
        &self,
        _system_prompt: Option<&str>,
        _message: &str,
        _model: &str,
        _temperature: f64,
        _options: StreamOptions,
    ) -> stream::BoxStream<'static, StreamResult<StreamChunk>> {
        // Default: return an empty stream (not supported)
        stream::empty().boxed()
    }

    /// Streaming chat with history.
    /// Default implementation extracts the last user message and delegates to
    /// `stream_chat_with_system`, mirroring the non-streaming `chat_with_history`.
    fn stream_chat_with_history(
        &self,
        messages: &[ChatMessage],
        model: &str,
        temperature: f64,
        options: StreamOptions,
    ) -> stream::BoxStream<'static, StreamResult<StreamChunk>> {
        let system = messages
            .iter()
            .find(|m| m.role == "system")
            .map(|m| m.content.as_str());
        let last_user = messages
            .iter()
            .rfind(|m| m.role == "user")
            .map(|m| m.content.as_str())
            .unwrap_or("");
        self.stream_chat_with_system(system, last_user, model, temperature, options)
    }

    /// Structured streaming chat interface.
    ///
    /// Default implementation adapts legacy text chunks from
    /// `stream_chat_with_history` into `StreamEvent::TextDelta` / `Final`.
    fn stream_chat(
        &self,
        request: ChatRequest<'_>,
        model: &str,
        temperature: f64,
        options: StreamOptions,
    ) -> stream::BoxStream<'static, StreamResult<StreamEvent>> {
        self.stream_chat_with_history(request.messages, model, temperature, options)
            .map(|chunk_result| chunk_result.map(StreamEvent::from_chunk))
            .boxed()
    }
}

/// Build tool instructions text for prompt-guided tool calling.
///
/// Generates a formatted text block describing available tools and how to
/// invoke them using XML-style tags. This is used as a fallback when the
/// provider doesn't support native tool calling.
pub fn build_tool_instructions_text(tools: &[ToolSpec]) -> String {
    let mut instructions = String::new();

    instructions.push_str("## Tool Use Protocol\n\n");
    instructions.push_str("To use a tool, wrap a JSON object in <tool_call></tool_call> tags:\n\n");
    instructions.push_str("<tool_call>\n");
    instructions.push_str(r#"{"name": "tool_name", "arguments": {"param": "value"}}"#);
    instructions.push_str("\n</tool_call>\n\n");
    instructions.push_str("You may use multiple tool calls in a single response. ");
    instructions.push_str("After tool execution, results appear in <tool_result> tags. ");
    instructions
        .push_str("Continue reasoning with the results until you can give a final answer.\n\n");
    instructions.push_str("### Available Tools\n\n");

    for tool in tools {
        writeln!(&mut instructions, "**{}**: {}", tool.name, tool.description)
            .expect("writing to String cannot fail");

        let parameters =
            serde_json::to_string(&tool.parameters).unwrap_or_else(|_| "{}".to_string());
        writeln!(&mut instructions, "Parameters: `{parameters}`")
            .expect("writing to String cannot fail");
        instructions.push('\n');
    }

    instructions
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures_util::StreamExt;

    struct CapabilityMockProvider;

    #[async_trait]
    impl Provider for CapabilityMockProvider {
        fn capabilities(&self) -> ProviderCapabilities {
            ProviderCapabilities {
                native_tool_calling: true,
                vision: true,
                prompt_caching: false,
            }
        }

        async fn chat_with_system(
            &self,
            _system_prompt: Option<&str>,
            _message: &str,
            _model: &str,
            _temperature: f64,
        ) -> anyhow::Result<String> {
            Ok("ok".into())
        }
    }

    #[test]
    fn chat_message_constructors() {
        let sys = ChatMessage::system("Be helpful");
        assert_eq!(sys.role, "system");
        assert_eq!(sys.content, "Be helpful");

        let user = ChatMessage::user("Hello");
        assert_eq!(user.role, "user");

        let asst = ChatMessage::assistant("Hi there");
        assert_eq!(asst.role, "assistant");

        let tool = ChatMessage::tool("{}");
        assert_eq!(tool.role, "tool");
    }

    #[test]
    fn chat_response_helpers() {
        let empty = ChatResponse {
            text: None,
            tool_calls: vec![],
            usage: None,
            reasoning_content: None,
        };
        assert!(!empty.has_tool_calls());
        assert_eq!(empty.text_or_empty(), "");

        let with_tools = ChatResponse {
            text: Some("Let me check".into()),
            tool_calls: vec![ToolCall {
                id: "1".into(),
                name: "shell".into(),
                arguments: "{}".into(),
            }],
            usage: None,
            reasoning_content: None,
        };
        assert!(with_tools.has_tool_calls());
        assert_eq!(with_tools.text_or_empty(), "Let me check");
    }

    #[test]
    fn token_usage_default_is_none() {
        let usage = TokenUsage::default();
        assert!(usage.input_tokens.is_none());
        assert!(usage.output_tokens.is_none());
    }

    #[test]
    fn chat_response_with_usage() {
        let resp = ChatResponse {
            text: Some("Hello".into()),
            tool_calls: vec![],
            usage: Some(TokenUsage {
                input_tokens: Some(100),
                output_tokens: Some(50),
                cached_input_tokens: None,
            }),
            reasoning_content: None,
        };
        assert_eq!(resp.usage.as_ref().unwrap().input_tokens, Some(100));
        assert_eq!(resp.usage.as_ref().unwrap().output_tokens, Some(50));
    }

    #[test]
    fn tool_call_serialization() {
        let tc = ToolCall {
            id: "call_123".into(),
            name: "file_read".into(),
            arguments: r#"{"path":"test.txt"}"#.into(),
        };
        let json = serde_json::to_string(&tc).unwrap();
        assert!(json.contains("call_123"));
        assert!(json.contains("file_read"));
    }

    #[test]
    fn conversation_message_variants() {
        let chat = ConversationMessage::Chat(ChatMessage::user("hi"));
        let json = serde_json::to_string(&chat).unwrap();
        assert!(json.contains("\"type\":\"Chat\""));

        let tool_result = ConversationMessage::ToolResults(vec![ToolResultMessage {
            tool_call_id: "1".into(),
            content: "done".into(),
        }]);
        let json = serde_json::to_string(&tool_result).unwrap();
        assert!(json.contains("\"type\":\"ToolResults\""));
    }

    #[test]
    fn provider_capabilities_default() {
        let caps = ProviderCapabilities::default();
        assert!(!caps.native_tool_calling);
        assert!(!caps.vision);
    }

    #[test]
    fn provider_capabilities_equality() {
        let caps1 = ProviderCapabilities {
            native_tool_calling: true,
            vision: false,
            prompt_caching: false,
        };
        let caps2 = ProviderCapabilities {
            native_tool_calling: true,
            vision: false,
            prompt_caching: false,
        };
        let caps3 = ProviderCapabilities {
            native_tool_calling: false,
            vision: false,
            prompt_caching: false,
        };

        assert_eq!(caps1, caps2);
        assert_ne!(caps1, caps3);
    }

    #[test]
    fn supports_native_tools_reflects_capabilities_default_mapping() {
        let provider = CapabilityMockProvider;
        assert!(provider.supports_native_tools());
    }

    #[test]
    fn supports_vision_reflects_capabilities_default_mapping() {
        let provider = CapabilityMockProvider;
        assert!(provider.supports_vision());
    }

    #[test]
    fn tools_payload_variants() {
        // Test Gemini variant
        let gemini = ToolsPayload::Gemini {
            function_declarations: vec![serde_json::json!({"name": "test"})],
        };
        assert!(matches!(gemini, ToolsPayload::Gemini { .. }));

        // Test Anthropic variant
        let anthropic = ToolsPayload::Anthropic {
            tools: vec![serde_json::json!({"name": "test"})],
        };
        assert!(matches!(anthropic, ToolsPayload::Anthropic { .. }));

        // Test OpenAI variant
        let openai = ToolsPayload::OpenAI {
            tools: vec![serde_json::json!({"type": "function"})],
        };
        assert!(matches!(openai, ToolsPayload::OpenAI { .. }));

        // Test PromptGuided variant
        let prompt_guided = ToolsPayload::PromptGuided {
            instructions: "Use tools...".to_string(),
        };
        assert!(matches!(prompt_guided, ToolsPayload::PromptGuided { .. }));
    }

    #[test]
    fn build_tool_instructions_text_format() {
        let tools = vec![
            ToolSpec {
                name: "shell".to_string(),
                description: "Execute commands".to_string(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "command": {"type": "string"}
                    }
                }),
            },
            ToolSpec {
                name: "file_read".to_string(),
                description: "Read files".to_string(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "path": {"type": "string"}
                    }
                }),
            },
        ];

        let instructions = build_tool_instructions_text(&tools);

        // Check for protocol description
        assert!(instructions.contains("Tool Use Protocol"));
        assert!(instructions.contains("<tool_call>"));
        assert!(instructions.contains("</tool_call>"));

        // Check for tool listings
        assert!(instructions.contains("**shell**"));
        assert!(instructions.contains("Execute commands"));
        assert!(instructions.contains("**file_read**"));
        assert!(instructions.contains("Read files"));

        // Check for parameters
        assert!(instructions.contains("Parameters:"));
        assert!(instructions.contains(r#""type":"object""#));
    }

    #[test]
    fn build_tool_instructions_text_empty() {
        let instructions = build_tool_instructions_text(&[]);

        // Should still have protocol description
        assert!(instructions.contains("Tool Use Protocol"));

        // Should have empty tools section
        assert!(instructions.contains("Available Tools"));
    }

    // Mock provider for testing.
    struct MockProvider {
        supports_native: bool,
    }

    #[async_trait]
    impl Provider for MockProvider {
        fn supports_native_tools(&self) -> bool {
            self.supports_native
        }

        async fn chat_with_system(
            &self,
            _system: Option<&str>,
            _message: &str,
            _model: &str,
            _temperature: f64,
        ) -> anyhow::Result<String> {
            Ok("response".to_string())
        }
    }

    #[test]
    fn provider_convert_tools_default() {
        let provider = MockProvider {
            supports_native: false,
        };

        let tools = vec![ToolSpec {
            name: "test_tool".to_string(),
            description: "A test tool".to_string(),
            parameters: serde_json::json!({"type": "object"}),
        }];

        let payload = provider.convert_tools(&tools);

        // Default implementation should return PromptGuided.
        assert!(matches!(payload, ToolsPayload::PromptGuided { .. }));

        if let ToolsPayload::PromptGuided { instructions } = payload {
            assert!(instructions.contains("test_tool"));
            assert!(instructions.contains("A test tool"));
        }
    }

    #[tokio::test]
    async fn provider_chat_prompt_guided_fallback() {
        let provider = MockProvider {
            supports_native: false,
        };

        let tools = vec![ToolSpec {
            name: "shell".to_string(),
            description: "Run commands".to_string(),
            parameters: serde_json::json!({"type": "object"}),
        }];

        let request = ChatRequest {
            messages: &[ChatMessage::user("Hello")],
            tools: Some(&tools),
        };

        let response = provider.chat(request, "model", 0.7).await.unwrap();

        // Should return a response (default impl calls chat_with_history).
        assert!(response.text.is_some());
    }

    #[tokio::test]
    async fn provider_chat_without_tools() {
        let provider = MockProvider {
            supports_native: true,
        };

        let request = ChatRequest {
            messages: &[ChatMessage::user("Hello")],
            tools: None,
        };

        let response = provider.chat(request, "model", 0.7).await.unwrap();

        // Should work normally without tools.
        assert!(response.text.is_some());
    }

    // Provider that echoes the system prompt for assertions.
    struct EchoSystemProvider {
        supports_native: bool,
    }

    #[async_trait]
    impl Provider for EchoSystemProvider {
        fn supports_native_tools(&self) -> bool {
            self.supports_native
        }

        async fn chat_with_system(
            &self,
            system: Option<&str>,
            _message: &str,
            _model: &str,
            _temperature: f64,
        ) -> anyhow::Result<String> {
            Ok(system.unwrap_or_default().to_string())
        }
    }

    // Provider with custom prompt-guided conversion.
    struct CustomConvertProvider;

    #[async_trait]
    impl Provider for CustomConvertProvider {
        fn supports_native_tools(&self) -> bool {
            false
        }

        fn convert_tools(&self, _tools: &[ToolSpec]) -> ToolsPayload {
            ToolsPayload::PromptGuided {
                instructions: "CUSTOM_TOOL_INSTRUCTIONS".to_string(),
            }
        }

        async fn chat_with_system(
            &self,
            system: Option<&str>,
            _message: &str,
            _model: &str,
            _temperature: f64,
        ) -> anyhow::Result<String> {
            Ok(system.unwrap_or_default().to_string())
        }
    }

    // Provider returning an invalid payload for non-native mode.
    struct InvalidConvertProvider;

    #[async_trait]
    impl Provider for InvalidConvertProvider {
        fn supports_native_tools(&self) -> bool {
            false
        }

        fn convert_tools(&self, _tools: &[ToolSpec]) -> ToolsPayload {
            ToolsPayload::OpenAI {
                tools: vec![serde_json::json!({"type": "function"})],
            }
        }

        async fn chat_with_system(
            &self,
            _system: Option<&str>,
            _message: &str,
            _model: &str,
            _temperature: f64,
        ) -> anyhow::Result<String> {
            Ok("should_not_reach".to_string())
        }
    }

    #[tokio::test]
    async fn provider_chat_prompt_guided_preserves_existing_system_not_first() {
        let provider = EchoSystemProvider {
            supports_native: false,
        };

        let tools = vec![ToolSpec {
            name: "shell".to_string(),
            description: "Run commands".to_string(),
            parameters: serde_json::json!({"type": "object"}),
        }];

        let request = ChatRequest {
            messages: &[
                ChatMessage::user("Hello"),
                ChatMessage::system("BASE_SYSTEM_PROMPT"),
            ],
            tools: Some(&tools),
        };

        let response = provider.chat(request, "model", 0.7).await.unwrap();
        let text = response.text.unwrap_or_default();

        assert!(text.contains("BASE_SYSTEM_PROMPT"));
        assert!(text.contains("Tool Use Protocol"));
    }

    #[tokio::test]
    async fn provider_chat_prompt_guided_uses_convert_tools_override() {
        let provider = CustomConvertProvider;

        let tools = vec![ToolSpec {
            name: "shell".to_string(),
            description: "Run commands".to_string(),
            parameters: serde_json::json!({"type": "object"}),
        }];

        let request = ChatRequest {
            messages: &[ChatMessage::system("BASE"), ChatMessage::user("Hello")],
            tools: Some(&tools),
        };

        let response = provider.chat(request, "model", 0.7).await.unwrap();
        let text = response.text.unwrap_or_default();

        assert!(text.contains("BASE"));
        assert!(text.contains("CUSTOM_TOOL_INSTRUCTIONS"));
    }

    #[tokio::test]
    async fn provider_chat_prompt_guided_rejects_non_prompt_payload() {
        let provider = InvalidConvertProvider;

        let tools = vec![ToolSpec {
            name: "shell".to_string(),
            description: "Run commands".to_string(),
            parameters: serde_json::json!({"type": "object"}),
        }];

        let request = ChatRequest {
            messages: &[ChatMessage::user("Hello")],
            tools: Some(&tools),
        };

        let err = provider.chat(request, "model", 0.7).await.unwrap_err();
        let message = err.to_string();

        assert!(message.contains("non-prompt-guided"));
    }

    struct StreamingChunkOnlyProvider;

    #[async_trait]
    impl Provider for StreamingChunkOnlyProvider {
        async fn chat_with_system(
            &self,
            _system_prompt: Option<&str>,
            _message: &str,
            _model: &str,
            _temperature: f64,
        ) -> anyhow::Result<String> {
            Ok("ok".to_string())
        }

        fn supports_streaming(&self) -> bool {
            true
        }

        fn stream_chat_with_history(
            &self,
            _messages: &[ChatMessage],
            _model: &str,
            _temperature: f64,
            _options: StreamOptions,
        ) -> stream::BoxStream<'static, StreamResult<StreamChunk>> {
            stream::iter(vec![
                Ok(StreamChunk::delta("hello")),
                Ok(StreamChunk::final_chunk()),
            ])
            .boxed()
        }
    }

    #[tokio::test]
    async fn provider_stream_chat_default_maps_legacy_chunks_to_events() {
        let provider = StreamingChunkOnlyProvider;
        let mut stream = provider.stream_chat(
            ChatRequest {
                messages: &[ChatMessage::user("hi")],
                tools: None,
            },
            "model",
            0.0,
            StreamOptions::new(true),
        );

        let first = stream.next().await.unwrap().unwrap();
        let second = stream.next().await.unwrap().unwrap();
        assert!(stream.next().await.is_none());

        match first {
            StreamEvent::TextDelta(chunk) => assert_eq!(chunk.delta, "hello"),
            other => panic!("expected text delta event, got {other:?}"),
        }
        assert!(matches!(second, StreamEvent::Final));
    }
}
