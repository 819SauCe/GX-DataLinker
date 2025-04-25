CREATE TABLE produtos_gp (
    id_produto TEXT PRIMARY KEY,
    codigo TEXT,
    nome TEXT,
    descricao TEXT,
    marca TEXT,
    inativo TEXT,
    ipi TEXT
);

ALTER TABLE produtos_gp ADD COLUMN id_marca TEXT;
