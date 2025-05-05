<script>
    import Header from "$lib/Header.svelte";
</script>

<Header />
<slot />

<style>
    :global(body) {
        padding: 0%;
        margin: 0%;
        border: 0%;
    }
</style>
