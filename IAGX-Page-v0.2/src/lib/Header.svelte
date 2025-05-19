<script>
  import { onMount } from 'svelte';

  let isOpen = false;
  let name = '';

  onMount(() => {
    name = localStorage.getItem('username');
  });
</script>

<nav class="navbar navbar-expand-lg navbar-dark bg-dark px-3">
  <a class="navbar-brand" href="/">
    <img src="/favicon.png" alt="Logo" width="30" height="30" class="d-inline-block align-top me-2">
    IAGX
  </a>

  <button class="navbar-toggler" type="button" on:click={() => isOpen = !isOpen}>
    <span class="navbar-toggler-icon"></span>
  </button>

  <div class={"collapse navbar-collapse" + (isOpen ? " show" : "")}>
    <ul class="navbar-nav me-auto mb-2 mb-lg-0">
      {#if name}
        <li class="nav-item">
          <a class="nav-link active" aria-current="page" href={`/perfil/${name}`}>Perfil</a>
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
      <input class="form-control me-2" type="search" placeholder="Buscar" aria-label="Search">
      <button class="btn btn-outline-success" type="submit">Buscar</button>
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
</style>
