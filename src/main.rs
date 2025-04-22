use chrono::{DateTime, Duration, Local, NaiveDateTime};
use dotenvy::dotenv;
use reqwest::Client;
use serde_json::Value;
use serde_json::json;
use std::env;
use tokio_postgres::NoTls;
use uuid::Uuid;

async fn obtain_token() -> Value {
    let client = Client::new();
    let res = client
        .post("https://global_trade.cr.wk.net.br/wk.api/api/v1/token")
        .json(&json!({
            "empresa": "Global_trade",
            "nomeusuario": "IAGX",
            "senha": "GmygZQ"
        }))
        .send()
        .await
        .unwrap()
        .json::<Value>()
        .await
        .unwrap();

    res
}

async fn obtain_nfe(token: &str) -> Value {
    let client = Client::new();
    let data_atual = Local::now().naive_local().date();
    let data_antiga = data_atual - Duration::days(1096);
    let url = format!(
        "https://global_trade.cr.wk.net.br/wk.api/api/comercial/v1/nota-fiscal?DataEmissaoInicial={}&DataEmissaoFinal={}",
        data_antiga, data_atual
    );
    let res = client
        .get(&url)
        .bearer_auth(token)
        .send()
        .await
        .unwrap()
        .json::<Value>()
        .await
        .unwrap();

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

    dotenv().ok();
    let db_url_1 = env::var("CONECTION_GLOBAL_X").unwrap_or_default();
    let db_url_2 = env::var("CONECTION_TEST_CASA").unwrap_or_default();
    let (client, connection) = if !db_url_1.is_empty() {
        match tokio_postgres::connect(&db_url_1, NoTls).await {
            Ok(conn) => conn,
            Err(e1) => {
                eprintln!("Falha na conexão com GLOBAL_X: {}", e1);
                match tokio_postgres::connect(&db_url_2, NoTls).await {
                    Ok(conn) => conn,
                    Err(e2) => panic!("Falha também na conexão TEST_CASA: {}", e2),
                }
            }
        }
    } else {
        match tokio_postgres::connect(&db_url_2, NoTls).await {
            Ok(conn) => conn,
            Err(e) => panic!("Falha na conexão TEST_CASA (sem GLOBAL_X): {}", e),
        }
    };
    println!("Conectou!");
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

    //apagando table
    client.execute("DELETE FROM itens_nota;", &[]).await.unwrap();
    client.execute("DELETE FROM notas_fiscais;", &[]).await.unwrap();
    if let Some(notas) = data.as_array() {
        for nota in notas {
            let chave = nota["chave"].as_str().unwrap_or("");
            let idfilial = nota.get("rateios").and_then(|r| r.as_array()).and_then(|arr| arr.get(0)).and_then(|r| r.get("idFilial")).and_then(|v| v.as_str()).unwrap_or("");
            let data_emissao = nota["dataEmissao"].as_str().unwrap_or("");
            let data_saida = nota["dataEntradaSaida"].as_str().unwrap_or("");
            let numero = nota["numero"].as_i64().unwrap_or(0);
            let cliente = nota["idClienteFornecedor"].as_str().unwrap_or("");
            let data_emissao_dt = DateTime::parse_from_rfc3339(data_emissao).map(|dt| dt.naive_utc()).unwrap_or_else(|_| NaiveDateTime::UNIX_EPOCH);
            let data_saida_dt = DateTime::parse_from_rfc3339(data_saida).map(|dt| dt.naive_utc()).unwrap_or_else(|_| NaiveDateTime::UNIX_EPOCH);
            let numero_str = numero.to_string();
            let id_uuid = Uuid::new_v4();
            let id = id_uuid.to_string();
            let tipo_api = nota["tipo"].as_str().unwrap_or("");
            let tipo = match tipo_api {"Entrada" => "Entrada","Saida" => "Saída",_ => "Saída",};
            let nomecliente = nota.get("localEntrega").and_then(|v| v.get("nome")).and_then(|n| n.as_str()).unwrap_or("");

            client.execute(
                "INSERT INTO notas_fiscais (id, id_uuid, chave, idfilial, tipo, dataemissao, dataentradasaida, numero, codigocliente, nomecliente)
                 VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)",
                &[&id, &id_uuid, &chave, &idfilial, &tipo, &data_emissao_dt, &data_saida_dt, &numero_str, &cliente, &nomecliente]
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
                    
                        let descricao_item = item.get("complemento").and_then(|v| v.as_str()).unwrap_or("");
                        let valor_total = item.get("valorTotal").and_then(|v| v.as_f64()).unwrap_or(0.0);
                        client.execute(
                            "INSERT INTO itens_nota (id_nota, id_produto, descricao, valor_total) VALUES ($1, $2, $3, $4)",
                            &[&id_uuid, &id_produto, &descricao_item, &valor_total]
                        ).await.unwrap();
                    }
                    
                    }
                }
            }            
        }

        println!("terminou!!!!")
    }