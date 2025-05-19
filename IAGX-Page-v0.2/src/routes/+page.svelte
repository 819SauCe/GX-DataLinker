<script>
  import { onMount } from "svelte";

  let containers = [];
  let name = "",
    description = "",
    leader1 = "",
    leader2 = "",
    leader3 = "",
    ip = "",
    port = "";
  let editingId = null;
  let isAdmin = false;
  let showOverlay = true;

function hideOverlay() {
  showOverlay = false;
}

  const fetchContainers = async (token) => {
    try {
      const res = await fetch(`${import.meta.env.VITE_API_URL}/containers`, {
        headers: token ? { Authorization: `Bearer ${token}` } : {},
      });
      if (res.ok) {
        containers = await res.json();
      } else {
        console.error("Erro ao buscar containers:", res.status);
      }
    } catch (err) {
      console.error("Erro de rede:", err);
    }
  };

  const saveContainer = async () => {
    const token = localStorage.getItem("token");
    const data = {
      name,
      description,
      leader_1: leader1,
      leader_2: leader2,
      leader_3: leader3,
      ip,
      port,
    };
    const headers = {
      "Content-Type": "application/json",
      ...(token && { Authorization: `Bearer ${token}` }),
    };

    if (editingId) {
      await fetch(`${import.meta.env.VITE_API_URL}/containers/${editingId}`, {
        method: "PUT",
        headers,
        body: JSON.stringify(data),
      });
    } else {
      await fetch(`${import.meta.env.VITE_API_URL}/containers`, {
        method: "POST",
        headers,
        body: JSON.stringify(data),
      });
    }

    name = "";
    description = "";
    leader1 = "";
    leader2 = "";
    leader3 = "";
    ip = "";
    port = "";
    editingId = null;
    await fetchContainers(token);
  };

  const editContainer = (c) => {
    name = c.name;
    description = c.description;
    leader1 = c.leader_1;
    leader2 = c.leader_2;
    leader3 = c.leader_3;
    ip = c.ip;
    port = c.port;
    editingId = c.id;
  };

  const deleteContainer = async (id) => {
    const token = localStorage.getItem("token");
    await fetch(`${import.meta.env.VITE_API_URL}/containers/${id}`, {
      method: "DELETE",
      headers: token ? { Authorization: `Bearer ${token}` } : {},
    });
    await fetchContainers(token);
  };

  onMount(async () => {
    const cookieTheme = document.cookie
    .split("; ")
    .find((row) => row.startsWith("theme="))
    ?.split("=")[1] || "default";

  document.documentElement.classList.remove("theme-default", "theme-globalx", "theme-dark");
  document.documentElement.classList.add(`theme-${cookieTheme}`);

    const token = localStorage.getItem("token");
    if (!token || token.split(".").length !== 3) {
      console.error("Token ausente ou inválido");
      localStorage.removeItem("token");
      window.location.href = "/login";
      return;
    }

    try {
      const payload = JSON.parse(atob(token.split(".")[1]));
      const now = Math.floor(Date.now() / 1000);
      if (payload.exp < now) {
        console.error("Token expirado");
        localStorage.removeItem("token");
        window.location.href = "/login";
        return;
      }
      isAdmin = payload.role === "admin";
    } catch (e) {
      console.error("Token inválido", e);
      localStorage.removeItem("token");
      window.location.href = "/login";
      return;
    }

    await fetchContainers(token);
  });
</script>

<div class="__body__main_menu__ flex flex-wrap gap-4 justify-center">
  {#each containers as c}
    <div class="__container mb-4 flex flex-col gap-2 p-2">
      <a
        href={`/container/${c.id}?name=${encodeURIComponent(c.name)}`}
        class="text-lg font-bold text-orange-400 hover:underline"
      >
        <h3>{c.name}</h3>
        <p class="text-sm text-white"></p>
        <p class="descricao">{c.description}</p>
        <h4>Líderes:</h4>
        <div class="lideres" style="display: flex;">
          <p class="lideres__container">{c.leader_1}</p>
          <p class="lideres__container">{c.leader_2}</p>
          <p class="lideres__container">{c.leader_3}</p>
        </div>
        <h4 class="mt-2">{c.ip}:{c.port}</h4>
      </a>
      {#if isAdmin}
        <div class="flex gap-2 mt-auto">
          <button on:click={() => editContainer(c)}>Editar</button>
          <button on:click={() => deleteContainer(c.id)}>Deletar</button>
        </div>
      {/if}
    </div>
  {/each}

  {#if isAdmin}
  <div class="admin-form" style="position: relative; width: 20rem; height: 15rem;">
    {#if showOverlay}
      <div
        on:click={hideOverlay} class="overlay"
        style="position: absolute; z-index: 10; width: 100%; height: 100%; background-color: #151d25; display: flex; align-items: center; justify-content: center; color: white; cursor: pointer; border-radius: 10px; font-size: 5rem;border: 1px solid #5e5e5e;">
        +
      </div>
    {/if}

    <div
      class="mb-4"
      style="display: flex; gap: 0.3rem; flex-direction: column; width: 100%; height: 100%; border-radius: 10px; padding: 0.3rem;">
      <input bind:value={name} placeholder="Nome" required />
      <input bind:value={description} placeholder="Descrição" />
      <input bind:value={leader1} placeholder="Líder 1" />
      <input bind:value={leader2} placeholder="Líder 2" />
      <input bind:value={leader3} placeholder="Líder 3" />
      <input bind:value={ip} placeholder="IP" required />
      <input bind:value={port} placeholder="Porta" required />
      <button type="button" on:click={saveContainer}>
        {editingId ? "Atualizar" : "Adicionar"}
      </button>
    </div>
  </div>
{/if}
</div>

<style>
  .__body__main_menu__ {
    display: flex;
    justify-content: center;
    align-items: center;
    background-color: var(--bg-color);
    font-family: var(--font);
    min-height: 92vh;
    height: auto;
  }

  .__body__main_menu__ h4 {
    font-size: 1.1rem;
    margin: 0%;
    padding: 0%;
  }

  .__body__main_menu__ p {
    font-size: 0.8rem;
    margin: 0%;
    padding: 0%;
  }

  .__container {
    display: flex;
    background-color: var(--container-background);
    border: 1px solid var(--border-color);
    border-radius: 10px;
    width: 20rem;
    height: 15rem;
    padding: 0.3rem;
    color: var(--text-color);
    transition: all 0.3s ease-in-out;
    flex-direction: column;
    align-items: flex-start;
    justify-content: flex-start;
    margin-top: 1rem;
  }

  .__container:hover {
    width: 21rem;
    height: 16rem;
  }

  .__container p {
    word-wrap: break-word;
    overflow-wrap: anywhere;
  }

  .admin-form {
    position: relative;
    width: 20rem;
    height: 15rem;
    transition: all 0.3s ease-in-out;
  }

  .admin-form:hover {
    width: 21rem !important;
    height: 16rem !important;
  }

  .admin-form:hover .overlay,
  .admin-form:hover .mb-4 {
    width: 100%;
    height: 100%;
  }

  .overlay {
    position: absolute;
    z-index: 10;
    width: 100%;
    height: 100%;
    background-color: var(--container-background);
    display: flex;
    align-items: center;
    justify-content: center;
    color: var(--text-color);
    cursor: pointer;
    border-radius: 10px;
    font-size: 5rem;
    border: 1px solid var(--border-color);
    transition: all 0.3s ease-in-out;
    pointer-events: auto;
  }

  .mb-4 {
    background-color: var(--container-background);
    border: 1px solid var(--border-color);
    transition: all 0.3s ease-in-out;
  }

  input {
    height: 1.5rem;
    border-radius: 10rem;
    border: none;
    padding-left: 10px;
    background-color: #343a40;
    color: var(--text-color);
  }

  input::placeholder {
    color: #b0b0b0;
  }

  button {
    height: 1.5rem;
    border-radius: 7px;
    border: none;
    background-color: var(--accent-color);
    color: rgb(0, 0, 0);
    transition: all 0.2s ease-in-out;
  }

  button:hover {
    cursor: pointer;
    background-color: var(--accent-color);
  }

  a {
    color: var(--text-color);
    text-decoration: none;
    transition: all 0.2s ease-in-out;
  }

  .lideres {
    margin-top: 0.5rem;
    gap: 0.3rem;
  }

  .lideres__container {
    display: flex;
    justify-content: center;
    align-items: center;
    width: 5rem;
    height: 1.9rem;
    line-height: 1;
    background-color: #373737;
    border-radius: 7px;
  }

  .lideres__container p {
    padding-bottom: 5px;
  }

  .descricao {
    color: #b0b0b0;
  }
</style>
