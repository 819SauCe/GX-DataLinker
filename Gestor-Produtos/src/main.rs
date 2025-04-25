use chrono::{Local, Timelike};
use dotenvy::dotenv;
use reqwest::Client;
use serde_json::Value;
use std::{array, env};
use tokio::time::{sleep, Duration as TokioDuration};
use tokio_postgres::NoTls;
use serde_json::json;
use serde::{Deserialize, Serialize};
use tokio::net::TcpListener;
use tokio::sync::Mutex;
use once_cell::sync::Lazy;
use axum::{Router, routing::post, extract::Json, response::IntoResponse};
use chrono::NaiveDate;

async fn obtain_token() -> Value {
    dotenv().ok();
    let client = Client::new();
    let dados: Value = serde_json::from_str(&std::env::var("JSON_DATA").unwrap()).unwrap();
    let res = client.post("https://global_trade.cr.wk.net.br/wk.api/api/v1/token").json(&dados).send().await.unwrap().json::<Value>().await.unwrap();

    res
}

async fn obter_produto(token: &str) -> Option<Value> {
    let client = reqwest::Client::new();
    let url = "https://global_trade.cr.wk.net.br/wk.api/api/empresarial/v1/produto";
    let res = client.get(url).bearer_auth(token).send().await.ok()?.json::<Value>().await.ok()?;

    Some(res)
}

async fn separando_dados() {
    dotenv().ok();
    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL não definida");
    let (client, connection) = tokio_postgres::connect(&db_url, NoTls).await.expect("Falha na conexão");
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("Erro na conexão!\nError: {}", e);
        }
    });
    println!("Conectado ao banco!");

    let token_data = obtain_token().await;
    let token = token_data["token"].as_str().unwrap();
    let produtos = obter_produto(token).await.unwrap();

    if let Some(array) = produtos.as_array() {
        for produtos in array {
            let id = produtos["id"].as_str().unwrap_or_default();
            let codigo = produtos["codigo"].as_str().unwrap_or_default();
            let nome = produtos["nome"].as_str().unwrap_or_default();
            let descricao = produtos["descricao"].as_str().unwrap_or_default();
            let inativo_bool = produtos["inativo"].as_bool().unwrap_or_default();
            let inativo = if !inativo_bool { "Ativo" } else { "Inativo" };
            let ipi = produtos["ipi"]["classificacaoFiscalNCM"].as_str().unwrap_or_default();

            let vazia = vec![];
            let lista_info = match produtos.get("listaInfoPlus").and_then(|v| v.as_array()) {
                Some(arr) => arr,
                None => &vazia,
            };

            let id_marca = lista_info
                .iter()
                .find(|item| item["posicao"] == 14)
                .and_then(|item| item["valor"].as_str())
                .unwrap_or_default();

                let nome_marca = if !id_marca.is_empty() {
                    let url = format!("https://global_trade.cr.wk.net.br/wk.api/api/empresarial/v1/informacao-complementar/{}", id_marca);
                    let resp = reqwest::Client::new()
                        .get(&url)
                        .bearer_auth(token)
                        .send()
                        .await
                        .ok();
                
                    if let Some(response) = resp {
                        if let Ok(json) = response.json::<Value>().await {
                            json["descricao"].as_str().unwrap_or_default().to_string()
                        } else {
                            String::new()
                        }
                    } else {
                        String::new()
                    }
                } else {
                    String::new()
                };                

            client.execute(
                "INSERT INTO produtos_gp (id_produto, marca, codigo, nome, descricao, inativo, ipi) 
                 VALUES ($1, $2, $3, $4, $5, $6, $7) 
                 ON CONFLICT (id_produto) DO UPDATE SET 
                    marca = EXCLUDED.marca,
                    codigo = EXCLUDED.codigo, 
                    nome = EXCLUDED.nome, 
                    descricao = EXCLUDED.descricao, 
                    inativo = EXCLUDED.inativo, 
                    ipi = EXCLUDED.ipi",
                &[&id, &nome_marca, &codigo, &nome, &descricao, &inativo, &ipi]
            ).await.unwrap();                       
        }
    }
}

#[tokio::main]
async fn main() {
    dotenv().ok();
    separando_dados().await;
}
