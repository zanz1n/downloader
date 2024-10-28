<script lang="ts">
    import "../app.postcss";
    import type { PageData } from "./$types";

    import {
        computePosition,
        autoUpdate,
        flip,
        shift,
        offset,
        arrow
    } from "@floating-ui/dom";
    import {
        storePopup,
        AppShell,
        initializeStores,
        Toast
    } from "@skeletonlabs/skeleton";
    import Header from "$lib/components/Header.svelte";

    storePopup.set({ computePosition, autoUpdate, flip, shift, offset, arrow });
    initializeStores();

    export let data: PageData;
</script>

<Toast />

<div class="w-screen h-screen">
    <AppShell>
        <svelte:fragment slot="header">
            <Header auth={data.auth} />
        </svelte:fragment>
        <slot />
    </AppShell>
</div>
