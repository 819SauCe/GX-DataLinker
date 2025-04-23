CREATE TABLE Estoque_and_EstoqueKits (
    id SERIAL PRIMARY KEY,
    codigo_produto VARCHAR(50) NOT NULL,
    quantidade_disponivel INT NOT NULL,
    nome VARCHAR(255) NOT NULL
);