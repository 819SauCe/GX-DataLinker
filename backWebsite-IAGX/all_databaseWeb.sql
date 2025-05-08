CREATE TABLE users (
    id SERIAL PRIMARY KEY,
    username VARCHAR(255) NOT NULL,
    password VARCHAR(255) NOT NULL,
    email VARCHAR(255) NOT NULL,
    type VARCHAR(255),
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE user_permissions (
    user_id INT PRIMARY KEY REFERENCES users(id),
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    ordens_compra BOOLEAN NOT NULL,
    estoque BOOLEAN NOT NULL,
    gestor_produto BOOLEAN NOT NULL,
    notas_fiscais BOOLEAN NOT NULL,
    auto_pregao BOOLEAN NOT NULL
);

CREATE TABLE connections (
    id SERIAL PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    description VARCHAR(500),
    leader_1 VARCHAR(255),
    leader_2 VARCHAR(255),
    leader_3 VARCHAR(255),
    ip VARCHAR(255) NOT NULL,
    port VARCHAR(255) NOT NULL
)