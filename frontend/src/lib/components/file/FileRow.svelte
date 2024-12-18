<script lang="ts">
    import DownloadIcon from "$lib/assets/icons/DownloadIcon.svelte";
    import FilePenIcon from "$lib/assets/icons/FilePenIcon.svelte";
    import ShareNodesIcon from "$lib/assets/icons/ShareNodesIcon.svelte";
    import TrashBinIcon from "$lib/assets/icons/TrashBinIcon.svelte";
    import { File, Files } from "$lib/file";
    import {
        getModalStore,
        getToastStore,
        type ModalSettings
    } from "@skeletonlabs/skeleton";
    import Clipboard from "./Clipboard.svelte";
    import FileShare from "./FileShare.svelte";
    import FileInfo from "./FileInfo.svelte";

    const rowClasses =
        "flex flex-col justify-center items-center xl:w-20 sm:w-16";

    const modalStore = getModalStore();
    const toastStore = getToastStore();
    const files = Files.getInstance();

    type Props = {
        file: File;
        refresh: () => void;
    };

    let { file, refresh }: Props = $props();

    function triggerDelete() {
        function response(r: boolean) {
            if (r) {
                files.deleteFile(file.id).then((res) => {
                    if (res.isOk()) {
                        toastStore.trigger({
                            message: `Deleted file "${res.value.data.name}"`,
                            timeout: 2000,
                            background: "variant-filled-success"
                        });
                    } else {
                        toastStore.trigger({
                            message:
                                "Failed to delete file: " +
                                res.error.toString(),
                            timeout: 2000,
                            background: "variant-filled-error"
                        });
                    }
                    refresh();
                });
            }
        }

        const modal: ModalSettings = {
            type: "confirm",
            title: "Confirm deletion",
            body: `Are you sure you want to delete the file "${file.data.name}"?`,
            buttonTextConfirm: "Delete",
            response
        };

        modalStore.trigger(modal);
    }

    function triggerInfo() {
        modalStore.trigger({
            type: "component",
            component: { ref: FileInfo, props: { file, refresh } }
        });
    }

    function triggerShare() {
        modalStore.trigger({
            type: "component",
            component: { ref: FileShare, props: { file, refresh } }
        });
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
            <Clipboard content={file.data.checksum256} />
            <p class="sm:block hidden">Checksum</p>
        </div>

        <div class={rowClasses}>
            <button class="btn-icon variant-outline" onclick={triggerInfo}>
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
