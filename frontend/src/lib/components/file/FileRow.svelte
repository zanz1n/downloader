<script lang="ts">
    import ClipboardCheckIcon from "$lib/assets/icons/ClipboardCheckIcon.svelte";
    import ClipboardIcon from "$lib/assets/icons/ClipboardIcon.svelte";
    import DownloadIcon from "$lib/assets/icons/DownloadIcon.svelte";
    import FilePenIcon from "$lib/assets/icons/FilePenIcon.svelte";
    import ShareNodesIcon from "$lib/assets/icons/ShareNodesIcon.svelte";
    import TrashBinIcon from "$lib/assets/icons/TrashBinIcon.svelte";
    import { File } from "$lib/file";
    import { clipboard, getToastStore } from "@skeletonlabs/skeleton";

    const toastStore = getToastStore();

    const rowClasses =
        "flex flex-col justify-center items-center xl:w-20 sm:w-16";

    type Props = {
        file: File;
        triggerDelete: () => void;
        triggerOpenInfo: () => void;
        triggerShare: () => void;
    };

    let { file, triggerDelete, triggerOpenInfo, triggerShare }: Props =
        $props();

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

<div
    class="card w-full flex flex-row justify-between items-center sm:gap-8 gap-6 sm:p-4 p-3"
>
    <div class="flex flex-col items-start gap-3">
        <div class="inline-block min-w-0">
            <p class="block sm:text-2xl text-xl">
                {file.data.name}
            </p>
        </div>
        <code class="sm:text-sm code">{file.formatSize()}</code>
    </div>

    <div class="flex flex-row sm:gap-5 gap-3">
        <div class={rowClasses + " lg:flex hidden"}>
            {#if copied}
                <button class="btn-icon hover:cursor-default" disabled>
                    <ClipboardCheckIcon />
                </button>
            {:else}
                <button
                    use:clipboard={file.data.checksum256}
                    onclick={onClipboardClick}
                    class="btn-icon variant-outline"
                >
                    <ClipboardIcon />
                </button>
            {/if}
            <p class="sm:block hidden">Checksum</p>
        </div>

        <div class={rowClasses}>
            <button class="btn-icon variant-outline" onclick={triggerOpenInfo}>
                <FilePenIcon />
            </button>
            <p class="sm:block hidden">Info</p>
        </div>

        <div class={rowClasses + " sm:flex hidden"}>
            <a
                class="btn-icon variant-ghost-success"
                href={file.getDownloadUrl().unwrapOr("")}
            >
                <DownloadIcon />
            </a>
            <p class="sm:block hidden">Download</p>
        </div>

        <div class={rowClasses}>
            <button
                class="btn-icon variant-ghost-tertiary"
                onclick={triggerShare}
            >
                <ShareNodesIcon />
            </button>
            <p class="sm:block hidden">Share</p>
        </div>

        <div class={rowClasses + " sm:flex hidden"}>
            <button
                class="btn-icon variant-ghost-error"
                onclick={triggerDelete}
            >
                <TrashBinIcon />
            </button>
            <p class="sm:block hidden">Delete</p>
        </div>
    </div>
</div>
