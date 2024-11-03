<script lang="ts">
    let { children } = $props();
</script>

<div class="container h-full mx-auto flex justify-center items-center">
    {@render children?.()}
</div>
