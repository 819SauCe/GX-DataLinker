<script>
    let actualPage = "IAGX";
  </script>
  
  <style>
    header {
      background: #1e1e1e;
      height: 3.5rem;
      display: flex;
      align-items: center;
      padding: 0 2rem;
      justify-content: space-between;
      box-shadow: 0 2px 5px rgba(0,0,0,0.4);
    }
  
    .logo {
      color: orange;
      font-size: 1.4rem;
      font-weight: bold;
      text-transform: uppercase;
    }
  
    nav a {
      color: white;
      text-decoration: none;
      margin-left: 1.5rem;
      transition: color 0.2s ease;
    }
  
    nav a:hover {
      color: orange;
    }
  </style>
  
  <header>
    <div class="logo">{actualPage}</div>
    <nav>
      <a href="/">Home</a>
      <a href="/containers">Conexões</a>
      <a href="/sobre">Sobre</a>
    </nav>
  </header>
  