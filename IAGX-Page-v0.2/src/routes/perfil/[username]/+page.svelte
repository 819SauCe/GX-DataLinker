<script>
<<<<<<< HEAD
  import { page } from '$app/stores';
  import { onMount } from 'svelte';

  let username = '';
  let email = '';
  let avatar = '';

  let theme = (() => {
    const match = document.cookie.match(/theme=([^;]+)/);
    return match ? match[1] : 'default';
=======
  import { page } from "$app/stores";
  import { onMount, tick } from "svelte";

  let username = "";
  let email = "";
  let avatar = "";
  let isOwnProfile = false;
  let online = false;
  const token = localStorage.getItem("token");

  let theme = (() => {
    const match = document.cookie.match(/theme=([^;]+)/);
    return match ? match[1] : "default";
>>>>>>> master
  })();

  $: userParam = $page.params.username;

<<<<<<< HEAD
  onMount(async () => {
    setTheme(theme);

=======
  async function pingUser() {
    await fetch(`https://api.iagx.com.br/ping`, {
      method: "GET",
      headers: { Authorization: `Bearer ${token}` },
    });
  }

  async function checkOnlineStatus() {
>>>>>>> master
    const res = await fetch(`https://api.iagx.com.br/perfil/${userParam}`);
    if (!res.ok) {
      console.error("Erro na API:", res.status);
      return;
    }

    try {
      const data = await res.json();
      username = data.username;
      email = data.email;
<<<<<<< HEAD
      avatar = data.avatar_url || '/avatars/no-user.webp';
    } catch (err) {
      console.error("Erro ao fazer parse do JSON:", err);
    }
  });

  function setTheme(value) {
    document.documentElement.classList.remove('theme-default', 'theme-globalx', 'theme-dark');
=======
      avatar = data.avatar_url
        ? `https://api.iagx.com.br${data.avatar_url}`
        : "/avatars/no-user.webp";

      if (data.last_seen) {
        const lastSeen = new Date(data.last_seen);
        const diff = (Date.now() - lastSeen.getTime()) / 1000;
        online = diff < 60;
      } else {
        online = false;
      }
    } catch (err) {
      console.error("Erro ao fazer parse do JSON:", err);
    }
  }

  onMount(async () => {
    setTheme(theme);
    const loggedUser = localStorage.getItem("username");
    isOwnProfile = loggedUser === userParam;

    if (isOwnProfile) {
      await pingUser();
      await checkOnlineStatus();
      setInterval(pingUser, 30000);
    } else {
      await checkOnlineStatus();
      setInterval(checkOnlineStatus, 60000);
    }

    await tick();
    const displayedName = document.querySelector("h2.mt-3")?.textContent?.trim();});

  function setTheme(value) {
    document.documentElement.classList.remove(
      "theme-default",
      "theme-globalx",
      "theme-dark"
    );
>>>>>>> master
    document.documentElement.classList.add(`theme-${value}`);
    document.cookie = `theme=${value}; path=/; max-age=31536000`;
    theme = value;
  }
<<<<<<< HEAD
</script>


<main class="perfil-background">
  <div class="perfil-container-user" style="display: flex; flex-direction:row">
    <div class="perfil-info text-center">
      <img class="perfil-avatar mb-4" src={avatar} alt="Avatar" />
      <h1 class="display-4 mb-2">{username}</h1>
      <p class="lead">{email}</p>
    </div>

    <div class="configuracao-user" style="margin-left: 1rem">
      <h1>Temas:</h1>
      <div class="form-check">
        <label>
          <input type="radio" name="theme" value="default" bind:group={theme} on:change={() => setTheme('default')} />
          Default
        </label>
      </div>
      <div class="form-check">
        <label>
          <input type="radio" name="theme" value="globalx" bind:group={theme} on:change={() => setTheme('globalx')} />
          Global-X
        </label>
      </div>
      <div class="form-check">
        <label>
          <input type="radio" name="theme" value="dark" bind:group={theme} on:change={() => setTheme('dark')} />
          Dark
        </label>
      </div>
=======

  async function handleImageUpload(event) {
    const file = event.target.files[0];
    if (!file) return;

    if (!file.type.startsWith("image/")) {
      alert("Arquivo inválido. Envie uma imagem.");
      return;
    }

    const maxSizeMB = 2;
    if (file.size > maxSizeMB * 1024 * 1024) {
      alert(`Imagem muito grande. Máximo permitido: ${maxSizeMB}MB.`);
      return;
    }

    const extension = file.name.split(".").pop();
    const newFileName = `${userParam}.${extension}`;

    const formData = new FormData();
    formData.append("image", file, newFileName);

    const res = await fetch(`${import.meta.env.VITE_API_URL}/upload-avatar`, {
      method: "POST",
      headers: {
        Authorization: `Bearer ${localStorage.getItem("token")}`,
      },
      body: formData,
    });

    if (res.ok) {
      alert("Imagem enviada com sucesso!");
      avatar = `https://api.iagx.com.br/avatars/${newFileName}?t=${Date.now()}`;
    } else {
      alert("Erro ao enviar imagem.");
    }
  }
</script>

<main class="perfil-background">
  <div class="container perfil-container-user py-4">
    <div class="row">
      <div class="col-md-4 text-center perfil-info mb-4 mb-md-0">
        <div class="avatar-wrapper position-relative d-inline-block">
          <img src={avatar} class="perfil-avatar img-fluid mb-3" alt="Avatar" />
          {#if !isOwnProfile}
            <span
              class="position-absolute top-0 end-0 translate-middle p-2 bg-success border border-light rounded-circle status-indicator"
              title={online ? "Online" : "Offline"}
            />
          {/if}
        </div>

        <div class="mt-3">
          {#if isOwnProfile}
            <label class="btn btn-outline-secondary w-100">
              Trocar foto
              <input type="file" class="d-none" on:change={handleImageUpload} />
            </label>
          {/if}
        </div>

        <h2 class="mt-3">{username}</h2>
        <p style="color: var(--text-color2);">{email}</p>
      </div>

      {#if isOwnProfile}
        <div class="col-md-8">
          <div class="config-box p-4 rounded">
            <h3 class="mb-4">Configurações</h3>

            <div class="mb-4">
              <h5>Tema</h5>
              <div class="form-check form-check-inline">
                <input
                  class="form-check-input"
                  type="radio"
                  name="theme"
                  value="default"
                  bind:group={theme}
                  on:change={() => setTheme("default")}
                />
                <label class="form-check-label">Default</label>
              </div>
              <div class="form-check form-check-inline">
                <input
                  class="form-check-input"
                  type="radio"
                  name="theme"
                  value="globalx"
                  bind:group={theme}
                  on:change={() => setTheme("globalx")}
                />
                <label class="form-check-label">Global-X</label>
              </div>
              <div class="form-check form-check-inline">
                <input
                  class="form-check-input"
                  type="radio"
                  name="theme"
                  value="dark"
                  bind:group={theme}
                  on:change={() => setTheme("dark")}
                />
                <label class="form-check-label">Dark</label>
              </div>
            </div>

            <div class="mb-4">
              <h5>Alterar E-mail</h5>
              <input
                type="email"
                placeholder="Email atual"
                class="form-control mb-2"
              />
              <input
                type="email"
                placeholder="Novo e-mail"
                class="form-control"
              />
            </div>

            <div class="mb-4">
              <h5>Alterar Senha</h5>
              <input
                type="password"
                placeholder="Senha atual"
                class="form-control mb-2"
              />
              <input
                type="password"
                placeholder="Nova senha"
                class="form-control mb-2"
              />
              <input
                type="password"
                placeholder="Confirmar nova senha"
                class="form-control"
              />
            </div>

            <button class="btn btn-primary">Salvar alterações</button>
          </div>
        </div>
      {/if}
>>>>>>> master
    </div>
  </div>
</main>

<style>
  .perfil-background {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 100%;
<<<<<<< HEAD
    min-height: 100vh;
    background-color: var(--bg-color);
    font-family: var(--font);
    padding: 5rem 1rem;
    color: var(--text-color2);
  }

  .perfil-container-user {
    width: 90rem;
    height: 30rem;
    padding: 1rem;
    background-color: var(--reverse-color);
    border: 1px solid var(--border-color);
    border-radius: 1rem;
  }

  .perfil-info {
    border-right: 1px solid var(--border-color);
    padding-right: 1rem;
  }

  .perfil-avatar {
    width: 20rem;
    height: 20rem;
    border-radius: 50%;
  }

  input[type="radio"] {
    margin-right: 0.5rem;
    accent-color: var(--accent-color);
=======
    height: 60rem;
    background-color: var(--bg-color);
  }

  .perfil-container-user {
    width: 100%;
    height: auto;
  }

  .perfil-avatar {
    width: 100%;
    max-width: 200px;
    height: auto;
    border-radius: 50%;
  }

  .status-indicator[title="Offline"] {
    background-color: gray !important;
  }

  .avatar-wrapper {
    width: 200px;
    height: 200px;
  }

  .perfil-container-user {
    background-color: var(--reverse-color);
    border: 1px solid var(--border-color);
    border-radius: 1rem;
    color: var(--text-color2);
  }

  .config-box {
    background-color: var(--reverse-color);
    color: var(--text-color2);
  }

  input:focus,
  textarea:focus {
    box-shadow: 0 0 0 0.2rem
    var(--accent-color-transparent, rgba(128, 0, 255, 0.25));
    border-color: var(--accent-color);
  }

  .btn {
    background-color: var(--accent-color);
    color: var(--text-color2);
>>>>>>> master
  }
</style>
