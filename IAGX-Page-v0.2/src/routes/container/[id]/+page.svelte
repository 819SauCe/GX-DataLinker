<script>
    import { page } from '$app/stores';
    import { onMount } from 'svelte';

    onMount(() => {
    const token = localStorage.getItem('token');
    if (!token) {
        window.location.href = "/login";
        return;
    }
    fetchContainer();
    });

  
    let messages = [];
    let message = '';
    let socket;
  
    let id = '', name = '', ip = '', port = '';
  
    $: id = $page.params.id;
    $: name = $page.url.searchParams.get('name');
  
    const fetchContainer = async () => {
      const res = await fetch(`http://localhost:3000/containers/${id}`);
      if (res.ok) {
        const data = await res.json();
        ip = data.ip;
        port = data.port;
        connect();
      }
    };
  
    const connect = () => {
      socket = new WebSocket(`ws://${ip}:${port}`);
      socket.onmessage = (e) => messages = [...messages, e.data];
      socket.onopen = () => messages = [...messages, `✅ Conectado a ${name}`];
      socket.onclose = () => messages = [...messages, `❌ Conexão encerrada`];
    };
  
    const sendMessage = () => {
      if (socket?.readyState === WebSocket.OPEN && message.trim()) {
        socket.send(message);
        message = '';
      }
    };
  
    onMount(fetchContainer);
  </script>
  
  <style>
    .container {
      max-width: 600px;
      margin: auto;
      padding-top: 2rem;
      font-family: 'Segoe UI', sans-serif;
    }
  
    .chat-box {
      background: #1e1e1e;
      color: white;
      border-radius: 8px;
      padding: 1rem;
      height: 300px;
      overflow-y: auto;
      margin-bottom: 1rem;
    }
  
    .input-box {
      display: flex;
      gap: 0.5rem;
    }
  
    .input-box input {
      flex: 1;
      padding: 0.5rem;
      border-radius: 4px;
      border: 1px solid #ccc;
    }
  
    .input-box button {
      background: orange;
      border: none;
      padding: 0.5rem 1rem;
      border-radius: 4px;
      color: black;
      font-weight: bold;
      cursor: pointer;
    }
  
    .input-box button:hover {
      background: #ff9900;
    }
  </style>
  
  <div class="container">
    <h2>Conexão: {name}</h2>
    <div class="chat-box">
      {#each messages as msg}
        <div>{msg}</div>
      {/each}
    </div>
    <div class="input-box">
      <input bind:value={message} placeholder="Digite sua mensagem..." on:keydown={(e) => e.key === 'Enter' && sendMessage()} />
      <button on:click={sendMessage}>Enviar</button>
    </div>
  </div>