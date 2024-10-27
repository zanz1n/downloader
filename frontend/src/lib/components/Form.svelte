<script lang="ts">
    export let title: string;

    export let onSubmit: ((e: any) => void) | null = null;

    function onSubmitInternal(e: SubmitEvent) {
        if (onSubmit) {
            const formData = new FormData(e.target as HTMLFormElement);
            const data: any = {};

            for (let field of formData) {
                const [key, value] = field;
                data[key] = value;
            }
            onSubmit(data);
        }
    }
</script>

<div
    class="container sm:py-4 sm:bg-transparent bg-surface-800 size-full flex justify-center items-center"
>
    <form
        on:submit|preventDefault={onSubmitInternal}
        class="sm:card items-center flex flex-col gap-6 p-6"
    >
        <h2 class="h2">{title}</h2>

        <slot />

        <div class="flex flex-col justify-center items-center w-full gap-2 py-4">
            <slot name="bottom" />
        </div>
    </form>
</div>
