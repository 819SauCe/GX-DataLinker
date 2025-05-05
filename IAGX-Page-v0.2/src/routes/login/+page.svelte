<script>
  let isLogin = true;
  let loginEmail = "", loginPassword = "";
  let regName = "", regEmail = "", regPassword = "";

  const handleLogin = async () => {
    const res = await fetch('http://localhost:3000/login', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ email: loginEmail, password: loginPassword })
    });
    const data = await res.json();
    console.log(data);

    if (data.message === "Login bem-sucedido") {
      localStorage.setItem('username', data.username);
      localStorage.setItem('token', 'token123'); // token fixo, ajustar depois com JWT
      window.location.href = "/";
    }
  };

  const handleRegister = async () => {
    const res = await fetch('http://localhost:3000/register', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ username: regName, email: regEmail, password: regPassword })
    });
    const data = await res.json();
    console.log(data);
  };
</script>

<link rel="stylesheet" href="https://cdn.jsdelivr.net/npm/bootstrap@5.3.3/dist/css/bootstrap.min.css">

<div class="container mt-5">
  <div class="row justify-content-center">
    <div class="col-md-6">
      <ul class="nav nav-tabs mb-3">
        <li class="nav-item">
          <a href="#" class="nav-link" class:active={isLogin} on:click|preventDefault={() => isLogin = true}>Login</a>
        </li>
        <li class="nav-item">
          <a href="#" class="nav-link" class:active={!isLogin} on:click|preventDefault={() => isLogin = false}>Registro</a>
        </li>
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
