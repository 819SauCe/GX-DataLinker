<script>
  import { page } from "$app/stores";
  import { onMount } from "svelte";
  import { marked } from "marked";
  import { tick } from "svelte";
  marked.setOptions({ sanitize: true });

  let messages = [];
  let message = "";
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
      messages = [
        ...messages,
        { type: "info", text: `✅ Conectado a ${name}` },
      ];
    }
  };

  const sendMessage = async () => {
    if (!message.trim()) return;

    messages = [...messages, { type: "user", text: message }];
    await tick();

    try {
      const res = await fetch(`http://${ip}:${port}/api/gerar_relatorio`, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ user_message: message }),
      });
      const data = await res.json();
      messages = [
        ...messages,
        {
          type: "server",
          text: data.relatorio || data.error || "Resposta inesperada.",
        },
      ];
    } catch (err) {
      messages = [
        ...messages,
        { type: "server", text: "❌ Erro na comunicação com o servidor." },
      ];
    } finally {
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
    style="overflow-y: scroll; scrollbar-width: none; -ms-overflow-style: none; width: 100%; height: 29rem;"
  >
    {#each messages as msg}
      <div
        style="
        color: white;
        padding: 0.5rem;
        text-align: {msg.type === 'user' ? 'right' : 'left'};
        background-color: {msg.type === 'user'
          ? '#ffeac9'
          : msg.type === 'server'
            ? 'orange'
            : '#6c757d'};
        color: {msg.type === 'server' ? 'black' : 'black'};
        margin: 0.25rem;
        border-radius: 8px;
        width: fit-content;
        max-width: 40rem;
        margin-left: {msg.type === 'user' ? 'auto' : '0'};
        margin-right: {msg.type === 'user' ? '0' : 'auto'};
      "
      >
        {@html marked(msg.text)}
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

<style>
  .__chat__::-webkit-scrollbar {
    display: none;
  }
</style>
