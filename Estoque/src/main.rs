/*
  Esse código está utilizando a api do motor antigo
do wk radar, no futuro talvez precise de grandes alt-
  rações, qualquer coisa mande uma mensagem para:

Instagram: https://www.instagram.com/ViitoJooj/
Github: https://github.com/819SauCe/
Local: Jaboticabal-SP
*/

use dotenvy::dotenv;
use serde_json::Value;
use std::env;
use tokio_postgres::NoTls;
use reqwest::header::{HeaderMap, HeaderValue};
use tokio_postgres::Error;

async fn data() -> Result<serde_json::Value, reqwest::Error> {
    dotenv().ok();
    const URL: &str = "http://global_trade.cr.wk.net.br/RadarWebWebServices/Areas/Estoque/Estoque.svc/json/BuscarSaldoProduto";
    let login_info: Value = serde_json::from_str(&env::var("JSON_DATA").unwrap()).unwrap();
    let filtro_estoque = serde_json::json!({"EstoqueProprio": true,"CodigoLocais": "46"});
    let bodies = serde_json::json!({"login": login_info,"filtro": filtro_estoque});
    let mut headers = HeaderMap::new();

    headers.insert("Content-Type", HeaderValue::from_static("application/json"));
    headers.insert("Accept", HeaderValue::from_static("application/json"));
    headers.insert("Accept-Encoding", HeaderValue::from_static("gzip, deflate, br"));
    headers.insert("Connection", HeaderValue::from_static("keep-alive"));

    let client = reqwest::Client::builder().default_headers(headers).build()?;
    let response = client.post(URL).json(&bodies).send().await?;
    let body = response.json::<serde_json::Value>().await?;

    Ok(body)
}

async fn data_kit() -> Result<serde_json::Value, reqwest::Error> {
    dotenv().ok();
    const URL: &str = "http://global_trade.cr.wk.net.br/RadarWebWebServices/Areas/Estoque/Estoque.svc/json/BuscarSaldoProdutoKits";
    let login_info: Value = serde_json::from_str(&env::var("JSON_DATA").unwrap()).unwrap();
    let filtro_estoque = serde_json::json!({"EstoqueProprio": true,"CodigoLocais": "46"});
    let bodies = serde_json::json!({"login": login_info,"filtro": filtro_estoque});
    let mut headers = HeaderMap::new();

    headers.insert("Content-Type", HeaderValue::from_static("application/json"));
    headers.insert("Accept", HeaderValue::from_static("application/json"));
    headers.insert("Accept-Encoding", HeaderValue::from_static("gzip, deflate, br"));
    headers.insert("Connection", HeaderValue::from_static("keep-alive"));

    let client = reqwest::Client::builder().default_headers(headers).build()?;
    let response = client.post(URL).json(&bodies).send().await?;
    let body = response.json::<serde_json::Value>().await?;

    Ok(body)
}

async fn insert_data() -> Result<(), Error> {
    dotenv().ok();
    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL não definida");
    let (client, connection) = tokio_postgres::connect(&db_url, NoTls).await.expect("Falha na conexão");
    println!("Conectado ao banco!");

    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("Erro na conexão!\nError: {}", e);
        }
    });

    let dados = data().await.expect("Falha ao obter dados da API");
    println!("{}", dados);

    if let Some(result) = dados.get("BuscarSaldoProdutoResult") {
        if let Some(produtos) = result.get("QuantidadeDisponivelProdutos").and_then(|v| v.as_array()) {
            for produto in produtos {
                let codigo_produto = produto.get("CodigoProduto").and_then(|v| v.as_str()).unwrap_or("");
                let quantidade_disponivel = produto.get("QuantidadeDisponivel").and_then(|v| v.as_str()).unwrap_or("");
                let nome: String = client.query_one("SELECT nome FROM produtos WHERE codigo = $1", &[&codigo_produto]).await.map(|row| row.get(0)).unwrap_or_else(|_| String::from("Produto não encontrado"));

                println!("Código do Produto: {}, Quantidade Disponível: {}, Nome: {}", codigo_produto, quantidade_disponivel, nome);
            }
        }
    }

    Ok(())
}

#[tokio::main]
async fn main() {
    match insert_data().await {
        Ok(_) => println!("Dados inseridos com sucesso!"),
        Err(e) => eprintln!("Erro ao inserir dados: {}", e),
    }
}
