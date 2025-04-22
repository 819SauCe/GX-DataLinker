CREATE TABLE notas_fiscais (
    id TEXT PRIMARY KEY,
    id_uuid UUID NOT NULL,
    chave TEXT,
    idfilial TEXT,
    tipo TEXT,
    dataemissao TIMESTAMP,
    dataentradasaida TIMESTAMP,
    numero TEXT,
    codigocliente TEXT,
    nomecliente TEXT
);