<script lang="ts">
    import CloseIcon from "$lib/assets/icons/CloseIcon.svelte";
    import { Files, type File as Object } from "$lib/file";
    import {
        FileDropzone,
        getModalStore,
        getToastStore
    } from "@skeletonlabs/skeleton";
    import Clipboard from "./Clipboard.svelte";
    import CloudUploadIcon from "$lib/assets/icons/CloudUploadIcon.svelte";
    import { AppError } from "$lib/error";
    import { Err, None, Option } from "ts-results-es";

    const files = Files.getInstance();

    const classes = "variant-filled-secondary";
    const activeClasses = "";

    const modalStore = getModalStore();
    const toastStore = getToastStore();

    type Props = {
        file: Object;
        refresh: () => void;
    };

    let { file, refresh }: Props = $props();

    type Progress = {
        current: number;
        total: number;
    };

    let valueName = $state(file.data.name);
    let valueMimeType = $state(file.data.mimeType);
    let uploading = $state<Progress>();

    let filesList: FileList | undefined = $state();
    let uploadedFile = $derived.by(() => {
        if (!!filesList && filesList.length > 0) {
            return filesList[0];
        }
        return null;
    });

    let changed = $derived(
        valueName != file.data.name ||
            valueMimeType != file.data.mimeType ||
            !!uploadedFile
    );

    function progressString(data: Progress) {
        const n = Math.round((data.current / data.total) * 100);
        return n.toString() + "%";
    }

    function clearAfter() {
        filesList = undefined;
        uploading = undefined;
    }

    function makeProgress(current: number, total: number) {
        uploading = { current, total };
    }

    async function handleUpdate() {
        if (!changed) return;

        let res;
        if (uploadedFile) {
            res = await files.updateFileData(
                file.id,
                valueName,
                uploadedFile,
                valueMimeType,
                makeProgress
            );
        } else {
            res = await files.updateFileInfo(file.id, valueName, valueMimeType);
        }

        modalStore.close();
        refresh();
        clearAfter();

        if (res.isOk()) {
            toastStore.trigger({
                message: `Updated file "${file.data.name}"`,
                timeout: 3000,
                background: "variant-filled-success"
            });
        } else {
            toastStore.trigger({
                message: "Failed to update file: " + res.error.toString(),
                timeout: 3000,
                background: "variant-filled-error"
            });
        }
    }
</script>

<div class="card p-4 w-full w-modal shadow-xl space-y-6">
    <header class="flex flex-row justify-between items-start gap-3">
        <div class="flex flex-col items-start gap-3">
            <div class="inline-block min-w-0">
                <p class="block text-2xl font-bold">
                    File: {valueName}
                </p>
            </div>
            <code class="sm:text-sm code">{file.formatSize()}</code>
        </div>

        <button
            class="btn-icon variant-ghost-surface"
            onclick={modalStore.close}
        >
            <CloseIcon />
        </button>
    </header>

    <form
        class=" sm:border sm:border-surface-500 sm:rounded-container-token sm:p-4 space-y-4 overflow-hidden"
        onsubmit={handleUpdate}
    >
        <div class="space-y-3">
            <div
                class="input-group input-group-divider grid-cols-[auto_1fr_auto]"
            >
                <div class="input-group-shim sm:w-28 inline-block">ID</div>
                <input type="text" value={file.id} disabled />
                <Clipboard {classes} {activeClasses} content={file.id} />
            </div>

            <div
                class="input-group input-group-divider grid-cols-[auto_1fr_auto]"
            >
                <div class="input-group-shim sm:w-28">User ID</div>
                <input type="text" value={file.userId} disabled />
                <Clipboard {classes} {activeClasses} content={file.userId} />
            </div>

            <div
                class="input-group input-group-divider grid-cols-[auto_1fr_auto]"
            >
                <div class="input-group-shim sm:w-28">Created at</div>
                <input
                    type="text"
                    value={file.createdAt.toLocaleString()}
                    disabled
                />
                <Clipboard
                    {classes}
                    {activeClasses}
                    content={file.createdAt.toLocaleString()}
                />
            </div>

            <div
                class="input-group input-group-divider grid-cols-[auto_1fr_auto]"
            >
                <div class="input-group-shim sm:w-28">Updated at</div>
                <input
                    type="text"
                    value={file.updatedAt.toLocaleString()}
                    disabled
                />
                <Clipboard
                    {classes}
                    {activeClasses}
                    content={file.updatedAt.toLocaleString()}
                />
            </div>

            <div
                class="input-group input-group-divider grid-cols-[auto_1fr_auto]"
            >
                <div class="input-group-shim sm:w-28">Checksum</div>
                <input type="text" value={file.data.checksum256} disabled />
                <Clipboard
                    {classes}
                    {activeClasses}
                    content={file.data.checksum256}
                />
            </div>

            <div
                class="input-group input-group-divider grid-cols-[auto_1fr_auto]"
            >
                <div class="input-group-shim sm:w-28">Name</div>
                <input type="text" bind:value={valueName} />
                <Clipboard {classes} {activeClasses} content={valueName} />
            </div>

            <div
                class="input-group input-group-divider grid-cols-[auto_1fr_auto]"
            >
                <div class="input-group-shim sm:w-28">Mime type</div>
                <input type="text" bind:value={valueMimeType} />
                <Clipboard {classes} {activeClasses} content={valueMimeType} />
            </div>
        </div>

        <FileDropzone
            name="files"
            bind:files={filesList}
            disabled={!!uploadedFile}
        >
            <svelte:fragment slot="message">
                {#if !!uploadedFile}
                    {#if !!uploading}
                        <progress
                            value={uploading.current}
                            max={uploading.total}
                        ></progress>
                        <strong>File "{uploadedFile.name}" uploading</strong>
                        <br />
                        <strong>{progressString(uploading)}</strong>
                    {:else}
                        <strong>File "{uploadedFile.name}" loaded</strong>
                    {/if}
                {:else}
                    <strong>Upload a file</strong> or drag and drop
                {/if}
            </svelte:fragment>
            <svelte:fragment slot="meta">
                The file content will be replaced
            </svelte:fragment>
        </FileDropzone>

        <button type="submit" class="btn variant-filled" disabled={!changed}>
            <span><CloudUploadIcon /></span>
            <span class="hidden sm:block">Update</span>
        </button>
    </form>
</div>
