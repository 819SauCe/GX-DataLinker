<script>
    import Header from "$lib/Header.svelte";
    import Foooter from "../lib/Footer.svelte";
</script>

<Header />
<slot />
<Foooter />

<style>
    @import url('https://fonts.googleapis.com/css2?family=Open+Sans:ital,wght@0,300..800;1,300..800&family=Roboto:ital,wght@0,100..900;1,100..900&display=swap');

    :global(body) {
        padding: 0%;
        margin: 0%;
        border: 0%;
        overflow-x: hidden;
    }

    :global(.theme-default) {
    --bg-color: #2a2a2a;
    --reverse-color: #ffff;
    --container-background: #151d25;
    --text-color: #ffffff;
    --text-color2: rgb(0, 0, 0);
    --accent-color: #ffa500;
    --border-color: #5e5e5e;
    --borde-color2: rgb(163, 106, 0);
    --font: "Roboto", sans-serif;
    }

    :global(.theme-globalx) {
        --bg-color: #1c1c1c;
        --reverse-color: #f0f0f0;
        --container-background: #212f3d;
        --text-color: #e6f7ff;
        --text-color2: #000000;
        --border-color: #00bcd4;
        --border-color2: #009292;
        --accent-color: #00ffff;
        --font: "Open Sans", sans-serif;
    }

    :global(.theme-dark) {
        --bg-color: #1e1e1e;
        --reverse-color: #2c2c2c;
        --container-background: #2a2a2a;
        --text-color: #f5f5f5;
        --text-color2: #e0e0e0;
        --accent-color: #bb86fc;
        --border-color: #3c3c3c;
        --border-color2: #8250df;
        --font: "Roboto", sans-serif;
    }

</style>
