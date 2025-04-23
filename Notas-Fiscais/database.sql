CREATE TABLE notas_fiscais (
    id TEXT PRIMARY KEY,
    id_uuid UUID NOT NULL,
    chave TEXT UNIQUE NOT NULL,
    idfilial TEXT,
    tipo TEXT,
    dataemissao TIMESTAMP,
    dataentradasaida TIMESTAMP,
    numero TEXT,
    codigocliente TEXT,
    nomecliente TEXT
);

CREATE TABLE produtos (
    id TEXT PRIMARY KEY,
    codigo TEXT,
    nome TEXT,
    descricao TEXT,
    tipo TEXT,
    preco_venda DOUBLE PRECISION,
    peso_bruto DOUBLE PRECISION,
    peso_liquido DOUBLE PRECISION,
    classificacao TEXT,
    referencia TEXT,
    gtin TEXT
);

CREATE TABLE itens_nota (
    id_nota UUID NOT NULL,
    id_produto TEXT NOT NULL,
    valor_total DOUBLE PRECISION,
    CONSTRAINT itens_nota_unique UNIQUE (id_nota, id_produto)
);