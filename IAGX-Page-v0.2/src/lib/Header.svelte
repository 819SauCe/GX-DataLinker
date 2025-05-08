<script>
    let isOpen = false;
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
        <li class="nav-item">
          <a class="nav-link active" aria-current="page" href="/">Início</a>
        </li>
        <li class="nav-item">
          <a class="nav-link" href="/sobre">Como usar ?</a>
        </li>
        <li class="nav-item">
          <a class="nav-link" href="/servicos">Dev</a>
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
      border: 1px solid orange;
      color: orange;
    }
    button:hover {
      background-color: orange;
      border: 1px solid rgb(163, 106, 0);
    }
  </style>