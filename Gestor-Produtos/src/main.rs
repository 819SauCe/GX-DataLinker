use chrono::{Local, Timelike};
use reqwest::Client;
use serde_json::Value;
use std::env;
use tokio::time::{sleep, Duration as TokioDuration};
use tokio_postgres::NoTls;
use serde_json::json;
use serde::{Deserialize, Serialize};
use tokio::net::TcpListener;
use tokio::sync::Mutex;
use once_cell::sync::Lazy;
use axum::{Router, routing::post, extract::Json, response::IntoResponse};
use dotenvy::from_path;
use std::path::Path;

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
    let res = client.post("https://global_trade.cr.wk.net.br/wk.api/api/v1/token").json(&dados).send().await.unwrap().json::<Value>().await.unwrap();

    res
}

async fn obter_produto(token: &str) -> Option<Value> {
    let client = reqwest::Client::new();
    let url = "https://global_trade.cr.wk.net.br/wk.api/api/empresarial/v1/produto";
    let res = client.get(url).bearer_auth(token).send().await.ok()?.json::<Value>().await.ok()?;

    Some(res)
}

async fn insert_data() {
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
                let resp = reqwest::Client::new().get(&url).bearer_auth(token).send().await.ok();

                if let Some(response) = resp {
                    if let Ok(json) = response.json::<Value>().await {
                        json["name"].as_str().unwrap_or_default().to_string()
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
                "INSERT INTO produtos_gp (id_produto, marca, codigo, nome, descricao, inativo, ipi, nome_valor_item) 
                 VALUES ($1, $2, $3, $4, $5, $6, $7, $8) 
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

async fn registrar_estoque_diario() {
    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL não definida");
    let (client, connection) = tokio_postgres::connect(&db_url, NoTls).await.expect("Erro ao conectar no banco");
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("Erro conexão: {}", e);
        }
    });

    let rows = client.query(
        "SELECT codigo_produto, quantidade_disponivel FROM estoque_and_estoquekits",
        &[]
    ).await.unwrap();

    for row in rows {
        let codigo: String = row.get(0);
        let qtd_str: String = row.get(1);
        let qtd = qtd_str.replace(".", "").replace(",", ".").parse::<f64>().unwrap_or(0.0);

        client.execute(
            "INSERT INTO estoque_diario (codigo_produto, quantidade) VALUES ($1, $2)",
            &[&codigo, &qtd]
        ).await.unwrap();
    }
}


async fn tratamento_resposta(mensagem: &str) -> String {
    let client = Client::new();
    let api_key = std::env::var("API_OPENAI").expect("API_OPENAI não definida");

    let mut history = MESSAGE_HISTORY.lock().await;
    history.push(json!({"role": "user", "content": mensagem}));
    while history.len() > 10 { history.remove(0); }

    let body = json!({
        "model": "gpt-4o-2024-11-20",
        "messages": vec![
            json!({"role": "system", "content": "Você é um IA focada em dar relatorios de items, não opne nem diga algo desnecessario se eu n pedir"})
        ].into_iter().chain(history.clone()).collect::<Vec<_>>()
    });

    let response = client.post("https://api.openai.com/v1/chat/completions")
        .bearer_auth(&api_key)
        .json(&body)
        .send()
        .await;

    match response {
        Ok(resp) => {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_else(|_| "Erro ao ler body".to_string());
            println!("STATUS: {}", status);
            println!("BODY:\n{}", text);

            let json: Value = serde_json::from_str(&text).unwrap_or_else(|_| json!({ "error": "resposta inválida" }));
            let content = json.get("choices")
                .and_then(|c| c.get(0))
                .and_then(|c| c.get("message"))
                .and_then(|m| m.get("content"))
                .and_then(|v| v.as_str())
                .unwrap_or("Erro: sem resposta da IA")
                .to_string();

            history.push(json!({"role": "assistant", "content": &content}));
            while history.len() > 10 { history.remove(0); }

            content
        },
        Err(e) => {
            println!("Erro na chamada da API: {}", e);
            "Erro na requisição para a IA".to_string()
        }
    }
}

#[axum::debug_handler]
async fn gerar_relatorio(Json(payload): Json<Mensagem>) -> impl IntoResponse {
    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL não definida");
    let (client, connection) = tokio_postgres::connect(&db_url, NoTls).await.expect("Erro ao conectar no banco");
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("Erro conexão: {}", e);
        }
    });

    let codigo_input = payload.user_message.trim();

    if codigo_input.len() < 4 || !codigo_input.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_') {
        let resposta = tratamento_resposta(codigo_input).await;
        return Json(RespostaIA { resposta });
    }
    
    // Lucro mensal
    let lucro_mensal = client.query(
        "SELECT TO_CHAR(n.dataemissao::timestamp, 'YYYY-MM') AS mes_ano,
                SUM(CASE WHEN n.tipo = 'Saída' THEN i.valor_total ELSE 0 END),
                SUM(CASE WHEN n.tipo = 'Entrada' THEN i.valor_total ELSE 0 END),
                SUM(CASE WHEN n.tipo = 'Saída' THEN i.valor_total ELSE 0 END) - 
                SUM(CASE WHEN n.tipo = 'Entrada' THEN i.valor_total ELSE 0 END)
         FROM itens_nota i
         JOIN notas_fiscais n ON i.id_nota = n.id_uuid
         JOIN produtos_nota p ON i.id_produto = p.id
         WHERE p.codigo = $1 AND n.dataemissao >= CURRENT_DATE - INTERVAL '12 months'
         GROUP BY TO_CHAR(n.dataemissao, 'YYYY-MM')
         ORDER BY mes_ano;",
        &[&codigo_input]
    ).await.unwrap();

    // Lucro total
    let lucro_anual_total = client.query(
        "SELECT p.codigo,
                SUM(CASE WHEN n.tipo = 'Saída' THEN i.valor_total ELSE 0 END) - 
                SUM(CASE WHEN n.tipo = 'Entrada' THEN i.valor_total ELSE 0 END)
         FROM itens_nota i
         JOIN notas_fiscais n ON i.id_nota = n.id_uuid
         JOIN produtos_nota p ON i.id_produto = p.id
         WHERE n.dataemissao >= CURRENT_DATE - INTERVAL '12 months'
         GROUP BY p.codigo;",
        &[]
    ).await.unwrap();

    let mut produtos: Vec<(String, f64)> = lucro_anual_total
        .iter()
        .map(|r| (r.get(0), r.get(1)))
        .collect();

    let total_lucro: f64 = produtos.iter().map(|(_, v)| *v).sum();
    produtos.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

    let lucro_item: f64 = produtos.iter()
        .find(|(codigo, _)| codigo == codigo_input)
        .map(|(_, v)| *v)
        .unwrap_or(0.0);

    let percentual_item = if total_lucro > 0.0 {
        (lucro_item / total_lucro) * 100.0
    } else {
        0.0
    };

    let mut acumulado = 0.0;
    let mut classificacao = "C";
    for (codigo, valor) in &produtos {
        let p = if total_lucro > 0.0 { (*valor / total_lucro) * 100.0 } else { 0.0 };
        acumulado += p;
        if codigo == codigo_input {
            classificacao = if acumulado <= 20.0 {
                "A"
            } else if acumulado <= 50.0 {
                "B"
            } else {
                "C"
            };
        }
    }

    let mut mensagem_final = format!("**Relatório do item '{}':**\n", codigo_input);
    for row in &lucro_mensal {
        let mes_ano = row.get::<_, String>(0);
        let vendas: f64 = row.get(1);
        let compras: f64 = row.get(2);
        let lucro: f64 = row.get(3);
        mensagem_final += &format!(
            "- {} | Vendas: {:.2} | Compras: {:.2} | Lucro: {:.2}\n",
            mes_ano, vendas, compras, lucro
        );
    }

    mensagem_final += &format!(
        "\nLucro total (12 meses): {:.2}\nLucro geral: {:.2}\nParticipação: {:.2}%\nClassificação ABC: {}\n",
        lucro_item, total_lucro, percentual_item, classificacao
    );

    if lucro_item == 0.0 && lucro_mensal.is_empty() {
        mensagem_final += "\n**Observação:** Item não teve movimentação nos últimos 12 meses.\n";
    }

    // Estoque e previsão
    if let Some(row) = client.query_opt(
        "SELECT quantidade_disponivel FROM estoque_and_estoquekits WHERE codigo_produto = $1",
        &[&codigo_input]
    ).await.unwrap()
    {
        let qtd_str: String = row.get(0);
        let qtd_estoque = qtd_str.replace(".", "").replace(",", ".").parse::<f64>().unwrap_or(0.0);

        let consumo = client.query_opt(
            "SELECT SUM(i.valor_total) FROM itens_nota i
             JOIN notas_fiscais n ON i.id_nota = n.id_uuid
             JOIN produtos_nota p ON i.id_produto = p.id
             WHERE p.codigo = $1 AND n.tipo = 'Saída' AND n.dataemissao >= CURRENT_DATE - INTERVAL '30 days'",
            &[&codigo_input]
        ).await.unwrap();

        let total_consumo: f64 = consumo
            .and_then(|r| r.get::<_, Option<f64>>(0))
            .unwrap_or(0.0);

        let media_diaria = total_consumo / 30.0;
        let dias = if media_diaria > 0.0 {
            qtd_estoque / media_diaria
        } else {
            f64::INFINITY
        };

        mensagem_final += &format!(
            "\n**Previsão de Estoque:**\n- Estoque atual: {:.2}\n- Consumo médio diário (R$): {:.2}\n",
            qtd_estoque, media_diaria
        );

        if dias.is_finite() {
            mensagem_final += &format!("- Duração estimada: {:.1} dias\n", dias);
        } else {
            mensagem_final += "- Sem consumo recente. Estoque não se esgotará.\n";
        }
    } else {
        mensagem_final += "\n**Estoque:** Informação indisponível.\n";
    }

    let resposta = tratamento_resposta(&mensagem_final).await;
    Json(RespostaIA { resposta })
}

async fn start_http_server() {
    println!("Servidor rodando!");
    let app = Router::new().route("/api/gerar_relatorio", post(gerar_relatorio));
    let listener = TcpListener::bind("0.0.0.0:5300").await.unwrap();
    axum::serve(listener, app.into_make_service()).await.unwrap();
}

async fn rotina_de_insercao() {
    let inicio = Local::now();
    let mut ultimo_registro = None;

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
            insert_data().await;
        }

        if agora.hour() == 0 && agora.minute() == 0 {
            let hoje = agora.date_naive();
            if ultimo_registro != Some(hoje) {
                println!("Registrando estoque diário (meia-noite)...");
                registrar_estoque_diario().await;
                ultimo_registro = Some(hoje);
            }
        }

        sleep(TokioDuration::from_secs(60)).await;
    }
}

#[tokio::main]
async fn main() {
    from_path(Path::new("../.env")).expect("Falha ao carregar .env");
    println!("DATABASE_URL: {:?}", env::var("DATABASE_URL"));
    println!("BODY_APIV2: {:?}", env::var("BODY_APIV2"));
    println!("API_OPENAI: {:?}", env::var("API_OPENAI"));

    tokio::join!(start_http_server(), rotina_de_insercao());
}
