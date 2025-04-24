CREATE TABLE Estoque_and_EstoqueKits (
    id SERIAL PRIMARY KEY,
    codigo_produto VARCHAR(50) NOT NULL,
    quantidade_disponivel VARCHAR(15) NOT NULL,
    nome VARCHAR(255) NOT NULL
);

ALTER TABLE Estoque_and_EstoqueKits ADD CONSTRAINT unique_codigo_produto UNIQUE (codigo_produto);
