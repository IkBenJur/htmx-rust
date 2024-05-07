use std::sync::{Arc, Mutex};

use askama::Template;
use axum::{
    extract::State,
    http::StatusCode,
    response::{Html, IntoResponse, Response},
    routing::{get, post},
    Form, Router,
};
use serde::Deserialize;
use tower_http::services::ServeDir;

struct AppState {
    todos: Mutex<Vec<String>>,
}

#[tokio::main]
async fn main() {
    // initialize tracing
    tracing_subscriber::fmt::init();

    let assets_path = std::env::current_dir().unwrap();
    let state = Arc::new(AppState {
        todos: Mutex::new(vec![
            String::from("rust"),
            String::from("rustt"),
            String::from("russst"),
        ]),
    });

    // build our application with a route
    let app = Router::new()
        // `GET /` goes to `root`
        .route("/", get(root))
        .route("/create-todo", post(create_todo))
        .with_state(state)
        .nest_service(
            "/styles",
            ServeDir::new(format!("{}/styles", assets_path.to_str().unwrap())),
        );

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

// A wrapper type that we'll use to encapsulate HTML parsed by askama into valid HTML for axum to serve.
struct HtmlTemplate<T>(T);

/// Allows us to convert Askama HTML templates into valid HTML for axum to serve in the response.
impl<T> IntoResponse for HtmlTemplate<T>
where
    T: Template,
{
    fn into_response(self) -> Response {
        // Attempt to render the template with askama
        match self.0.render() {
            // If we're able to successfully parse and aggregate the template, serve it
            Ok(html) => Html(html).into_response(),
            // If we're not, return an error or some bit of fallback HTML
            Err(err) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to render template. Error: {}", err),
            )
                .into_response(),
        }
    }
}

#[derive(Template, Clone)]
#[template(path = "todo-list.html")]
struct TodoList {
    todos: Vec<String>,
}

#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate {
    todos: Vec<String>,
}

async fn root(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let todos = state.todos.lock().unwrap();

    let template = IndexTemplate {
        todos: todos.clone(),
    };
    return HtmlTemplate(template);
}

#[derive(Deserialize)]
struct TodoForm {
    todo: String,
}

async fn create_todo(
    State(state): State<Arc<AppState>>,
    Form(form): Form<TodoForm>,
) -> impl IntoResponse {
    let mut lock = state.todos.lock().unwrap();
    lock.push(form.todo);

    let template = TodoList {
        todos: lock.clone(),
    };

    return HtmlTemplate(template);
}
