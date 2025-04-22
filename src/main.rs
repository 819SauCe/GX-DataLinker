use chrono::{DateTime, Local, NaiveDateTime, Timelike, Duration as ChronoDuration};
use dotenvy::dotenv;
use reqwest::Client;
use serde_json::Value;
use std::env;
use tokio::time::{sleep, Duration as TokioDuration};
use tokio_postgres::NoTls;
use uuid::Uuid;
use serde_json::json;
use serde::{Deserialize, Serialize};
use axum::{Router, routing::post, Json};
use tokio::net::TcpListener;
use tokio_postgres::types::ToSql;

#[derive(Deserialize)]
struct Mensagem {
    user_message: String,
}

#[derive(Serialize)]
struct RespostaIA {
    resposta: String,
}

async fn obtain_token() -> Value {
    let client = Client::new();
    let dados: Value = serde_json::from_str(&std::env::var("JSON_DATA").unwrap()).unwrap();
    let res = client.post("https://global_trade.cr.wk.net.br/wk.api/api/v1/token").json(&dados).send().await.unwrap().json::<Value>().await.unwrap();

    res
}

async fn obtain_nfe(token: &str) -> Value {
    let client = Client::new();
    let data_atual = Local::now().naive_local().date();
    let data_antiga = data_atual - ChronoDuration::days(1096);
    let url = format!("https://global_trade.cr.wk.net.br/wk.api/api/comercial/v1/nota-fiscal?DataEmissaoInicial={}&DataEmissaoFinal={}",data_antiga, data_atual);
    let res = client.get(&url).bearer_auth(token).send().await.unwrap().json::<Value>().await.unwrap();

    return res;
}

async fn obter_produto(token: &str, id_produto: &str) -> Option<Value> {
    let client = Client::new();
    let url = format!("https://global_trade.cr.wk.net.br/wk.api/api/empresarial/v1/produto/{}", id_produto);
    let res = client.get(&url).bearer_auth(token).send().await.ok()?.json::<Value>().await.ok()?;
    
    Some(res)
}


#[tokio::main]
async fn main() {
    // conexão com o banco
    dotenv().ok();
    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL não definida");
    let (client, connection) = tokio_postgres::connect(&db_url, NoTls).await.expect("Falha na conexão");

    println!("Conectado ao banco!");
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("Erro na conexão!\nError: {}", e);
        }
    });

    //obtendo token
    println!("Obtendo token");
    let token = obtain_token().await;
    let token_str = token["token"].as_str().unwrap();
    let data = obtain_nfe(token_str).await;
    println!("Token pego!");

    if let Some(notas) = data.as_array() {
        for nota in notas {
            let chave = nota["chave"].as_str().unwrap_or("");
            let idfilial = nota.get("rateios").and_then(|r| r.as_array()).and_then(|arr| arr.get(0)).and_then(|r| r.get("idFilial")).and_then(|v| v.as_str()).unwrap_or("");
            let data_emissao = nota["dataEmissao"].as_str().unwrap_or("");
            let data_saida = nota["dataEntradaSaida"].as_str().unwrap_or("");
            let numero = nota["numero"].as_i64().unwrap_or(0);
            let cliente = nota["idClienteFornecedor"].as_str().unwrap_or("");
            let data_emissao_dt = DateTime::parse_from_rfc3339(data_emissao).map(|dt| dt.naive_utc()).ok();
            let data_saida_dt = DateTime::parse_from_rfc3339(data_saida).map(|dt| dt.naive_utc()).unwrap_or_else(|_| NaiveDateTime::UNIX_EPOCH);
            let numero_str = numero.to_string();
            let row = client.query_opt("SELECT id_uuid FROM notas_fiscais WHERE chave = $1",&[&chave]).await.unwrap();
            let id_uuid = if let Some(r) = row {r.get::<_, uuid::Uuid>(0)} else {Uuid::new_v4()};
            let id = id_uuid.to_string();
            let tipo_api = nota["tipo"].as_str().unwrap_or("");
            let tipo = match tipo_api {"Entrada" => "Entrada","Saida" => "Saída",_ => "Saída",};
            let nomecliente = nota.get("localEntrega").and_then(|v| v.get("nome")).and_then(|n| n.as_str()).unwrap_or("");
            let none_dt: Option<NaiveDateTime> = None;
            let data_emissao_sql: &(dyn ToSql + Sync) = match &data_emissao_dt {Some(dt) => dt,None => &none_dt,};

            client.execute(
                "INSERT INTO notas_fiscais (id, id_uuid, chave, idfilial, tipo, dataemissao, dataentradasaida, numero, codigocliente, nomecliente)
                 VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
                 ON CONFLICT (chave) DO NOTHING;",
                &[&id, &id_uuid, &chave, &idfilial, &tipo, data_emissao_sql, &data_saida_dt, &numero_str, &cliente, &nomecliente]
            ).await.unwrap();

            if let Some(itens) = nota.get("itens").and_then(|i| i.as_array()) {
                for item in itens {
                    if let Some(id_produto) = item.get("produtoServico").and_then(|p| p.get("id")).and_then(|v| v.as_str()) {
                        let existe = client.query_opt("SELECT 1 FROM produtos WHERE id = $1", &[&id_produto]).await.unwrap();
                    
                        if existe.is_none() {
                            if let Some(produto) = obter_produto(token_str, id_produto).await {
                                let nome = produto.get("nome").and_then(|v| v.as_str()).unwrap_or("");
                                let descricao = produto.get("descricao").and_then(|v| v.as_str()).unwrap_or("");
                                let tipo = produto.get("tipo").and_then(|v| v.as_str()).unwrap_or("");
                                let preco_venda = produto.get("precoVenda").and_then(|v| v.as_f64()).unwrap_or(0.0);
                                let peso_bruto = produto.get("pesoBruto").and_then(|v| v.as_f64()).unwrap_or(0.0);
                                let peso_liquido = produto.get("pesoLiquido").and_then(|v| v.as_f64()).unwrap_or(0.0);
                                let classificacao = produto.get("classificacao").and_then(|v| v.as_str()).unwrap_or("");
                                let referencia = produto.get("referencia").and_then(|v| v.as_str()).unwrap_or("");
                                let gtin = produto.get("complemento").and_then(|c| c.get("gtin")).and_then(|v| v.as_str()).unwrap_or("");
                    
                                client.execute(
                                    "INSERT INTO produtos (id, codigo, nome, descricao, tipo, preco_venda, peso_bruto, peso_liquido, classificacao, referencia, gtin)
                                     VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
                                     ON CONFLICT (id) DO NOTHING",
                                    &[&id_produto, &id_produto, &nome, &descricao, &tipo, &preco_venda, &peso_bruto, &peso_liquido, &classificacao, &referencia, &gtin]
                                ).await.unwrap();
                            }
                        }
                    
                        // esta parte roda sempre, independente de já existir o produto
                        let descricao_item = item.get("complemento").and_then(|v| v.as_str()).unwrap_or("");
                        let valor_total = item.get("valorTotal").and_then(|v| v.as_f64()).unwrap_or(0.0);
                        let id_produto_string: String = id_produto.to_string();
                        let id_produto_ref: &(dyn ToSql + Sync) = &id_produto_string;
                        let id_uuid_ref: &(dyn ToSql + Sync) = &id_uuid;

                        client.execute(
                            "INSERT INTO itens_nota (id_nota, id_produto, valor_total)
                            VALUES ($1::uuid, $2::text, $3)
                            ON CONFLICT (id_nota, id_produto) DO NOTHING;",
                            &[id_uuid_ref, id_produto_ref, &valor_total]
                        ).await.unwrap();


    
                    }
                    
                    }
                }
            }            
        }

        println!("Completo")
}

async fn tratamento_resposta(mensagem: &str) -> String {
    dotenv().ok();
    let client = Client::new();
    let api_key = env::var("API_OPENAI").unwrap();
    let body = json!({
        "model": "gpt-4-turbo",
        "messages": [
            {"role": "system", "content": "Você é um robo focado em fazer relatorio de notas fiscais"},
            {"role": "user", "content": mensagem}
        ]
    });

    let res = client.post("https://api.openai.com/v1/chat/completions").bearer_auth(api_key).json(&body).send().await.unwrap().json::<serde_json::Value>().await.unwrap();

    res["choices"][0]["message"]["content"].as_str().unwrap_or("Erro na resposta").to_string()
}

async fn gerar_relatorio(Json(payload): Json<Mensagem>) -> Json<RespostaIA> {
    let resposta = tratamento_resposta(&payload.user_message).await;
    Json(RespostaIA { resposta })
}

async fn start_http_server() {
    println!("Servidor rodando!");
    let app = Router::new().route("/api/gerar_relatorio", post(gerar_relatorio));   
    let listener = TcpListener::bind("0.0.0.0:5400").await.unwrap();
    axum::serve(listener, app.into_make_service()).await.unwrap();
}

async fn rotina_de_insercao() {
    loop {
        let agora = Local::now();
        let hora = agora.format("%H:%M").to_string();
        let data = agora.format("%d/%m/%Y").to_string();
        let minutos = agora.minute();
        let bar_construct = "-";
        let bar = bar_construct.repeat(16);
        println!("Hora: {}", hora);
        println!("Data: {}", data);
        println!("{}", bar);

        if minutos % 15 == 0 {
            println!("inserindo dados!");
            inserir_dados().await;
        }

        sleep(TokioDuration::from_secs(60)).await;
    }
}

#[tokio::main]
async fn main() {
    tokio::join!(
        start_http_server(),
        rotina_de_insercao()
    );
}
