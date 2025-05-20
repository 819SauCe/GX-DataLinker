<script>
  let isLogin = true;
  let loginEmail = "", loginPassword = "";
  let regName = "", regEmail = "", regPassword = "";

  const handleLogin = async () => {
    const res = await fetch(`${import.meta.env.VITE_API_URL}/login`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ email: loginEmail, password: loginPassword })
  });

  const data = await res.json();
  console.log(data);

  if (res.ok && data.message && data.message.split(".").length === 3) {
    localStorage.setItem('username', data.username);
    localStorage.setItem('token', data.message);
    window.location.href = "/";
  } else {
    alert("Login falhou!");
  }
};

  const handleRegister = async () => {
    const res = await fetch(`${import.meta.env.VITE_API_URL}/register`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ username: regName, email: regEmail, password: regPassword })
    });
    const data = await res.json();
    console.log(data);
  };
</script>

<link rel="stylesheet" href="https://cdn.jsdelivr.net/npm/bootstrap@5.3.3/dist/css/bootstrap.min.css">

<div class="__body__login">
  <div class="container mt-0">
  <div class="row justify-content-center">
    <div class="col-md-6">
      <ul class="nav nav-tabs mb-3">
        <li class="nav-item">
          <a href="#" class="nav-link" class:active={isLogin} on:click|preventDefault={() => isLogin = true}>Login</a>
        </li>
    <!-- <li class="nav-item">
          <a href="#" class="nav-link" class:active={!isLogin} on:click|preventDefault={() => isLogin = false}>Registro</a>
        </li> --> 
      </ul>

      {#if isLogin}
        <form on:submit|preventDefault={handleLogin}>
          <div class="mb-3">
            <label class="form-label">Email</label>
            <input type="email" bind:value={loginEmail} class="form-control" required />
          </div>
          <div class="mb-3">
            <label class="form-label">Senha</label>
            <input type="password" bind:value={loginPassword} class="form-control" required />
          </div>
          <button type="submit" class="btn btn-primary w-100">Entrar</button>
        </form>
      {:else}
        <form on:submit|preventDefault={handleRegister}>
          <div class="mb-3">
            <label class="form-label">Nome</label>
            <input type="text" bind:value={regName} class="form-control" required />
          </div>
          <div class="mb-3">
            <label class="form-label">Email</label>
            <input type="email" bind:value={regEmail} class="form-control" required />
          </div>
          <div class="mb-3">
            <label class="form-label">Senha</label>
            <input type="password" bind:value={regPassword} class="form-control" required />
          </div>
          <button type="submit" class="btn btn-success w-100">Registrar</button>
        </form>
      {/if}
    </div>
  </div>
</div>
</div>

<style>
  .__body__login {
    width: 100%;
    height: 92vh;
    background-color: #2a2a2a;
    color: white;
  }

  button {
    color: white;
    background-color: orange;
    border: 1px solid rgb(116, 116, 116);
  }

  button:hover {
    color: white;
    background-color: rgb(219, 142, 0);
    border: 1px solid rgb(116, 116, 116);
  }

  .nav-link {
    color: orange;
    margin-top: 5rem;
  }

  .nav-link:hover {
    color: rgb(219, 142, 0);
  }

</style>