<script>
  import { page } from "$app/stores";
  import { onMount } from "svelte";
  

  let messages = [];
  let message = "";
  let socket;
  let id = "",
    name = "",
    ip = "",
    port = "";
  let chatBox;

  $: id = $page.params.id;
  $: name = $page.url.searchParams.get("name");

  const fetchContainer = async () => {
    const res = await fetch(`${import.meta.env.VITE_API_URL}/containers/${id}`);
    if (res.ok) {
      const data = await res.json();
      ip = data.ip;
      port = data.port;
      connect();
    }
  };

  const connect = () => {
    socket = new WebSocket(`ws://${ip}:${port}`);
    socket.onmessage = (e) =>
      (messages = [...messages, { type: "server", text: e.data }]);
    socket.onopen = () =>
      (messages = [
        ...messages,
        { type: "info", text: `✅ Conectado a ${name}` },
      ]);
    socket.onclose = () =>
      (messages = [
        ...messages,
        { type: "info", text: `❌ Conexão encerrada` },
      ]);
  };

  const sendMessage = () => {
    if (socket?.readyState === WebSocket.OPEN && message.trim()) {
      messages = [...messages, { type: "user", text: message }];
      socket.send(JSON.stringify({ user_message: message }));
      message = "";
    }
  };

  $: {
    if (chatBox) chatBox.scrollTop = chatBox.scrollHeight;
    localStorage.setItem(`chat_${id}`, JSON.stringify(messages));
  }

  onMount(() => {
    const token = localStorage.getItem("token");
    if (!token) {
      window.location.href = "/login";
      return;
    }
    const saved = localStorage.getItem(`chat_${id}`);
    if (saved) messages = JSON.parse(saved);
    fetchContainer();
  });
</script>

<div
  class="__body__main_chat__"
  style="background-color: #151d25; width: 100%; height: 92vh;"
>
  <h1
    style="text-align: center; font-size: 2rem; color: white; padding-top:1rem;"
  >
    {name}
  </h1>
  <hr style="color: white;" />

  <div
    class="__chat__"
    bind:this={chatBox}
    style="overflow-y: auto; width: 100%; height: 29rem;"
  >
    {#each messages as msg}
      <div
        style="
        color: white;
        padding: 0.5rem;
        text-align: {msg.type === 'user' ? 'right' : 'left'};
        background-color: {msg.type === 'user'
          ? '#0d6efd'
          : msg.type === 'server'
            ? '#198754'
            : '#6c757d'};
        margin: 0.25rem;
        border-radius: 8px;
        max-width: 60%;
        margin-left: {msg.type === 'user' ? 'auto' : '0'};
        margin-right: {msg.type === 'user' ? '0' : 'auto'};
      "
      >
        {msg.text}
      </div>
    {/each}
  </div>

  <hr style="color: white;" />

  <div class="input-group mb-3" style="max-width: 93rem; margin: auto;">
    <input
      type="text"
      class="form-control"
      placeholder="Digite sua mensagem"
      bind:value={message}
      on:keydown={(e) => e.key === "Enter" && sendMessage()}
    />
    <button
      class="btn"
      type="button"
      on:click={sendMessage}
      style="background-color: orange; color: black; border: none;"
      >Enviar</button
    >
  </div>
</div>
