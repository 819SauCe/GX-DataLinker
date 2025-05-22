<script>
<<<<<<< HEAD
  import { onMount } from 'svelte';

  let isOpen = false;
  let name = '';

  onMount(() => {
    name = localStorage.getItem('username');
=======
  import { onMount } from "svelte";

  let isOpen = false;
  let name = "";
  let avatar = "/avatars/no-user.webp";
  let currentIcon = "/favicon.png";

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
        const res = await fetch(`https://api.iagx.com.br/perfil/${name}`);
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
    observer.observe(document.documentElement, { attributes: true, attributeFilter: ["class"] });
>>>>>>> master
  });
</script>

<nav class="navbar navbar-expand-lg navbar-dark bg-dark px-3">
  <a class="navbar-brand" href="/">
<<<<<<< HEAD
    <img src="/favicon.png" alt="Logo" width="30" height="30" class="d-inline-block align-top me-2">
    IAGX
  </a>

  <button class="navbar-toggler" type="button" on:click={() => isOpen = !isOpen}>
=======
    <img
      src={currentIcon}
      alt="Logo"
      width="30"
      height="30"
      class="d-inline-block align-top me-2"
    />
    IAGX
  </a>

  <button
    class="navbar-toggler"
    type="button"
    on:click={() => (isOpen = !isOpen)}
  >
>>>>>>> master
    <span class="navbar-toggler-icon"></span>
  </button>

  <div class={"collapse navbar-collapse" + (isOpen ? " show" : "")}>
    <ul class="navbar-nav me-auto mb-2 mb-lg-0">
      {#if name}
        <li class="nav-item">
<<<<<<< HEAD
          <a class="nav-link active" aria-current="page" href={`/perfil/${name}`}>Perfil</a>
=======
          <a
            class="nav-link active"
            aria-current="page"
            href={`/perfil/${name}`}>Perfil</a
          >
>>>>>>> master
        </li>
      {/if}
      <li class="nav-item">
        <a class="nav-link" href="/guia">Guia</a>
      </li>
      <li class="nav-item">
        <a class="nav-link" href="/dev">Dev</a>
      </li>
      <li class="nav-item">
        <a class="nav-link" href="/contato">Contato</a>
      </li>
    </ul>

    <form class="d-flex">
<<<<<<< HEAD
      <input class="form-control me-2" type="search" placeholder="Buscar" aria-label="Search">
      <button class="btn btn-outline-success" type="submit">Buscar</button>
=======
      <a href={`/perfil/${name}`}
        ><img class="user_img" src={avatar} alt="User img" /></a
      >
      <a href={`/perfil/${name}`} style="font-size: 1.1rem;">Olá {name}</a>
>>>>>>> master
    </form>
  </div>
</nav>

<style>
  button {
    border: 1px solid var(--accent-color);
    color: var(--accent-color);
  }

  button:hover {
    background-color: var(--accent-color);
    border: 1px solid var(--borde-color2);
  }
<<<<<<< HEAD
=======

  .user_img {
    width: 2rem;
    border-radius: 50%;
    margin-right: 1rem;
  }

  a {
    color: white;
    text-decoration: none;
}
>>>>>>> master
</style>
