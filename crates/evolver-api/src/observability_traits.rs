use std::time::Duration;

#[derive(Debug, Clone)]
pub enum ObserverEvent {
    AgentStart {
        provider: String,
        model: String,
    },

    LlmRequest {
        provider: String,
        model: String,
        messages_count: usize,
    },

    LlmResponse {
        provider: String,
        model: String,
        duration: Duration,
        success: bool,
        error_message: Option<String>,
        input_tokens: Option<u64>,
        output_tokens: Option<u64>,
    },

    AgentEnd {
        provider: String,
        model: String,
        duration: Duration,
        tokens_used: Option<u64>,
        cost_usd: Option<f64>,
    },

    ToolCallStart {
        tool: String,
        arguments: Option<String>,
    },

    ToolCall {
        tool: String,
        duration: Duration,
        success: bool,
    },

    TurnComplete,

    ChannelMessage {
        channel: String,
        // inbound or outbound
        direction: String,
    },

    HeartbeatTick,

    CacheHit {
        // `"hot"` (in-memory) or "warm" SQLite
        cache_type: String,
        tokens_saved: u64,
    },

    CacheMiss {
        cache_type: String,
    },

    Error {
        component: String,
        message: String,
    },

    HandCompleted {
        hand_name: String,
        duration_ms: u64,
        findings_count: usize,
    },

    HandFailed {
        hand_name: String,
        error: String,
        duration_ms: u64,
    },

    DeploymentStarted {
        deploy_id: String,
    },

    DeploymentCompleted {
        deploy_id: String,
        commit_sha: String,
    },

    DeploymentFailed {
        deploy_id: String,
        reason: String,
    },

    RecoveryCompleted {
        deploy_id: String,
    },
}

#[derive(Debug, Clone)]
pub enum ObserverMetric {
    RequestLatency(Duration),

    TokensUsed(u64),

    ActiveSessions(u64),

    QueueDepth(u64),

    HandRunDuration {
        hand_name: String,
        duration: Duration,
    },

    HandFindingsCount {
        hand_name: String,
        count: u64,
    },

    HandSuccessRate {
        hand_name: String,
        success: bool,
    },

    DeploymentLeadTime(Duration),

    RecoveryTime(Duration),
}

pub trait Observer: Send + Sync + 'static {
    fn record_event(&self, event: &ObserverEvent);

    fn record_metric(&self, metric: &ObserverMetric);

    fn flush(&self) {}

    fn name(&self) -> &str;

    fn as_any(&self) -> &dyn std::any::Any;
}
