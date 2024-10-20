<script lang="ts">
    import { page } from "$app/stores";

    let message = $page.error?.message ?? "Unknown";
    let status = $page.status;
</script>

<svelte:head>
    <title>{status} {message}</title>
</svelte:head>

<div class="size-full flex flex-col gap-5 items-center justify-center">
    <h1 class="h1 text-9xl">{status}</h1>
    <h1 class="h1">{message}</h1>
</div>
