use rmcp::{
    ErrorData as McpError,
    ServerHandler,
    handler::server::{
        router::tool::ToolRouter,
        wrapper::Parameters,
    },
    model::*,
    tool, tool_handler, tool_router,
};
use schemars::JsonSchema;
use serde::Deserialize;

#[derive(Deserialize, JsonSchema)]
pub struct SearchArxivParams {
    /// Search query
    pub query: String,
    /// Maximum number of results (default: 5, max: 20)
    pub max_results: Option<u32>,
}

#[derive(Clone)]
pub struct ArxivServer {
    #[allow(dead_code)]
    tool_router: ToolRouter<Self>,
}

#[tool_router]
impl ArxivServer {
    pub fn new() -> Self {
        Self {
            tool_router: Self::tool_router(),
        }
    }

    #[tool(description = "Search for academic papers on arXiv")]
    async fn search_arxiv(
        &self,
        Parameters(req): Parameters<SearchArxivParams>,
    ) -> Result<CallToolResult, McpError> {
        let max = req.max_results.unwrap_or(5).clamp(1, 20);

        match crate::arxiv_client::search(&req.query, max).await {
            Ok(papers) => {
                let json = serde_json::to_string_pretty(&papers)
                    .unwrap_or_else(|e| format!("Error serializing: {}", e));
                Ok(CallToolResult::success(vec![Content::text(json)]))
            }
            Err(e) => Ok(CallToolResult::error(vec![Content::text(
                format!("Search failed: {}", e),
            )])),
        }
    }
}

#[tool_handler]
impl ServerHandler for ArxivServer {
    fn get_info(&self) -> ServerInfo {
        let mut info = ServerInfo::default();
        info.capabilities = ServerCapabilities::builder().enable_tools().build();
        info.server_info = Implementation::from_build_env();
        info
    }
}
