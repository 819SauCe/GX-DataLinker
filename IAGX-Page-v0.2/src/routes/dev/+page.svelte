<script>
<<<<<<< HEAD
    let users = [
        { id: "1", name: "João", password: "123", email: "joao@exemplo.com", type: "admin", create_date: "19/05/2025" },
        { id: "2", name: "Maria", password: "123", email: "maria@exemplo.com", type: "user", create_date: "01/01/2025" },
=======
    import { onMount } from "svelte";

    let isOpen = false;
    let name = "";
    let avatar = "/avatars/no-user.webp";
    let currentIcon = "/favicon.png";
    let users = [
        {
            id: "1",
            name: "João",
            password: "123",
            email: "joao@exemplo.com",
            type: "admin",
            create_date: "19/05/2025",
        },
        {
            id: "2",
            name: "Maria",
            password: "123",
            email: "maria@exemplo.com",
            type: "user",
            create_date: "01/01/2025",
        },
>>>>>>> master
    ];

    function addUser() {
        users = [...users, { name: "", email: "" }];
    }

    function removeUser(index) {
        users = users.filter((_, i) => i !== index);
    }

    function saveUser(index) {
        const user = users[index];
        console.log("Salvar:", user);
    }
<<<<<<< HEAD
=======

    function updateIcon() {
        const icon = getComputedStyle(document.documentElement)
            .getPropertyValue("--icon")
            .trim()
            .replace(/"/g, "");

        if (icon) currentIcon = icon;
    }

    function getThemeFromCookie() {
        const match = document.cookie.match(/theme=([^;]+)/);
        return match ? match[1] : "default";
    }

    onMount(async () => {
        name = localStorage.getItem("username");

        if (name) {
            try {
                const res = await fetch(
                    `https://api.iagx.com.br/perfil/${name}`,
                );
                if (res.ok) {
                    const data = await res.json();
                    if (data.avatar_url) {
                        avatar = `https://api.iagx.com.br${data.avatar_url}`;
                    }
                }
            } catch (err) {
                console.error("Erro ao buscar avatar:", err);
            }
        }

        updateIcon();
        const observer = new MutationObserver(updateIcon);
        observer.observe(document.documentElement, {
            attributes: true,
            attributeFilter: ["class"],
        });
    });
>>>>>>> master
</script>

<main class="main-wrapper d-flex align-items-center justify-content-center">
    <div class="ferramentas">
        <h1>Ferramentas</h1>
        <p>Ferramentas úteis para o desenvolvimento da IA, Página e Back.</p>
        <div class="containers-ferramentas gap-4">
            <div class="card status-card text-light text-center p-4">
                <a
                    href="https://status.iagx.com.br"
                    target="_blank"
                    class="text-decoration-none text-light d-flex flex-column h-100"
                >
                    <h2 class="h4 mb-3">Status - IAGX</h2>
                    <p class="flex-grow-1 mb-3">
                        Faz o monitoramento de todas as IA,<br />
                        e também do próprio back end da página,<br />
                        com protocolos de ping e registro<br />
                        de inatividade.
                    </p>
                    <p class="small text-muted mb-0">
                        https://status.iagx.com.br
                    </p>
                </a>
            </div>

            <div class="card status-card text-light text-center p-4">
                <a
                    href="https://downdetector.com.br/fora-do-ar/openai/"
                    target="_blank"
                    class="text-decoration-none text-light d-flex flex-column h-100"
                >
                    <h2 class="h4 mb-3">Status - OpenIA</h2>
                    <p class="flex-grow-1 mb-3">
                        Monitoramento pelo downdetector<br />
                        para analisar se o ChatGPT ou a<br />
                        OpenIA está funcionando corretamente.
                    </p>
                    <p class="small text-muted mb-0">
                        https://downdetector.com.br/fora-do-ar/openai/
                    </p>
                </a>
            </div>
        </div>

        <h2 class="mt-5 mb-3">Configuração de usuario</h2>
        <table class="table table-dark table-bordered table-hover align-middle">
            <thead>
                <tr>
                    <th>ID</th>
                    <th>Username</th>
                    <th>Password</th>
                    <th>Email</th>
                    <th>Type</th>
                    <th>create_at</th>
                    <th style="width: 120px">Ações</th>
                </tr>
            </thead>
            <tbody>
                {#each users as user, i}
                    <tr>
                        <td>
                            <input
                                type="text"
                                class="form-control bg-dark text-white border-0"
                                bind:value={user.id}
                            />
                        </td>
                        <td>
                            <input
                                type="email"
                                class="form-control bg-dark text-white border-0"
                                bind:value={user.name}
                            />
                        </td>
                        <td>
                            <input
                                type="text"
                                class="form-control bg-dark text-white border-0"
<<<<<<< HEAD
                                bind:value={user.password}/>
=======
                                bind:value={user.password}
                            />
>>>>>>> master
                        </td>
                        <td>
                            <input
                                type="text"
                                class="form-control bg-dark text-white border-0"
<<<<<<< HEAD
                                bind:value={user.email}/>
=======
                                bind:value={user.email}
                            />
>>>>>>> master
                        </td>
                        <td>
                            <input
                                type="text"
                                class="form-control bg-dark text-white border-0"
<<<<<<< HEAD
                                bind:value={user.type}/>
=======
                                bind:value={user.type}
                            />
>>>>>>> master
                        </td>
                        <td>
                            <input
                                type="text"
                                class="form-control bg-dark text-white border-0"
<<<<<<< HEAD
                                bind:value={user.create_date}/>
=======
                                bind:value={user.create_date}
                            />
>>>>>>> master
                        </td>
                        <td>
                            <div class="d-flex gap-2">
                                <button
                                    class="btn btn-sm btn-outline-success"
                                    on:click={() => saveUser(i)}>💾</button
                                >
                                <button
                                    class="btn btn-sm btn-outline-danger"
                                    on:click={() => removeUser(i)}>🗑</button
                                >
                            </div>
                        </td>
                    </tr>
                {/each}
            </tbody>
        </table>

        <div class="text-end mt-3">
            <button class="btn btn-outline-primary" on:click={addUser}
                >➕ Adicionar Usuário</button
            >
        </div>
    </div>
</main>

<style>
    .main-wrapper {
        width: 100%;
        min-height: 100vh;
        background-color: #2a2a2a;
        padding: 5rem 1rem;
    }

    .ferramentas {
        width: 90rem;
        border-radius: 1rem;
        text-align: center;
        background-color: #151d25;
        border: 1px solid #5e5e5e;
        color: white;
        padding: 2rem;
    }

    .containers-ferramentas {
        display: flex;
        justify-content: center;
        align-items: stretch;
        gap: 2rem;
        margin-top: 2rem;
    }

    .status-card {
        flex: 1;
        max-width: 380px;
        background-color: #151d25;
        border: 1px solid #5e5e5e;
        border-radius: 1rem;
        transition:
            transform 0.3s ease,
            box-shadow 0.3s ease;
        display: flex;
        flex-direction: column;
    }

    .status-card:hover {
        transform: translateY(-5px);
        box-shadow: 0 0 20px rgba(255, 255, 255, 0.1);
    }
</style>
