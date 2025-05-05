use axum::{
    extract::{Path, State},
    http::{Request, StatusCode},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::{get, post, put, delete},
    Json, Router, Server,
};
use serde::{Deserialize, Serialize};
use sqlx::{postgres::PgPoolOptions, PgPool, Row};
use std::{env, net::SocketAddr, path::Path as StdPath};
use dotenvy::from_path;
use tower_http::cors::{Any, CorsLayer};

#[derive(Deserialize)]
struct RegisterRequest {
    username: String,
    password: String,
    email: String,
}

#[derive(Deserialize)]
struct LoginRequest {
    email: String,
    password: String,
}

#[derive(Serialize)]
struct LoginResponse {
    message: String,
    username: String,
}

#[derive(Serialize)]
struct ApiResponse {
    message: String,
}

#[derive(Deserialize)]
struct ConnectionRequest {
    name: String,
    description: Option<String>,
    leader_1: Option<String>,
    leader_2: Option<String>,
    leader_3: Option<String>,
    ip: String,
    port: String,
}

#[derive(Serialize, sqlx::FromRow)]
struct Connection {
    id: i32,
    name: String,
    description: Option<String>,
    leader_1: Option<String>,
    leader_2: Option<String>,
    leader_3: Option<String>,
    ip: String,
    port: String,
}

// Middleware simples de autenticação por token
async fn require_auth<B>(req: Request<B>, next: Next<B>) -> Result<Response, Response> {
    if let Some(auth) = req.headers().get("Authorization") {
        if auth == "Bearer token123" {
            return Ok(next.run(req).await);
        }
    }
    Err(StatusCode::UNAUTHORIZED.into_response())
}

async fn register(State(pool): State<PgPool>, Json(payload): Json<RegisterRequest>) -> Json<ApiResponse> {
    let user_id: (i32,) = sqlx::query_as("INSERT INTO users (username, password, email) VALUES ($1, $2, $3) RETURNING id")
        .bind(&payload.username)
        .bind(&payload.password)
        .bind(&payload.email)
        .fetch_one(&pool)
        .await
        .unwrap();

    sqlx::query("INSERT INTO user_permissions (user_id, ordens_compra, estoque, gestor_produto, notas_fiscais, auto_pregao) VALUES ($1, false, false, false, false, false)")
        .bind(user_id.0)
        .execute(&pool)
        .await
        .unwrap();

    Json(ApiResponse { message: "Usuário registrado com sucesso".into() })
}

async fn login(State(pool): State<PgPool>, Json(payload): Json<LoginRequest>) -> Json<LoginResponse> {
    let user = sqlx::query("SELECT username, password FROM users WHERE email = $1")
        .bind(&payload.email)
        .fetch_one(&pool)
        .await;

    match user {
        Ok(row) => {
            let stored_password: String = row.get("password");
            let username: String = row.get("username");

            if stored_password == payload.password {
                Json(LoginResponse {
                    message: "Login bem-sucedido".into(),
                    username,
                })
            } else {
                Json(LoginResponse {
                    message: "Senha incorreta".into(),
                    username: "".into(),
                })
            }
        }
        Err(_) => Json(LoginResponse {
            message: "Usuário não encontrado".into(),
            username: "".into(),
        }),
    }
}

async fn create_connection(State(pool): State<PgPool>, Json(payload): Json<ConnectionRequest>) -> Json<ApiResponse> {
    sqlx::query("INSERT INTO connections (name, description, leader_1, leader_2, leader_3, ip, port) VALUES ($1, $2, $3, $4, $5, $6, $7)")
        .bind(&payload.name)
        .bind(&payload.description)
        .bind(&payload.leader_1)
        .bind(&payload.leader_2)
        .bind(&payload.leader_3)
        .bind(&payload.ip)
        .bind(&payload.port)
        .execute(&pool)
        .await
        .unwrap();

    Json(ApiResponse { message: "Conexão criada com sucesso".into() })
}

async fn list_connections(State(pool): State<PgPool>) -> Json<Vec<Connection>> {
    let connections = sqlx::query_as::<_, Connection>("SELECT * FROM connections")
        .fetch_all(&pool)
        .await
        .unwrap();

    Json(connections)
}

async fn update_connection(State(pool): State<PgPool>, Path(id): Path<i32>, Json(payload): Json<ConnectionRequest>) -> Json<ApiResponse> {
    sqlx::query("UPDATE connections SET name = $1, description = $2, leader_1 = $3, leader_2 = $4, leader_3 = $5, ip = $6, port = $7 WHERE id = $8")
        .bind(&payload.name)
        .bind(&payload.description)
        .bind(&payload.leader_1)
        .bind(&payload.leader_2)
        .bind(&payload.leader_3)
        .bind(&payload.ip)
        .bind(&payload.port)
        .bind(id)
        .execute(&pool)
        .await
        .unwrap();

    Json(ApiResponse { message: "Conexão atualizada com sucesso".into() })
}

async fn delete_connection(State(pool): State<PgPool>, Path(id): Path<i32>) -> Json<ApiResponse> {
    sqlx::query("DELETE FROM connections WHERE id = $1")
        .bind(id)
        .execute(&pool)
        .await
        .unwrap();

    Json(ApiResponse { message: "Conexão deletada com sucesso".into() })
}

#[tokio::main]
async fn main() {
    from_path(StdPath::new("../.env")).expect("Falha ao carregar .env");
    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL não definida");
    let pool = PgPoolOptions::new().connect(&db_url).await.unwrap();

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let protected_routes = Router::new()
        .route("/containers", post(create_connection))
        .route("/containers", get(list_connections))
        .route("/containers/:id", put(update_connection))
        .route("/containers/:id", delete(delete_connection))
        .layer(middleware::from_fn(require_auth));

    let app = Router::new()
        .route("/register", post(register))
        .route("/login", post(login))
        .merge(protected_routes)
        .with_state(pool)
        .layer(cors);

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
