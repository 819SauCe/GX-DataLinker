<script>
  import { page } from "$app/stores";
  import { onMount, tick } from "svelte";
  import { marked } from "marked";
  marked.setOptions({ sanitize: true });

  let messages = [];
  let message = "";
  let history = [];
  let historyIndex = -1;
  let id = "",
    name = "",
    ip = "",
    port = "";
  let chatBox;

  $: id = $page.params.id;
  $: name = $page.url.searchParams.get("name");

  const HISTORY_KEY = `chat_history_${id}`;

  onMount(async () => {
    const token = localStorage.getItem("token");
    if (!token) {
      window.location.href = "/login";
      return;
    }

    const saved = localStorage.getItem(`chat_${id}`);
    if (saved) messages = JSON.parse(saved);
    const savedHistory = localStorage.getItem(HISTORY_KEY);
    if (savedHistory) history = JSON.parse(savedHistory);

    fetchContainer();
  });

  const fetchContainer = async () => {
    const token = localStorage.getItem("token");
    const res = await fetch(
      `${import.meta.env.VITE_API_URL}/containers/${id}`,
      {
        headers: {
          Authorization: `Bearer ${token}`,
          "Content-Type": "application/json",
        },
      },
    );
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
    history.unshift(message);
    localStorage.setItem(HISTORY_KEY, JSON.stringify(history));
    historyIndex = -1;
    const textoEnviado = message;
    messages = [...messages, { type: "user", text: textoEnviado }];
    message = "";
    await tick();

    try {
      /* const res = await fetch(
        `https://api.iagx.com.br/api/estoque/api/gerar_relatorio`,
        {
          method: "POST",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify({ user_message: textoEnviado }),
        }
      ); */

      let rota = "";
      if (name.toLowerCase().includes("estoque")) {
        rota = "estoque";
      } else if (
        name.toLowerCase().includes("ordens") ||
        name.toLowerCase().includes("ordem")
      ) {
        rota = "ordcompra";
      } else {
        rota = "estoque"; // padrão de fallback
      }

      const res = await fetch(
        `https://api.iagx.com.br/api/${rota}/gerar_relatorio`,
        {
          method: "POST",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify({ user_message: textoEnviado }),
        },
      );

      const data = await res.json();
      messages = [
        ...messages,
        {
          type: "server",
          text:
            data.resposta ||
            data.relatorio ||
            data.error ||
            "Resposta inesperada.",
        },
      ];
    } catch (err) {
      messages = [
        ...messages,
        { type: "server", text: "❌ Erro na comunicação com o servidor." },
      ];
    }
  };

  // lida com ↑ e ↓ no input
  function handleKey(event) {
    if (event.key === "ArrowUp") {
      event.preventDefault();
      if (historyIndex < history.length - 1) {
        historyIndex += 1;
        message = history[historyIndex];
      }
    } else if (event.key === "ArrowDown") {
      event.preventDefault();
      if (historyIndex > 0) {
        historyIndex -= 1;
        message = history[historyIndex];
      } else {
        historyIndex = -1;
        message = "";
      }
    }
  }

  $: {
    if (chatBox) chatBox.scrollTop = chatBox.scrollHeight;
    localStorage.setItem(`chat_${id}`, JSON.stringify(messages));
  }
</script>

<div class="__body__main_chat__">
  
  <div class="__chat__" bind:this={chatBox} style="overflow-y: scroll; height: 35rem;">
  <h1 style="text-align: center; font-size: 2rem; color:var(--text-color); padding-top:1rem;">{name}</h1>
  <hr style="color: white;" />

    <!--Historico de mensagems-->
    {#each messages as msg}
      <div
        class:user-message={msg.type === "user"}
        class:server-message={msg.type === "server"}
        class:info-message={msg.type === "info"}
        class="chat-message"
      >
        {@html marked(msg.text)}
      </div>
    {/each}

  </div>

  <!--Chat de texto-->
  <hr style="color: white;"/>
  <form on:submit|preventDefault={sendMessage} style="max-width:90rem; margin:auto; display: flex;">
    <input type="text" class="form-control" placeholder="Hello, World!" bind:value={message} on:keydown={handleKey}/>
    <button class="btn" type="submit" style=""><img src="/Send_msg.png" alt="Send_msg"></button>
  </form>
</div>

<style>
  .__chat__::-webkit-scrollbar {
    display: none;
  }

  .__body__main_chat__ {
    height: 40rem;
    background-color: var(--bg-color);
  }

  .chat-message {
  padding: 0.5rem;
  border-radius: 8px;
  width: 40rem;
  width: fit-content;
  margin: 0.25rem;
}

.btn {
  background: var(--accent-color);
  color: var(--text-color2);
  border:none;
  border-top-left-radius: 0px;
  border-bottom-left-radius: 0px;
}

.btn img {
  width: 1.5rem;
  height:auto;
}

.user-message {
  background-color: #ffeac9;
  color: black;
  text-align: right;
  margin-left: auto;
}

.server-message {
  background-color: var(--accent-color);
  color: black;
  text-align: left;
  margin-left: 0;
}

.info-message {
  background-color: #6c757d;
  color: black;
  text-align: left;
  margin-left: 0;
}

.form-control {
  border-top-left-radius: 10px;
  border-bottom-left-radius: 10px;
  border-top-right-radius: 0;
  border-bottom-right-radius: 0;
}
</style>
