#[derive(Debug, Clone)]
pub enum TurnEvent {
    Chunk {
        delta: String,
    },

    Thinking {
        delta: String,
    },

    ToolCall {
        name: String,
        args: serde_json::Value,
    },

    ToolResult {
        name: String,
        output: String,
    },
}
