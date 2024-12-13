<script lang="ts">
    import RefreshIcon from "$lib/assets/icons/RefreshIcon.svelte";
    import SearchIcon from "$lib/assets/icons/SearchIcon.svelte";
    import UploadIcon from "$lib/assets/icons/UploadIcon.svelte";
    import FileModal from "$lib/components/file/FileModal.svelte";
    import FileRow from "$lib/components/file/FileRow.svelte";
    import FileShare from "$lib/components/file/FileShare.svelte";
    import { File, Files } from "$lib/file";
    import {
        getModalStore,
        getToastStore,
        ProgressRadial,
        SlideToggle,
        type ModalComponent,
        type ModalSettings
    } from "@skeletonlabs/skeleton";

    const toastStore = getToastStore();
    const modalStore = getModalStore();
    const files = Files.getInstance();

    const count = 10;

    async function fetchFiles(
        page: number,
        all: boolean,
        _filter: string,
        _c: number
    ) {
        let res;
        if (all) {
            res = await files.getAllFiles(count, page * count);
        } else {
            res = await files.getUserFiles(count, page * count);
        }

        if (res.isErr()) {
            toastStore.trigger({
                message: res.error.toString(),
                timeout: 5000,
                background: "variant-filled-error"
            });
        }

        return res;
    }

    let showAll = $state(false);
    let currentPage = $state(0);
    let updateCt = $state(0);
    let filter = $state("");

    let pageData = $derived(fetchFiles(currentPage, showAll, filter, updateCt));

    function refresh() {
        updateCt++;
    }

    function deleteFile(file: File) {
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

    function openFileInfo(file: File) {
        const component: ModalComponent = {
            ref: FileModal,
            props: { file }
        };

        const modal: ModalSettings = {
            type: "component",
            component
        };

        modalStore.trigger(modal);
    }

    function shareFile(file: File) {
        const component: ModalComponent = {
            ref: FileShare,
            props: { file }
        };

        const modal: ModalSettings = {
            type: "component",
            component
        };

        modalStore.trigger(modal);
    }
</script>

<div
    class="max-w-[1200px] mx-auto sm:p-6 p-3 gap-8 flex flex-col justify-center"
>
    <div class="flex flex-row items-center justify-between p-1">
        <h2 class="h2">
            {#if showAll}
                All files
            {:else}
                Your files
            {/if}
        </h2>
        <div class="flex flex-row gap-4">
            <p>
                {#if showAll}
                    Showing all files
                {:else}
                    Showing your files
                {/if}
            </p>
            <SlideToggle
                size="sm"
                name="slider-label"
                bind:checked={showAll}
                on:change={() => {
                    currentPage = 0;
                }}
            />
        </div>
    </div>

    <div class="flex flex-row justify-center items-center gap-3">
        <div class="hidden lg:block">
            <button class="btn variant-filled-secondary" onclick={refresh}>
                <span><RefreshIcon /></span>
                <span>Refresh</span>
            </button>
        </div>

        <div class="input-group input-group-divider grid-cols-[auto_1fr_auto]">
            <div class="input-group-shim hidden">
                <SearchIcon />
            </div>
            <input type="search" placeholder="Search by name" />
            <button class="variant-filled-secondary">Search</button>
        </div>

        <button class="btn variant-filled-secondary">
            <span><UploadIcon /></span>
            <span class="hidden lg:block">Upload File</span>
        </button>
    </div>

    <div class="flex flex-col items-center w-full gap-3">
        {#await pageData}
            <ProgressRadial width="w-16 py-16" value={undefined} />
        {:then data}
            {#if data.isOk()}
                {#each data.value as file}
                    <FileRow
                        {file}
                        triggerDelete={() => deleteFile(file)}
                        triggerOpenInfo={() => openFileInfo(file)}
                        triggerShare={() => shareFile(file)}
                    />
                {/each}
            {/if}
        {/await}
    </div>
</div>
