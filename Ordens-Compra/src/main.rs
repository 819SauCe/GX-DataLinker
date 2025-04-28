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
use chrono::NaiveDate;
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

fn parse_date_br(date_str: &str) -> Option<NaiveDate> {
    NaiveDate::parse_from_str(date_str, "%d/%m/%Y").ok()
}

async fn obtain_token() -> Value {
    let client = Client::new();
    let dados: Value = serde_json::from_str(&std::env::var("BODY_APIV2").unwrap()).unwrap();
    let res = client.post("https://global_trade.cr.wk.net.br/wk.api/api/v1/token").json(&dados).send().await.unwrap().json::<Value>().await.unwrap();

    res
}

async fn obtain_ord(token: &str) -> Value {
    let client = Client::new();
    let url = format!("https://global_trade.cr.wk.net.br/wk.api/api/compras/v1/ordem-compra?situacaoAutorizacao=Autorizada&situacaoAtendimento=Pendente");
    let res = client.get(&url).bearer_auth(token).send().await.unwrap().json::<Value>().await.unwrap();

    return res;
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

    println!("Obtendo token");
    let token = obtain_token().await;
    let token_str = token["token"].as_str().unwrap();
    let data = obtain_ord(token_str).await;
    println!("Token pego!");

    let stmt = client.prepare("
        INSERT INTO ord_compra (
            id, chave, data_emissao, data_competencia, data_necessidade,
            id_filial_emitente, id_filial_faturamento, id_natureza_operacao_produto,
            id_fornecedor_transportador, id_classificacao, id_gerencial, id_requisitante,
            id_departamento, id_comprador, id_moeda, observacoes,
            situacao_autorizacao, situacao_atendimento, situacao_integracao,
            data_entrega, numero_dias_entrega, local_entrega,
            id_transportadora, frete_por_conta, frete, seguro,
            despesas_acessorias, valor_total
        ) VALUES (
            $1, $2, $3, $4, $5,
            $6, $7, $8, $9, $10,
            $11, $12, $13, $14, $15, $16,
            $17, $18, $19, $20, $21, $22,
            $23, $24, $25, $26, $27, $28
        ) ON CONFLICT (id) DO NOTHING
    ").await.unwrap();

    let stmt_prod = client.prepare("
        INSERT INTO ord_produtos (id_ordem, id_produto, id_usuario)
        VALUES ($1, $2, $3)
        ON CONFLICT (id_ordem, id_produto) DO UPDATE SET id_usuario = EXCLUDED.id_usuario
    ").await.unwrap();

    if let Some(ords) = data.as_array() {
        for ord in ords {
            let id = ord["id"].as_str().unwrap_or_default();
            let chave = ord["chave"].as_str().unwrap_or_default();
            let data_emissao_dt = parse_date_br(ord["dataEmissao"].as_str().unwrap_or_default());
            let data_competencia_dt = parse_date_br(ord["dataCompetencia"].as_str().unwrap_or_default());
            let data_necessidade_dt = parse_date_br(ord["dataNecessidade"].as_str().unwrap_or_default());
            let id_filial_emitente = ord["idFilialEmitente"].as_str().unwrap_or_default();
            let id_filial_faturamento = ord["idFilialFaturamento"].as_str().unwrap_or_default();
            let id_natureza_operacao_produto = ord["idNaturezaOperacaoProduto"].as_str().unwrap_or_default();
            let id_fornecedor_transportador = ord["idFornecedorTransportador"].as_str().unwrap_or_default();
            let id_classificacao = ord["idClassificacao"].as_str().unwrap_or_default();
            let id_gerencial = ord["idGerencial"].as_str().unwrap_or_default();
            let id_requisitante = ord["idRequisitante"].as_str().unwrap_or_default();
            let id_departamento = ord["idDepartamento"].as_str().unwrap_or_default();
            let id_comprador = ord["idComprador"].as_str().unwrap_or_default();
            let id_moeda = ord["idMoeda"].as_str().unwrap_or_default();
            let observacoes = ord["observacoes"].as_str().unwrap_or_default();
            let situacao_autorizacao = ord["situacaoAutorizacao"].as_str().unwrap_or_default();
            let situacao_atendimento = ord["situacaoAtendimento"].as_str().unwrap_or_default();
            let situacao_integracao = ord["situacaoIntegracao"].as_str().unwrap_or_default();
            let data_entrega_dt = parse_date_br(ord["entrega"]["dataEntrega"].as_str().unwrap_or_default());
            let numero_dias_entrega = ord["entrega"]["numeroDiasEntrega"].as_i64().unwrap_or(0) as i32;
            let local_entrega = ord["entrega"]["localEntrega"].as_str().unwrap_or_default();
            let id_transportadora = ord["transporte"]["idTransportadora"].as_str().unwrap_or_default();
            let frete_por_conta = ord["transporte"]["fretePorConta"].as_str().unwrap_or_default();
            let frete = ord["totais"]["frete"].as_f64().unwrap_or(0.0);
            let seguro = ord["totais"]["seguro"].as_f64().unwrap_or(0.0);
            let despesas_acessorias = ord["totais"]["despesasAcessorias"].as_f64().unwrap_or(0.0);
            let valor_total = ord["totais"]["valorTotal"].as_f64().unwrap_or(0.0);

            client.execute(&stmt, &[
                &id, &chave, &data_emissao_dt, &data_competencia_dt, &data_necessidade_dt,
                &id_filial_emitente, &id_filial_faturamento, &id_natureza_operacao_produto,
                &id_fornecedor_transportador, &id_classificacao, &id_gerencial, &id_requisitante,
                &id_departamento, &id_comprador, &id_moeda, &observacoes,
                &situacao_autorizacao, &situacao_atendimento, &situacao_integracao,
                &data_entrega_dt, &numero_dias_entrega, &local_entrega,
                &id_transportadora, &frete_por_conta, &frete, &seguro,
                &despesas_acessorias, &valor_total
            ]).await.unwrap();

            if let Some(itens) = ord["itens"].as_array() {
                for item in itens {
                    let id_produto = item["produtoServico"]["id"].as_str().unwrap_or_default();
                    if let Some(autorizacoes) = ord["autorizacoes"].as_array() {
                        for auto in autorizacoes {
                            let id_usuario = auto["idUsuario"].as_str().unwrap_or_default();
                            let exists = client.query("SELECT 1 FROM produtos_nota WHERE id = $1", &[&id_produto]).await.unwrap();
                            if !exists.is_empty() {
                                client.execute(&stmt_prod, &[&id, &id_produto, &id_usuario]).await.unwrap();
                                let row = client.query_one("SELECT codigo FROM produtos WHERE id = $1", &[&id_produto]).await.unwrap();
                                let codigo_produto = row.get::<_, String>(0);
                                client.execute("UPDATE ord_produtos SET codigo_produto = $1 WHERE id_ordem = $2 AND id_produto = $3", &[&codigo_produto, &id, &id_produto]).await.unwrap();
                            }
                        }
                    }
                }
            }
        }
    }
}

async fn tratamento_resposta(mensagem: &str) -> String {
    let client = Client::new();
    let api_key = std::env::var("API_OPENAI").expect("API_OPENAI não definida");
    let mut history = MESSAGE_HISTORY.lock().await;

    history.push(json!({"role": "user", "content": mensagem}));
    while history.len() > 10 { history.remove(0); }

    let body = json!({
        "model": "gpt-4-turbo",
        "messages": vec![
            json!({"role": "system", "content": "Organize os dados da Ordem de Compra em formato de relatório objetivo,
             separando por categorias (Dados Gerais, Logística, Financeiro, Observações). Não analise ou sugira ações.
              se receber um código mas sem demais informações diga que o item não existe ou não foi comprado."})
        ].into_iter().chain(history.clone()).collect::<Vec<_>>()
    });

    println!("Mensagem para IA: {}", mensagem);

    let response = client.post("https://api.openai.com/v1/chat/completions")
        .bearer_auth(&api_key)
        .json(&body)
        .send()
        .await;

    match response {
        Ok(resp) => {
            let status = resp.status();
            match resp.text().await {
                Ok(text) => {
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
                    println!("Erro ao ler body da resposta: {}", e);
                    "Erro: body ilegível".to_string()
                }
            }
        },
        Err(e) => {
            println!("Erro na chamada da API: {}", e);
            "Erro: falha na requisição HTTP".to_string()
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

    let user_code = payload.user_message.trim();
    let row_opt = client.query_opt("
        SELECT oc.chave, oc.data_emissao::TEXT, oc.data_competencia::TEXT, oc.data_necessidade::TEXT,
               oc.id_filial_emitente, oc.id_filial_faturamento, oc.id_natureza_operacao_produto,
               oc.id_classificacao, oc.id_gerencial, oc.id_requisitante, oc.id_departamento,
               oc.id_comprador, oc.id_moeda, oc.observacoes, oc.situacao_autorizacao,
               oc.situacao_atendimento, oc.situacao_integracao, oc.data_entrega::TEXT, 
               oc.numero_dias_entrega::TEXT, oc.local_entrega, oc.id_transportadora, 
               oc.frete_por_conta, oc.frete::TEXT, oc.seguro::TEXT, 
               oc.despesas_acessorias::INT, oc.valor_total::INT
        FROM ord_produtos op
        JOIN ord_compra oc ON op.id_ordem = oc.id
        WHERE op.codigo_produto = $1
    ", &[&user_code]).await.unwrap();

    let mensagem_final = if let Some(row) = row_opt {
        let chave: &str = row.get(0);
        let data_emissao: Option<&str> = row.get(1);
        let data_competencia: Option<&str> = row.get(2);
        let data_chegada: Option<&str> = row.get(3);
        let id_filial_eminente: &str = row.get(4);
        let id_filial_faturamento: &str = row.get(5);
        let id_natureza_operacao_produto: &str = row.get(6);
        let id_classificacao: &str = row.get(7);
        let id_gerencial: &str = row.get(8);
        let id_requisitante: &str = row.get(9);
        let id_departamento: &str = row.get(10);
        let id_comprador: &str = row.get(11);
        let id_moeda: &str = row.get(12);
        let observacoes: Option<&str> = row.get(13);
        let situacao_autorizacao: &str = row.get(14);
        let situacao_atendimento: &str = row.get(15);
        let situacao_integracao: &str = row.get(16);
        let data_entrega: Option<&str> = row.get(17);
        let numero_dias_entrega: &str = row.get(18);
        let local_entrega: &str = row.get(19);
        let id_transportadora: &str = row.get(20);
        let frete_por_conta: &str = row.get(21);
        let frete_valor: &str = row.get(22);
        let seguro_valor: &str = row.get(23);
        let despesas_acessoria: i32 = row.get(24);
        let valor_total: i32 = row.get(25);

        format!(
            "Dados da Ordem de Compra:\n\
            Chave: {}\n\
            Data Emissão: {}\n\
            Data Competência: {}\n\
            Data Chegada: {}\n\
            Filial Emitente: {}\n\
            Filial Faturamento: {}\n\
            Natureza da Operação: {}\n\
            Classificação: {}\n\
            Gerencial: {}\n\
            Requisitante: {}\n\
            Departamento: {}\n\
            Comprador: {}\n\
            Moeda: {}\n\
            Observações: {}\n\
            Autorização: {}\n\
            Atendimento: {}\n\
            Integração: {}\n\
            Data de Entrega: {}\n\
            Nº Dias Entrega: {}\n\
            Local Entrega: {}\n\
            Transportadora: {}\n\
            Frete por Conta: {}\n\
            Valor Frete: {}\n\
            Valor Seguro: {}\n\
            Despesas Acessórias: {}\n\
            Valor Total: {}",
            chave,
            data_emissao.unwrap_or("N/A"),
            data_competencia.unwrap_or("N/A"),
            data_chegada.unwrap_or("N/A"),
            id_filial_eminente, id_filial_faturamento, id_natureza_operacao_produto,
            id_classificacao, id_gerencial, id_requisitante, id_departamento, id_comprador,
            id_moeda, observacoes.unwrap_or("N/A"), situacao_autorizacao, situacao_atendimento,
            situacao_integracao, data_entrega.unwrap_or("N/A"), numero_dias_entrega,
            local_entrega, id_transportadora, frete_por_conta,
            frete_valor, seguro_valor, despesas_acessoria, valor_total
        )
    } else {
        payload.user_message.clone()
    };

    let resposta = tratamento_resposta(&mensagem_final).await;
    Json(RespostaIA { resposta })
}

async fn start_http_server() {
    println!("Servidor rodando!");
    let app = Router::new().route("/api/gerar_relatorio", post(gerar_relatorio));
    
    match TcpListener::bind("0.0.0.0:5100").await {
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
    insert_data().await;
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
            insert_data().await;
        }

        sleep(TokioDuration::from_secs(60)).await;
    }
}

#[tokio::main]
async fn main() {
    from_path(Path::new("../.env")).expect("Falha ao carregar .env");

    tokio::join!(
        start_http_server(),
        rotina_de_insercao()
    );
}
