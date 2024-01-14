use std::{collections::HashMap, sync::Arc};
use axum::{
    http::StatusCode,
    response::IntoResponse,
    Json, Router,
    extract::{Path, Query, State},
    routing::{delete, get, post}
};
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Clone)]
struct Tool {
    id: Uuid,
    title: String,
    link: String,
    description: String,
    tags: Vec<String>,
}

#[derive(Serialize, Deserialize)]
struct CreateTool {
    title: String,
    link: String,
    description: String,
    tags: Vec<String>,
}

#[derive(Deserialize)]
struct SearchToolsQuery {
    tag: Option<String>
}

type AppState = Arc<RwLock<HashMap<Uuid, Tool>>>;

#[tokio::main]
async fn main() {
    let mut tools: HashMap<Uuid, Tool> = HashMap::new();

    let tool = Tool {
        id: Uuid::now_v7(),
        title: String::from("Notion"),
        description: String::from("All in one tool"),
        link: String::from("https://notion.so"),
        tags: vec![String::from("text")],
    };

    tools.insert(tool.id, tool);

    let state: AppState = Arc::new(RwLock::new(tools));
    let app = Router::new()
        .route("/", get(|| async {"Hello world"}))
        .route("/tools", get(get_tools))
        .route("/tools", post(create_tool))
        .route("/tools/:id", delete(remove_tool))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn get_tools(State(tools): State<AppState>, Query(query): Query<SearchToolsQuery>) -> impl IntoResponse {
    let tools = tools.read().await.clone();
    let tag = query.tag;
    match tag {
        Some(tag) => {
            let filtered_tools = tools.into_iter().filter(|(_, tool)| {
                tool.tags.contains(&tag)
            }).collect();
            (StatusCode::OK, Json(filtered_tools))
        },
        None => (StatusCode::OK, Json(tools))
    }
}

async fn create_tool(
    State(tools): State<AppState>,
    Json(create_tool): Json<CreateTool>
)
-> impl IntoResponse {
    let id = Uuid::now_v7();
    let tool: Tool = Tool {
        id,
        title: create_tool.title,
        link: create_tool.link,
        description: create_tool.description,
        tags: create_tool.tags,

    };
    tools.write().await.insert(id, tool.clone());
    (StatusCode::CREATED, Json(tool))
}

async fn remove_tool(
    State(tools): State<AppState>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    match tools.write().await.remove(&id) {
        Some(_) => StatusCode::OK,
        None => StatusCode::NOT_FOUND
    }
}