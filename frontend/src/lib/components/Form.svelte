<script lang="ts">
    export let title: string;
</script>

<div class="container sm:bg-transparent bg-surface-800 size-full flex justify-center items-center">
    <form class="sm:card items-center flex flex-col gap-6 p-6">
        <h2 class="h2">{title}</h2>

        <slot />

        <div class="flex flex-col justify-center items-center w-full gap-2 py-4">
            <slot name="bottom" />
        </div>
    </form>
</div>
