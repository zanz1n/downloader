<script lang="ts">
    import ClipboardCheckIcon from "$lib/assets/icons/ClipboardCheckIcon.svelte";
    import { clipboard, getToastStore } from "@skeletonlabs/skeleton";

    const toastStore = getToastStore();

    type Props = {
        classes?: string;
        activeClasses?: string;
        content: string;
    };

    let {
        classes = "btn-icon variant-outline",
        activeClasses = "btn-icon hover:cursor-default",
        content
    }: Props = $props();

    let copied = $state(false);

    function onClipboardClick() {
        toastStore.trigger({
            message: "Copied to clipboard",
            timeout: 2000
        });
        setTimeout(() => {
            copied = true;
            setTimeout(() => {
                copied = false;
            }, 2000);
        }, 100);
    }
</script>

{#if copied}
    <button class={activeClasses} disabled>
        <ClipboardCheckIcon />
    </button>
{:else}
    <button class={classes} use:clipboard={content} onclick={onClipboardClick}>
        <ClipboardCheckIcon />
    </button>
{/if}
