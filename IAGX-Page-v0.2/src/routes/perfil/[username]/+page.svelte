<script>
  import { page } from '$app/stores';
  import { onMount } from 'svelte';

  let username = '';
  let email = '';
  let avatar = '';

  let theme = (() => {
    const match = document.cookie.match(/theme=([^;]+)/);
    return match ? match[1] : 'default';
  })();

  $: userParam = $page.params.username;

  onMount(async () => {
    setTheme(theme);

    const res = await fetch(`https://api.iagx.com.br/perfil/${userParam}`);
    if (!res.ok) {
      console.error("Erro na API:", res.status);
      return;
    }

    try {
      const data = await res.json();
      username = data.username;
      email = data.email;
      avatar = data.avatar_url || '/avatars/no-user.webp';
    } catch (err) {
      console.error("Erro ao fazer parse do JSON:", err);
    }
  });

  function setTheme(value) {
    document.documentElement.classList.remove('theme-default', 'theme-globalx', 'theme-dark');
    document.documentElement.classList.add(`theme-${value}`);
    document.cookie = `theme=${value}; path=/; max-age=31536000`;
    theme = value;
  }
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
    </div>
  </div>
</main>

<style>
  .perfil-background {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 100%;
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
  }
</style>
