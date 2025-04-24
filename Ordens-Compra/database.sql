CREATE TABLE ord_compra (
    id TEXT PRIMARY KEY,
    chave TEXT,
    data_emissao DATE,
    data_competencia DATE,
    data_necessidade DATE,
    id_filial_emitente TEXT,
    id_filial_faturamento TEXT,
    id_natureza_operacao_produto TEXT,
    id_fornecedor_transportador TEXT,
    id_classificacao TEXT,
    id_gerencial TEXT,
    id_requisitante TEXT,
    id_departamento TEXT,
    id_comprador TEXT,
    id_moeda TEXT,
    observacoes TEXT,
    situacao_autorizacao TEXT,
    situacao_atendimento TEXT,
    situacao_integracao TEXT,
    data_entrega DATE,
    numero_dias_entrega INTEGER,
    local_entrega TEXT,
    id_transportadora TEXT,
    frete_por_conta TEXT,
    frete DOUBLE PRECISION,
    seguro DOUBLE PRECISION,
    despesas_acessorias DOUBLE PRECISION,
    valor_total DOUBLE PRECISION
);

CREATE TABLE ord_produtos (
    id SERIAL PRIMARY KEY,
    id_ordem TEXT REFERENCES ord_compra(id),
    id_produto TEXT REFERENCES produtos(id),
    id_usuario TEXT,
    codigo_produto VARCHAR(50)
);

ALTER TABLE ord_produtos ADD CONSTRAINT ord_produtos_unique UNIQUE (id_ordem, id_produto);
