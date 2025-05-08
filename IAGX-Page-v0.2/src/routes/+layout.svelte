<script>
    import Header from "$lib/Header.svelte";
    import Foooter from "../lib/Footer.svelte";
</script>

<Header />
<slot />
<Foooter />

<style>
    :global(body) {
        padding: 0%;
        margin: 0%;
        border: 0%;
        overflow-x: hidden;
    }
</style>
