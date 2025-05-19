# Plataforma de Integração e Análise de Dados com IA(GX-DataLinker)
## Este projeto é um sistema web completo com frontend e backend, desenvolvido com foco em performance, integração e automação de relatórios empresariais personalizados, especialmente para atender as necessidades da **Global-Trade**, **Global-X**, **BioSigma** e **E-Lab**. 
### A stack utilizada inclui:

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
- Vite (Variavel global)
- Ter Créditos na openIA
- API Key da OpenAI (.env).
- Login no WKrada API (.env).
- Conexão com o banco de dados (.env).
> 💡 **Funcionários da Global-X**: o sistema completo pode ser acessado na rede interna em:  
> `\\GlobalTrade\Público\João Oqueres\Help - GX_DataLinker`
=======
# sv

Everything you need to build a Svelte project, powered by [`sv`](https://github.com/sveltejs/cli).

## Creating a project

If you're seeing this, you've probably already done this step. Congrats!

```bash
# create a new project in the current directory
npx sv create

# create a new project in my-app
npx sv create my-app
```

## Developing

Once you've created a project and installed dependencies with `npm install` (or `pnpm install` or `yarn`), start a development server:

```bash
npm run dev

# or start the server and open the app in a new browser tab
npm run dev -- --open
```

## Building

To create a production version of your app:

```bash
npm run build
```

You can preview the production build with `npm run preview`.
