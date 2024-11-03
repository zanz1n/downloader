<script lang="ts">
    import type { Snippet } from "svelte";

    type Props = {
        onSubmit: ((e: unknown) => void) | null;
        children?: Snippet;
        bottom?: Snippet;
        title: string;
    };

    let { onSubmit = null, children, bottom, title }: Props = $props();

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

<div
    class="container sm:py-4 sm:bg-transparent bg-surface-800 size-full flex justify-center items-center"
>
    <form
        onsubmit={onSubmitInternal}
        class="sm:card items-center flex flex-col gap-6 p-6"
    >
        <h2 class="h2">{title}</h2>

        {@render children?.()}

        <div
            class="flex flex-col justify-center items-center w-full gap-2 py-4"
        >
            {@render bottom?.()}
        </div>
    </form>
</div>
