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

  const fetchContainers = async (token) => {
    try {
      const res = await fetch("http://localhost:3000/containers", {
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
      await fetch(`http://localhost:3000/containers/${editingId}`, {
        method: "PUT",
        headers,
        body: JSON.stringify(data),
      });
    } else {
      await fetch("http://localhost:3000/containers", {
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
    await fetch(`http://localhost:3000/containers/${id}`, {
      method: "DELETE",
      headers: token ? { Authorization: `Bearer ${token}` } : {},
    });
    await fetchContainers(token);
  };

  onMount(async () => {
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
    <a
      href={`/container/${c.id}?name=${encodeURIComponent(c.name)}`}
      class="text-lg font-bold text-orange-400 hover:underline"
    >
      <div class="__container mb-4 flex flex-col gap-2 p-2">
        <h3>{c.name}</h3>
        <p class="text-sm text-white"></p>
        <p>{c.description}</p>
        <h4>Líderes:</h4>
        <div class="lideres" style="display: flex;">
          <p class="lideres__container">{c.leader_1}</p>
          <p class="lideres__container">{c.leader_2}</p>
          <p class="lideres__container">{c.leader_3}</p>
        </div>
      </div>
    </a>
    {#if isAdmin}
      <h4>
        <a
          href={`/container/${c.id}?name=${encodeURIComponent(c.name)}`}
          class="text-lg font-bold text-orange-400 hover:underline"
          >{c.ip}:{c.port}</a
        >
      </h4>
      <div class="flex gap-2">
        <button on:click={() => editContainer(c)}>Editar</button>
        <button on:click={() => deleteContainer(c.id)}>Deletar</button>
      </div>
    {/if}
  {/each}
</div>

{#if isAdmin}
  <div
    class="mb-4"
    style="display: flex; gap: 0.3rem; flex-direction: column; width: 20rem; height: 15rem; border-radius: 10px; padding: 0.3rem"
  >
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
{/if}

<style>
  .__body__main_menu__ {
    display: flex;
    justify-content: center;
    align-items: center;
    background-color: #2a2a2a;
    height: 92vh;
  }

  .__container {
    background-color: #151d25;
    border: 1px solid #5e5e5e;
    border-radius: 10px;
    width: 20rem;
    height: 15rem;
    padding: 0.3rem;
    color: white;
    gap: 0.1px;
  }

  .mb-4 {
    background-color: #151d25;
    border: 1px solid #5e5e5e;
  }

  input {
    height: 1.5rem;
    border-radius: 10rem;
    border: none;
    padding-left: 10px;
    background-color: #343a40;
    color: white;
  }
  input::placeholder {
    color: #b0b0b0;
  }
  button {
    height: 1.5rem;
    border-radius: 10rem;
    border: none;
    background-color: orange;
    color: rgb(0, 0, 0);
    transition: all 0.2s ease-in-out;
  }
  button:hover {
    cursor: pointer;
    background-color: rgb(255, 177, 33);
  }

  a {
    color: white;
    text-decoration: none;
    transition: all 0.2s ease-in-out;
  }
  a:hover {
    color: orange;
  }

  .lideres {
    gap: 0.3rem;
  }

  .lideres__container {
    display: flex;
    justify-content: center;
    width: 4rem;
    background-color: #373737;
    border-radius: 35%;
  }
</style>
