<script lang="ts">
    let { children } = $props();
</script>

<div
    class="container sm:bg-transparent sm:py-4 mx-auto bg-surface-800 size-full flex justify-center items-center"
>
    {@render children?.()}
</div>
