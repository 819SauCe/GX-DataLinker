/*
  Esse código está utilizando a api do motor antigo
do wk radar, no futuro talvez precise de grandes alt-
  rações, qualquer coisa mande uma mensagem para:

Instagram: https://www.instagram.com/ViitoJooj/
Github: https://github.com/819SauCe/
Local: Jaboticabal-SP
*/

//use dotenvy::dotenv;
use serde_json::Value;
use std::env;
use tokio_postgres::NoTls;
use reqwest::header::{HeaderMap, HeaderValue};
use tokio_postgres::Error;
use reqwest::Client;
use serde_json::json;
use tokio::sync::Mutex;
use once_cell::sync::Lazy;
use axum::{Router, routing::{post, options}, extract::Json, response::IntoResponse};
use serde::{Deserialize, Serialize};
use tokio::net::TcpListener;
use chrono::{Local, Timelike};
use tokio::time::{sleep, Duration as TokioDuration};
use tower_http::cors::{Any, CorsLayer};

static MESSAGE_HISTORY: Lazy<Mutex<Vec<serde_json::Value>>> = Lazy::new(|| Mutex::new(Vec::new()));

#[derive(Deserialize)]
struct Mensagem {
    user_message: String,
}

#[derive(Serialize)]
struct RespostaIA {
    resposta: String,
}

async fn data(url: &str) -> Result<serde_json::Value, reqwest::Error> {
    let login_info: Value = serde_json::from_str(&env::var("BODY_APIV1").expect("Falha ao obter a variável de ambiente 'BODY_APIV1'")).expect("Falha ao deserializar o JSON da variável 'BODY_APIV1'");
    let filtro_estoque = serde_json::json!({"EstoqueProprio": true,"CodigoLocais": "46"});
    let bodies = serde_json::json!({"login": login_info,"filtro": filtro_estoque});
    let mut headers = HeaderMap::new();

    headers.insert("Content-Type", HeaderValue::from_static("application/json"));
    headers.insert("Accept", HeaderValue::from_static("application/json"));
    headers.insert("Accept-Encoding", HeaderValue::from_static("gzip, deflate, br"));
    headers.insert("Connection", HeaderValue::from_static("keep-alive"));

    let client = reqwest::Client::builder().default_headers(headers).build()?;
    let response = client.post(url).json(&bodies).send().await?;
    let body = response.json::<serde_json::Value>().await?;

    Ok(body)
}

async fn insert_data_from_url(url: &str, client: &tokio_postgres::Client,) -> Result<(), Error> {
    let dados = data(url).await.expect("Falha ao obter dados da API");
    for key in &["BuscarSaldoProdutoResult", "BuscarSaldoProdutoKitResult"] {
        if let Some(result) = dados.get(*key) {
            if let Some(items) = result
                .get("QuantidadeDisponivelProdutos")
                .and_then(|v| v.as_array()){
                for item in items {
                    let codigo: &str = item.get("CodigoProduto").and_then(|v| v.as_str()).unwrap_or("");
                    let qtd: String = item.get("QuantidadeDisponivel").and_then(|v| v.as_str().map(|s| s.to_string())).unwrap_or_else(|| "0".to_string());
                    let nome: String = client.query_one("SELECT nome FROM produtos WHERE codigo = $1",&[&codigo],).await.map(|row| row.get(0)).unwrap_or_else(|_| "Nome não encontrado!".to_string());

                    client.execute("INSERT INTO Estoque_and_EstoqueKits 
                                   (codigo_produto, quantidade_disponivel, nome)
                                    VALUES ($1, $2, $3)
                                    ON CONFLICT (codigo_produto) DO UPDATE
                                    SET quantidade_disponivel = EXCLUDED.quantidade_disponivel,
                                    nome                  = EXCLUDED.nome",&[&codigo, &qtd, &nome],).await?;
                }
            }
        }
    }

    Ok(())
}

async fn insert_data() -> Result<(), Error> {
    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL não definida");
    let (client, connection) = tokio_postgres::connect(&db_url, NoTls).await.expect("Falha na conexão");
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("Erro na conexão!\nError: {}", e);
        }
    });
    println!("Banco conectado!");

    let url1 = "http://global_trade.cr.wk.net.br/RadarWebWebServices/Areas/Estoque/Estoque.svc/json/BuscarSaldoProduto";
    let url2 = "http://global_trade.cr.wk.net.br/RadarWebWebServices/Areas/Estoque/Estoque.svc/json/BuscarSaldoProdutoKit";
    tokio::try_join!(insert_data_from_url(url1, &client),insert_data_from_url(url2, &client),)?;

    Ok(())
}

async fn tratamento_resposta(mensagem: &str) -> String {
    let client = Client::new();
    let api_key = env::var("API_OPENAI").expect("Variável API_OPENAI não definida");
    let mut history = MESSAGE_HISTORY.lock().await;

    history.push(json!({"role": "user", "content": mensagem}));
    while history.len() > 10 { history.remove(0); }

    let body = json!({
        "model": "gpt-4-turbo",
        "messages": vec![
            json!({"role": "system", "content":
             "Você é uma IA focada em dar relatorios de estoque com base no que vou te passar
              Não escreva nada além das informações q eu passei, apenas se eu tiver duvida sobre
               algo relacionado aquele item. se receber um código mas sem demais informações diga que o item não existe ou não tem em estoque."})
        ].into_iter().chain(history.clone()).collect::<Vec<_>>()
    });

    let res = client.post("https://api.openai.com/v1/chat/completions").bearer_auth(api_key).json(&body).send().await.unwrap().json::<Value>().await.unwrap();
    let content = res["choices"][0]["message"]["content"].as_str().unwrap_or("Erro na resposta").to_string();

    history.push(json!({"role": "assistant", "content": content}));
    while history.len() > 10 { history.remove(0); }

    content
}

#[axum::debug_handler]
async fn gerar_relatorio(Json(payload): Json<Mensagem>) -> impl IntoResponse {
    println!("Recebendo mensagem: {:?}", payload.user_message);

    let db_url = match env::var("DATABASE_URL") {
        Ok(url) => url,
        Err(_) => {
            eprintln!("❌ DATABASE_URL não definida.");
            return Json(RespostaIA {
                resposta: "Erro interno: DATABASE_URL ausente.".to_string(),
            });
        }
    };

    let (client, connection) = match tokio_postgres::connect(&db_url, NoTls).await {
        Ok((client, connection)) => (client, connection),
        Err(err) => {
            eprintln!("❌ Falha ao conectar no banco: {}", err);
            return Json(RespostaIA {
                resposta: "Erro interno ao conectar no banco.".to_string(),
            });
        }
    };
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("❌ Erro na conexão do banco: {}", e);
        }
    });

    let user_input = payload.user_message.trim();
    let row_opt = match client
        .query_opt(
            "SELECT codigo_produto, quantidade_disponivel, nome \
             FROM estoque_and_estoquekits \
             WHERE codigo_produto ILIKE $1 \
             LIMIT 1;",
            &[&user_input],
        )
        .await
    {
        Ok(row) => row,
        Err(err) => {
            eprintln!("❌ Erro ao buscar no banco: {}", err);
            return Json(RespostaIA {
                resposta: "Erro ao consultar estoque.".to_string(),
            });
        }
    };

    let mensagem_para_ia = if let Some(row) = row_opt {
        let codigo_produto: &str = row.get(0);
        let quantidade_disponivel: &str = row.get(1);
        let nome: &str = row.get(2);
        format!(
            "Item em estoque\n\
             Código do produto: {}\n\
             Nome: {}\n\
             Quantidade disponível: {}",
            codigo_produto, nome, quantidade_disponivel
        )
    } else {
        payload.user_message.clone()
    };

    let resposta = tratamento_resposta(&mensagem_para_ia).await;
    Json(RespostaIA { resposta })
}

async fn start_http_server() {
    println!("Servidor rodando!");

    let cors = CorsLayer::new()
    .allow_origin(Any)
    .allow_methods(Any)
    .allow_headers(Any);

    let app = Router::new()
        .route("/gerar_relatorio", post(gerar_relatorio))
        .route("/gerar_relatorio", options(|| async { "" }))
        .layer(cors);


    match TcpListener::bind("0.0.0.0:5200").await {
        Ok(listener) => {
            if let Err(e) = axum::serve(listener, app.into_make_service()).await {
                eprintln!("Erro ao iniciar o servidor: {}", e);
            }
        },
        Err(e) => {
            eprintln!("Erro ao fazer bind na porta: {}", e);
        }
    }
}

async fn rotina_de_insercao() {
    let inicio = Local::now();
    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL não definida");
    let (client, connection) = tokio_postgres::connect(&db_url, NoTls).await.expect("Erro na conexão no banco");
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("Erro conexão: {}", e);
        }
    });
    
    let url1 = "http://global_trade.cr.wk.net.br/RadarWebWebServices/Areas/Estoque/Estoque.svc/json/BuscarSaldoProduto";
    let url2 = "http://global_trade.cr.wk.net.br/RadarWebWebServices/Areas/Estoque/Estoque.svc/json/BuscarSaldoProdutoKit";
    insert_data().await.expect("Erro ao inserir dados!");

    loop {
        let agora = Local::now();
        let uptime = agora.signed_duration_since(inicio);
        let horas = uptime.num_hours();
        let minutos_uptime = uptime.num_minutes() % 60;
        let hora = agora.format("%H:%M").to_string();
        let data = agora.format("%d/%m/%Y").to_string();
        let minutos = agora.minute();
        let uptime_str = format!("Uptime: {}h {}min", horas, minutos_uptime);
        let barra = "-".repeat(uptime_str.len());

        println!("Hora: {}", hora);
        println!("Data: {}", data);
        println!("{}", uptime_str);
        println!("{}", barra);

        if minutos % 15 == 0 {
            println!("inserindo dados!");
            insert_data_from_url(url1, &client).await.expect("Erro ao inserir dados!");
            insert_data_from_url(url2, &client).await.expect("Erro ao inserir dados!");
        }
        sleep(TokioDuration::from_secs(60)).await;
    }
}

#[tokio::main]
async fn main() {
    dotenvy::from_path("/opt/IAGX-Page-v0.2/.env").ok();
    tokio::join!(
        start_http_server(),
        rotina_de_insercao()
    );
}