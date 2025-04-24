# Plataforma de Integração e Análise de Dados com IA
## Este projeto é um sistema web completo com frontend e backend, desenvolvido com foco em performance, integração e automação de relatórios empresariais personalizados. A stack utilizada inclui:

- Frontend: Svelte, HTML, CSS e JavaScript.
- Backend: Python (Django).
- Servidores da IA: Rust.
- Sistema: AWS.
- Docker: focado em alta performance.
- Banco de Dados: PostgreSQL.

## O que ele faz?
- O sistema coleta dados de estoque e operações comerciais através da integração com a API REST da WK Radar, utilizando múltiplos serviços para:
- Capturar dados de produtos, notas fiscais e ordens de compra.
- Armazenar essas informações no banco de dados da empresa.
- Gerar relatórios inteligentes com o auxílio da OpenAI, de forma customizada conforme os comandos dos usuários.

Esse processo é executado automaticamente com monitoramento contínuo e respostas
rápidas via API HTTP, permitindo acesso fácil aos relatórios via requisições POST.

# Pré requisitos:
- Docker.
- Rust instalado.
- Python instalado.
- PostgreSQL instalado.
- Node e NPM.
- API Key da OpenAI (.env).
- Login no WKrada API (.env).
- Conexão com o banco de dados (.env).
### Caso você seja funcionario da Global-X pode acessar a pasta do sistema e seguir esse caminho -> \GlobalTrade\Público\João Oqueres\Help - GX_DataLinker
