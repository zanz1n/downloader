<script lang="ts">
    import type { Snippet } from "svelte";

    type Props = {
        onSubmit?: (e: unknown) => void;
        children?: Snippet;
        bottom?: Snippet;
        title?: string;
    };

    let { onSubmit, children, bottom, title }: Props = $props();

    function onSubmitInternal(e: SubmitEvent) {
        e.preventDefault();

        if (onSubmit) {
            const formData = new FormData(e.target as HTMLFormElement);
            const data: Record<string, unknown> = {};

            for (let field of formData) {
                const [key, value] = field;
                data[key] = value;
            }
            onSubmit(data);
        }
    }
</script>

<form
    onsubmit={onSubmitInternal}
    class="sm:card flex flex-col items-center gap-6 p-6"
>
    {#if title}
        <h2 class="h2">{title}</h2>
    {/if}

    {@render children?.()}

    <div class="flex flex-col justify-center items-center w-full gap-2 py-4">
        {@render bottom?.()}
    </div>
</form>
