<script>
    import { onMount } from 'svelte';
  
    let containers = [];
    let name = '', description = '', leader1 = '', leader2 = '', leader3 = '', ip = '', port = '';
    let editingId = null;
  
    const fetchContainers = async (token) => {
      try {
        const res = await fetch('http://localhost:3000/containers', {
          headers: token ? { 'Authorization': `Bearer ${token}` } : {}
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
      const token = localStorage.getItem('token');
      const data = { name, description, leader_1: leader1, leader_2: leader2, leader_3: leader3, ip, port };
      const headers = {
        'Content-Type': 'application/json',
        ...(token && { 'Authorization': `Bearer ${token}` })
      };
  
      if (editingId) {
        await fetch(`http://localhost:3000/containers/${editingId}`, {
          method: 'PUT',
          headers,
          body: JSON.stringify(data)
        });
      } else {
        await fetch('http://localhost:3000/containers', {
          method: 'POST',
          headers,
          body: JSON.stringify(data)
        });
      }
  
      name = ''; description = ''; leader1 = ''; leader2 = ''; leader3 = ''; ip = ''; port = '';
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
      const token = localStorage.getItem('token');
      await fetch(`http://localhost:3000/containers/${id}`, {
        method: 'DELETE',
        headers: token ? { 'Authorization': `Bearer ${token}` } : {}
      });
      await fetchContainers(token);
    };
  
    onMount(async () => {
      const token = localStorage.getItem('token');
      if (!token) {
        window.location.href = "/login";
        return;
      }
      await fetchContainers(token);
    });
  </script>
  
  <form on:submit|preventDefault={saveContainer} class="mb-4">
    <input bind:value={name} placeholder="Nome" required />
    <input bind:value={description} placeholder="Descrição" />
    <input bind:value={leader1} placeholder="Líder 1" />
    <input bind:value={leader2} placeholder="Líder 2" />
    <input bind:value={leader3} placeholder="Líder 3" />
    <input bind:value={ip} placeholder="IP" required />
    <input bind:value={port} placeholder="Porta" required />
    <button type="submit">{editingId ? 'Atualizar' : 'Adicionar'}</button>
  </form>
  
  <ul>
    {#each containers as c}
      <li>
        <p>id: {c.id}</p>
        <a href={`/container/${c.id}?name=${encodeURIComponent(c.name)}`}><strong>{c.name}</strong></a>
        <br />
        Líderes: {c.leader_1}, {c.leader_2}, {c.leader_3}
        <br />
        <button on:click={() => editContainer(c)}>Editar</button>
        <button on:click={() => deleteContainer(c.id)}>Deletar</button>
      </li>
    {/each}
  </ul>
  