pub mod agent;
pub mod channel;
pub mod media;
pub mod memory_traits;
pub mod observability_traits;
pub mod peripherals_traits;
pub mod provider;
pub mod runtime_traits;
pub mod schema;
pub mod tool;

tokio::task_local! {
    pub static TOOL_LOOP_THREAD_ID: Option<String>;

    pub static TOOL_CHOICE_OVERRIDE: Option<String>;
}