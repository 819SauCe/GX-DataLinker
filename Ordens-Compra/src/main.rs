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
use tower_http::cors::{Any, CorsLayer};
use axum::{Router, routing::{post, options}, extract::Json, response::IntoResponse};
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
    let res = client.post("http://global_trade.cr.wk.net.br/wk.api/api/v1/token").json(&dados).send().await.unwrap().json::<Value>().await.unwrap();

    res
}

//Inserir Produtos

async fn sync_produtos(token: &str, client: &tokio_postgres::Client) {
    let reqwest_client = Client::new();

    let res = reqwest_client
        .get("http://global_trade.cr.wk.net.br/wk.api/api/empresarial/v1/produto")
        .bearer_auth(token)
        .send()
        .await
        .expect("Falha na requisição de produtos")
        .json::<Value>()
        .await
        .expect("Falha ao decodificar JSON");

    if let Some(array) = res.as_array() {
        for produto in array {
            let id = produto.get("id").and_then(|v| v.as_str()).unwrap_or("");
            let codigo = produto.get("codigo").and_then(|v| v.as_str()).unwrap_or("");
            let nome = produto.get("nome").and_then(|v| v.as_str()).unwrap_or("");

            if !id.is_empty() && !codigo.is_empty() {
                let _ = client
                    .execute(
                        "INSERT INTO produtos (id, codigo, nome)
                         VALUES ($1, $2, $3)
                         ON CONFLICT (id) DO UPDATE
                         SET codigo = EXCLUDED.codigo,
                             nome = EXCLUDED.nome",
                        &[&id, &codigo, &nome],
                    )
                    .await;
            }
        }
        println!("Produtos sincronizados: {}", array.len());
    } else {
        println!("Nenhum produto retornado pela API.");
    }
}

//Fim de inserir produtos


async fn obtain_ord(token: &str) -> Value {
    let client = Client::new();
    let url = format!("http://global_trade.cr.wk.net.br/wk.api/api/compras/v1/ordem-compra?situacaoAutorizacao=Autorizada&situacaoAtendimento=Pendente");
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

    //Inserindo Produtos
    sync_produtos(token_str, &client).await;
  

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
INSERT INTO ord_produtos (
    id_ordem, id_produto, id_usuario, quantidade, data_necessidade, valor_unitario, moeda
)
VALUES ($1, $2, $3, $4, $5, $6, $7)
ON CONFLICT (id_ordem, id_produto) DO UPDATE SET
    id_usuario = EXCLUDED.id_usuario,
    quantidade = EXCLUDED.quantidade,
    data_necessidade = EXCLUDED.data_necessidade,
    valor_unitario = EXCLUDED.valor_unitario,
    moeda = EXCLUDED.moeda
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

            let res = client.execute(&stmt, &[
                &id, &chave, &data_emissao_dt, &data_competencia_dt, &data_necessidade_dt,
                &id_filial_emitente, &id_filial_faturamento, &id_natureza_operacao_produto,
                &id_fornecedor_transportador, &id_classificacao, &id_gerencial, &id_requisitante,
                &id_departamento, &id_comprador, &id_moeda, &observacoes,
                &situacao_autorizacao, &situacao_atendimento, &situacao_integracao,
                &data_entrega_dt, &numero_dias_entrega, &local_entrega,
                &id_transportadora, &frete_por_conta, &frete, &seguro,
                &despesas_acessorias, &valor_total
            ]).await;
            
            match res {
                Ok(_) => (),
                Err(e) => eprintln!("Erro ao inserir em ord_compra para id {}: {}", id, e),
            }
            

            if let Some(itens) = ord["itens"].as_array() {
                for item in itens {
                    let id_produto = item["produtoServico"]["id"].as_str().unwrap_or_default();
            
                    // Verifica se produto existe
                    let exists = client
                        .query("SELECT 1 FROM produtos WHERE id = $1", &[&id_produto])
                        .await
                        .unwrap();
            
                    if exists.is_empty() {
                        println!("Produto {} não existe em produtos. Ignorando...", id_produto);
                        continue;
                    }
            
                    // Continua apenas se houver autorizações
                    if let Some(autorizacoes) = ord["autorizacoes"].as_array() {
                        for auto in autorizacoes {
                            let id_usuario = auto["idUsuario"].as_str().unwrap_or_default();
                            let quantidade = item["quantidadeSolicitada"].as_f64().unwrap_or(0.0);

                            let data_necessidade = parse_date_br(item["dataNecessidade"].as_str().unwrap_or_default());
                            let valor_unitario = item["valorUnitario"].as_f64().unwrap_or(0.0);

                            let moeda = ord["idMoeda"].as_str().unwrap_or("N/A");

                                        
                            // Primeiro: garante que a linha em ord_produtos exista (ou atualiza id_usuario se já existir)
                         if let Err(e) = client.execute(&stmt_prod, &[
                            &id, 
                            &id_produto, 
                            &id_usuario,
                            &quantidade.to_string(),             
                            &data_necessidade, 
                            &valor_unitario.to_string(),        
                            &moeda
                        ]).await {
                            eprintln!("Erro ao inserir/atualizar ord_produtos ({} - {}): {}", id, id_produto, e);
                            continue;
                        }




            
                            // Segundo: atualiza o campo codigo_produto
                            let row_opt = client
                                .query_opt("SELECT codigo FROM produtos WHERE id = $1", &[&id_produto])
                                .await;
            
                            match row_opt {
                                Ok(Some(row)) => {
                                    let codigo_produto: String = row.get(0);
                                    let result = client.execute(
                                        "UPDATE ord_produtos SET codigo_produto = $1 WHERE id_ordem = $2 AND id_produto = $3",
                                        &[&codigo_produto, &id, &id_produto]
                                    ).await;
            
                                    match result {
                                        Ok(rows) if rows > 0 => (),
                                        Ok(_) => eprintln!(
                                            "Nenhuma linha atualizada para produto {} na ordem {}. Verifique se o INSERT foi feito corretamente.",
                                            id_produto, id
                                        ),
                                        Err(e) => eprintln!(
                                            "Erro no UPDATE de ord_produtos para id_produto {}: {}",
                                            id_produto, e
                                        ),
                                    }
                                },
                                Ok(None) => {
                                    eprintln!("Produto {} não encontrado para atualizar código.", id_produto);
                                }
                                Err(e) => {
                                    eprintln!("Erro na consulta de produto {}: {}", id_produto, e);
                                }
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

    let user_code = payload.user_message.trim().to_uppercase();
    let rows_result = client.query(
        "
        SELECT 
            oc.chave, 
            oc.data_emissao::TEXT, 
            oc.data_competencia::TEXT, 
            oc.data_necessidade::TEXT,
            emp_emit.nome_fantasia AS nome_emitente,
            emp_fat.nome_fantasia AS nome_faturamento,
            oc.id_natureza_operacao_produto,
            oc.id_classificacao, 
            oc.id_gerencial, 
            oc.id_requisitante, 
            oc.id_departamento, 
            oc.id_comprador,
            moe.simbolo AS moeda_simbolo,
            oc.observacoes, 
            oc.situacao_autorizacao,
            oc.situacao_atendimento, 
            oc.situacao_integracao, 
            oc.data_entrega::TEXT, 
            oc.numero_dias_entrega::TEXT, 
            oc.local_entrega, 
            oc.id_transportadora, 
            oc.frete_por_conta, 
            oc.frete::TEXT, 
            oc.seguro::TEXT, 
            oc.despesas_acessorias::INT, 
            oc.valor_total::INT,
            prod.nome AS nome_produto,
            op.quantidade,
            op.valor_unitario
        FROM ord_produtos op
        JOIN ord_compra oc ON op.id_ordem = oc.id
        LEFT JOIN empresas_wk emp_emit ON emp_emit.id = oc.id_filial_emitente
        LEFT JOIN empresas_wk emp_fat ON emp_fat.id = oc.id_filial_faturamento
        LEFT JOIN moedas_indices moe ON moe.id = oc.id_moeda
        LEFT JOIN produtos prod ON prod.id = op.id_produto
        WHERE op.codigo_produto = $1
        ", &[&user_code]).await;

    let mensagem_final = match rows_result {
        Ok(rows) if !rows.is_empty() => {
            let mut msg = String::new();

            for row in rows {
                let chave: &str = row.get(0);
                let data_emissao: Option<&str> = row.get(1);
                let data_competencia: Option<&str> = row.get(2);
                let data_necessidade: Option<&str> = row.get(3);
                let nome_emitente: Option<&str> = row.get(4);
                let nome_faturamento: Option<&str> = row.get(5);
                let id_natureza_operacao_produto: &str = row.get(6);
                let id_classificacao: &str = row.get(7);
                let id_gerencial: &str = row.get(8);
                let id_requisitante: &str = row.get(9);
                let id_departamento: &str = row.get(10);
                let id_comprador: &str = row.get(11);
                let moeda_simbolo: Option<&str> = row.get(12);
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
                let nome_produto: Option<&str> = row.get(26);
                let quantidade: Option<&str> = row.get(27);
                let valor_unitario: Option<&str> = row.get(28);
                
                let valor_unitario_fmt = format!(
                    "{} {}",
                    valor_unitario.unwrap_or("N/A"),
                    moeda_simbolo.unwrap_or("N/A")
                );

                let linha = format!(
                    "Dados da Ordem de Compra:\n
                    Chave: {}\n
                    Data Emissão: {}\n
                    Data Competência: {}\n
                    Previsão Chegada: {}\n
                    Filial Emitente: {}\n
                    Filial Faturamento: {}\n
                    Natureza da Operação: {}\n
                    Classificação: {}\n
                    Gerencial: {}\n
                    Requisitante: {}\n
                    Departamento: {}\n
                    Comprador: {}\n
                    Moeda: {}\n
                    Observações: {}\n
                    Autorização: {}\n
                    Atendimento: {}\n
                    Integração: {}\n
                    Data de Entrega: {}\n
                    Nº Dias Entrega: {}\n
                    Local Entrega: {}\n
                    Transportadora: {}\n
                    Frete por Conta: {}\n
                    Valor Frete: {}\n
                    Valor Seguro: {}\n
                    Despesas Acessórias: {}\n
                    Valor Total: {}\n
                    Produto: {}\n
                    Quantidade: {}\n
                    Valor Unitário: {}\n\n",
                    chave,
                    data_emissao.unwrap_or("N/A"),
                    data_competencia.unwrap_or("N/A"),
                    data_necessidade.unwrap_or("N/A"),
                    nome_emitente.unwrap_or("N/A"),
                    nome_faturamento.unwrap_or("N/A"),
                    id_natureza_operacao_produto, id_classificacao, id_gerencial, id_requisitante,
                    id_departamento, id_comprador,
                    moeda_simbolo.unwrap_or("N/A"),
                    observacoes.unwrap_or("N/A"), situacao_autorizacao, situacao_atendimento,
                    situacao_integracao, data_entrega.unwrap_or("N/A"), numero_dias_entrega,
                    local_entrega, id_transportadora, frete_por_conta,
                    frete_valor, seguro_valor, despesas_acessoria, valor_total,
                    nome_produto.unwrap_or("N/A"),
                    quantidade.unwrap_or("N/A"),
                    valor_unitario_fmt
                );

                msg.push_str(&linha);
            }

            msg
        }
        Ok(_) => format!("Nenhuma ordem de compra encontrada para o código '{}'.", user_code),
        Err(e) => {
            eprintln!("Erro ao consultar a ordem de compra para '{}': {}", user_code, e);
            format!("Erro ao consultar a ordem de compra para '{}'.", user_code)
        }
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
    from_path(Path::new("/opt/IAGX-Page-v0.2/.env")).expect("Falha ao carregar .env");

    tokio::join!(
        start_http_server(),
        rotina_de_insercao()
    );
}