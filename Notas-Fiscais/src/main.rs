/*
  Tomar cuidado com a pegada do token
sempre monitorar o uptime e tratamento
de erros.
Contato:

Instagram: https://www.instagram.com/ViitoJooj/
Github: https://github.com/819SauCe/
Local: Jaboticabal-SP
*/

use chrono::{Duration as ChronoDuration, Local, NaiveDateTime, Timelike};
use dotenvy::from_path;
use once_cell::sync::Lazy;
use tower_http::cors::{Any, CorsLayer};
use axum::{Router, routing::{post, options}, extract::Json, response::IntoResponse};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use serde_json::json;
use std::env;
use std::path::Path;
use tokio::net::TcpListener;
use tokio::sync::Mutex;
use tokio::time::{Duration as TokioDuration, sleep};
use tokio_postgres::NoTls;
use tokio_postgres::types::ToSql;
use uuid::Uuid;

static MESSAGE_HISTORY: Lazy<Mutex<Vec<serde_json::Value>>> = Lazy::new(|| Mutex::new(Vec::new()));

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
    let dados: Value = serde_json::from_str(&std::env::var("BODY_APIV2").unwrap()).unwrap();
    let res = client
        .post("http://global_trade.cr.wk.net.br/wk.api/api/v1/token")
        .json(&dados)
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
    let data_antiga = data_atual - ChronoDuration::days(1096);
    let url = format!(
        "http://global_trade.cr.wk.net.br/wk.api/api/comercial/v1/nota-fiscal?DataEmissaoInicial={}&DataEmissaoFinal={}",
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
    let url = format!(
        "http://global_trade.cr.wk.net.br/wk.api/api/empresarial/v1/produto/{}",
        id_produto
    );
    let res = client
        .get(&url)
        .bearer_auth(token)
        .send()
        .await
        .ok()?
        .json::<Value>()
        .await
        .ok()?;

    Some(res)
}

async fn inserir_dados() {
    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL não definida");
    let (client, connection) = tokio_postgres::connect(&db_url, NoTls)
        .await
        .expect("Falha na conexão");

    println!("Conectado ao banco!");
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("Erro na conexão!\nError: {}", e);
        }
    });

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
            let data_emissao_format =NaiveDateTime::parse_from_str(data_emissao, "%Y-%m-%dT%H:%M:%S").unwrap_or_else(|_| NaiveDateTime::UNIX_EPOCH);
            let data_saida = nota["dataEntradaSaida"].as_str().unwrap_or("");
            let data_saida_format = NaiveDateTime::parse_from_str(data_saida, "%Y-%m-%dT%H:%M:%S").unwrap_or_else(|_| NaiveDateTime::UNIX_EPOCH);
            let numero = nota["numero"].as_i64().unwrap_or(0);
            let cliente = nota["idClienteFornecedor"].as_str().unwrap_or("");
            let numero_str = numero.to_string();
            let row = client.query_opt("SELECT id_uuid FROM notas_fiscais WHERE chave = $1",&[&chave],).await.unwrap();
            let id_uuid = if let Some(r) = row {r.get::<_, Uuid>(0)} else {Uuid::new_v4()};
            let id = id_uuid.to_string();
            let tipo_api = nota["tipo"].as_str().unwrap_or("");
            let tipo = match tipo_api {"Entrada" => "Entrada","Saida" => "Saída",_ => "Saída",};
            let nomecliente = nota.get("localEntrega").and_then(|v| v.get("nome")).and_then(|n| n.as_str()).unwrap_or("");

            client.execute(
                "INSERT INTO notas_fiscais (id, id_uuid, chave, idfilial, tipo, dataemissao, dataentradasaida, numero, codigocliente, nomecliente)
                 VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
                 ON CONFLICT (chave) DO UPDATE 
                 SET idfilial = EXCLUDED.idfilial, 
                     tipo = EXCLUDED.tipo, 
                     dataemissao = EXCLUDED.dataemissao, 
                     dataentradasaida = EXCLUDED.dataentradasaida, 
                     numero = EXCLUDED.numero, 
                     codigocliente = EXCLUDED.codigocliente, 
                     nomecliente = EXCLUDED.nomecliente;",
                &[&id, &id_uuid, &chave, &idfilial, &tipo, &data_emissao_format, &data_saida_format, &numero_str, &cliente, &nomecliente]
            ).await.unwrap();

            if let Some(itens) = nota.get("itens").and_then(|i| i.as_array()) {
                for item in itens {
                    if let Some(id_produto) = item
                        .get("produtoServico")
                        .and_then(|p| p.get("id"))
                        .and_then(|v| v.as_str())
                    {
                        let existe = client
                            .query_opt("SELECT 1 FROM produtos_nota WHERE id = $1", &[&id_produto])
                            .await
                            .unwrap();

                        if existe.is_none() {
                            if let Some(produto) = obter_produto(token_str, id_produto).await {
                                let nome =
                                    produto.get("nome").and_then(|v| v.as_str()).unwrap_or("");
                                let codigo: &str =
                                    produto.get("codigo").and_then(|v| v.as_str()).unwrap_or("");
                                let descricao = produto
                                    .get("descricao")
                                    .and_then(|v| v.as_str())
                                    .unwrap_or("");
                                let tipo =
                                    produto.get("tipo").and_then(|v| v.as_str()).unwrap_or("");
                                let preco_venda = produto
                                    .get("precoVenda")
                                    .and_then(|v| v.as_f64())
                                    .unwrap_or(0.0);
                                let peso_bruto = produto
                                    .get("pesoBruto")
                                    .and_then(|v| v.as_f64())
                                    .unwrap_or(0.0);
                                let peso_liquido = produto
                                    .get("pesoLiquido")
                                    .and_then(|v| v.as_f64())
                                    .unwrap_or(0.0);
                                let classificacao = produto
                                    .get("classificacao")
                                    .and_then(|v| v.as_str())
                                    .unwrap_or("");
                                let referencia = produto
                                    .get("referencia")
                                    .and_then(|v| v.as_str())
                                    .unwrap_or("");
                                let gtin = produto
                                    .get("complemento")
                                    .and_then(|c| c.get("gtin"))
                                    .and_then(|v| v.as_str())
                                    .unwrap_or("");

                                client.execute(
                                    "INSERT INTO produtos_nota (id, codigo, nome, descricao, tipo, preco_venda, peso_bruto, peso_liquido, classificacao, referencia, gtin)
                                     VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
                                     ON CONFLICT (id) DO NOTHING",
                                    &[&id_produto, &codigo, &nome, &descricao, &tipo, &preco_venda, &peso_bruto, &peso_liquido, &classificacao, &referencia, &gtin]
                                ).await.unwrap();
                            } else {
                                println!(
                                    "Produto {} não encontrado na API, pulando item!",
                                    id_produto
                                );
                                continue; // SE não encontrou na API, pula o item
                            }
                        }

                        println!("insert {}", item);

                        let valor_total = item
                            .get("valorTotal")
                            .and_then(|v| v.as_f64())
                            .unwrap_or(0.0);
                        let id_produto_string: String = id_produto.to_string();
                        let id_produto_ref: &(dyn ToSql + Sync) = &id_produto_string;
                        let id_uuid_ref: &(dyn ToSql + Sync) = &id_uuid;

                        client
                            .execute(
                                "INSERT INTO itens_nota (id_nota, id_produto, valor_total)
                             VALUES ($1::uuid, $2::text, $3)
                             ON CONFLICT (id_nota, id_produto) DO UPDATE 
                            SET valor_total = EXCLUDED.valor_total;",
                                &[id_uuid_ref, id_produto_ref, &valor_total],
                            )
                            .await
                            .unwrap();
                    }
                }
            }
        }
    }

    println!("Completo");
}

async fn tratamento_resposta(mensagem: &str) -> String {
    let client = Client::new();
    let api_key = std::env::var("API_OPENAI").expect("API_OPENAI não definida");

    let mut history = MESSAGE_HISTORY.lock().await;
    history.push(json!({"role": "user", "content": mensagem}));
    while history.len() > 10 {
        history.remove(0);
    }

    let body = json!({
        "model": "gpt-4-turbo",
        "messages": vec![
            json!({"role": "system", "content": "Você é um IA focada em dar relatorios de notas fiscais, faça um relatorio igual o q vou te mandar e não opne nem responda coisas, se identificar que eu não mandei um relatorio responda com 'nota fiscal n encontrada contatar o suporte'"})
        ].into_iter().chain(history.clone()).collect::<Vec<_>>()
    });

    let response = client
        .post("https://api.openai.com/v1/chat/completions")
        .bearer_auth(&api_key)
        .json(&body)
        .send()
        .await;

    match response {
        Ok(resp) => {
            let status = resp.status();
            let text = resp
                .text()
                .await
                .unwrap_or_else(|_| "Erro ao ler body".to_string());
            println!("STATUS: {}", status);
            println!("BODY:\n{}", text);

            let json: Value = serde_json::from_str(&text)
                .unwrap_or_else(|_| json!({ "error": "resposta inválida" }));
            let content = json
                .get("choices")
                .and_then(|c| c.get(0))
                .and_then(|c| c.get("message"))
                .and_then(|m| m.get("content"))
                .and_then(|v| v.as_str())
                .unwrap_or("Erro: sem resposta da IA")
                .to_string();

            history.push(json!({"role": "assistant", "content": &content}));
            while history.len() > 10 {
                history.remove(0);
            }

            content
        }
        Err(e) => {
            println!("Erro na chamada da API: {}", e);
            "Erro na requisição para a IA".to_string()
        }
    }
}

#[axum::debug_handler]
async fn gerar_relatorio(Json(payload): Json<Mensagem>) -> impl IntoResponse {
    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL não definida");
    let (client, connection) = tokio_postgres::connect(&db_url, NoTls)
        .await
        .expect("Erro ao conectar no banco");
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("Erro conexão: {}", e);
        }
    });

    let numero_input = payload.user_message.trim();
    let row_opt = client.query_opt("SELECT id_uuid, chave, tipo, dataemissao, dataentradasaida, codigocliente, nomecliente FROM notas_fiscais WHERE numero = $1", &[&numero_input]).await.unwrap();

    let mensagem_final = if let Some(row) = row_opt {
        let id_uuid: Uuid = row.get(0);
        let chave: &str = row.get(1);
        let tipo: &str = row.get(2);
        let dataemissao: Option<NaiveDateTime> = row.get(3);
        let dataentradasaida: NaiveDateTime = row.get(4);
        let codigocliente: &str = row.get(5);
        let nomecliente: &str = row.get(6);

        let dataemissao_fmt = dataemissao
            .map(|d| d.format("%d/%m/%Y").to_string())
            .unwrap_or("N/D".to_string());
        let dataentradasaida_fmt = dataentradasaida.format("%d/%m/%Y").to_string();

        let itens = client
            .query(
                "SELECT id_produto, valor_total FROM itens_nota WHERE id_nota = $1",
                &[&id_uuid],
            )
            .await
            .unwrap();
        let mut texto_itens = String::new();
        for item in itens {
            let id_produto: &str = item.get(0);
            let valor_total: f64 = item.get(1);
            texto_itens += &format!("- Produto: {}, Total: {:.2}\n", id_produto, valor_total);
        }

        format!(
            "Nota fiscal encontrada!\n\
            Número: {}\nTipo: {}\nCliente: {} ({})\nChave: {}\n\
            Emissão: {}\nSaída: {}\nItens:\n{}",
            numero_input,
            tipo,
            nomecliente,
            codigocliente,
            chave,
            dataemissao_fmt,
            dataentradasaida_fmt,
            texto_itens
        )
    } else {
        payload.user_message.clone()
    };

    let resposta = tratamento_resposta(&mensagem_final).await;
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


    match TcpListener::bind("0.0.0.0:5400").await {
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
            inserir_dados().await;
        }

        sleep(TokioDuration::from_secs(60)).await;
    }
}

#[tokio::main]
async fn main() {
    from_path(Path::new("/opt/IAGX-Page-v0.2/.env")).expect("Falha ao carregar .env");
    //println!("DATABASE_URL: {:?}", env::var("DATABASE_URL"));
    //println!("BODY_APIV2: {:?}", env::var("BODY_APIV2"));
    //println!("API_OPENAI: {:?}", env::var("API_OPENAI"));
    inserir_dados().await;

    tokio::join!(start_http_server(), rotina_de_insercao());
}
