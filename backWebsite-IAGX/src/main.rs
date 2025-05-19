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
use jsonwebtoken::{encode, EncodingKey, Header};
use chrono::{Utc, Duration};

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

#[derive(Serialize, Deserialize)]
struct Claims {
    sub: String,
    username: String,
    role: String,
    exp: usize,
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

#[derive(Serialize, sqlx::FromRow)]
struct UserProfile {
    id: i32,
    username: String,
    email: String,
    avatar_url: Option<String>,
}

async fn require_auth<B>(req: Request<B>, next: Next<B>) -> Result<Response, Response> {
    if let Some(auth) = req.headers().get("Authorization") {
        if let Ok(auth_str) = auth.to_str() {
            if auth_str.starts_with("Bearer ") {
                let token = auth_str.trim_start_matches("Bearer ");
                let key = b"seu_segredo_super_forte";
                let token_data = jsonwebtoken::decode::<Claims>(
                    token,
                    &jsonwebtoken::DecodingKey::from_secret(key),
                    &jsonwebtoken::Validation::default(),
                );
                if token_data.is_ok() {
                    return Ok(next.run(req).await);
                }
            }
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
    let user = sqlx::query("SELECT username, password, type FROM users WHERE email = $1")
        .bind(&payload.email)
        .fetch_one(&pool)
        .await;

    match user {
        Ok(row) => {
            let stored_password: String = row.get("password");
            let username: String = row.get("username");
            let role: String = row.try_get("type").unwrap_or("user".into());

            if stored_password == payload.password {
                let expiration = Utc::now()
                    .checked_add_signed(Duration::hours(24))
                    .unwrap()
                    .timestamp() as usize;

                let claims = Claims {
                    sub: payload.email.clone(),
                    username: username.clone(),
                    role: role.clone(),
                    exp: expiration,
                };

                let token = encode(
                    &Header::default(),
                    &claims,
                    &EncodingKey::from_secret(b"seu_segredo_super_forte"),
                ).unwrap();

                Json(LoginResponse {
                    message: token,
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

async fn get_connection_by_id(
    State(pool): State<PgPool>,
    Path(id): Path<i32>,
) -> Result<Json<Connection>, StatusCode> {
    let result = sqlx::query_as::<_, Connection>("SELECT * FROM connections WHERE id = $1")
        .bind(id)
        .fetch_one(&pool)
        .await;

    match result {
        Ok(conn) => Ok(Json(conn)),
        Err(_) => Err(StatusCode::NOT_FOUND),
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

async fn get_user_by_username(
    State(pool): State<PgPool>,
    Path(username): Path<String>,
) -> Result<Json<UserProfile>, StatusCode> {
    let result = sqlx::query_as::<_, UserProfile>(
        "SELECT id, username, email, avatar_url FROM users WHERE username = $1"
    )
    .bind(username)
    .fetch_one(&pool)
    .await;

    match result {
        Ok(user) => Ok(Json(user)),
        Err(_) => Err(StatusCode::NOT_FOUND),
    }
}

#[tokio::main]
async fn main() {
    from_path(StdPath::new("../.env")).expect("Falha ao carregar .env");
    let db_url = env::var("DATABASE_URL_FOR_WEB").expect("DATABASE_URL_FOR_WEB não definida");
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
        .route("/containers/:id", get(get_connection_by_id))
        .layer(middleware::from_fn(require_auth));
        

    let app = Router::new()
        .route("/register", post(register))
        .route("/login", post(login))
        .route("/perfil/:username", get(get_user_by_username))
        .merge(protected_routes)
        .with_state(pool)
        .layer(cors);

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
