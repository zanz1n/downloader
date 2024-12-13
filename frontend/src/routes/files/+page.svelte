<script lang="ts">
    import RefreshIcon from "$lib/assets/icons/RefreshIcon.svelte";
    import SearchIcon from "$lib/assets/icons/SearchIcon.svelte";
    import UploadIcon from "$lib/assets/icons/UploadIcon.svelte";
    import FileRow from "$lib/components/file/FileRow.svelte";
    import { Files } from "$lib/file";
    import {
        getToastStore,
        ProgressRadial,
        SlideToggle
    } from "@skeletonlabs/skeleton";

    const toastStore = getToastStore();
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
            <div class="input-group-shim">
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
                    <FileRow {file} {refresh} />
                {/each}
            {/if}
        {/await}
    </div>
</div>
