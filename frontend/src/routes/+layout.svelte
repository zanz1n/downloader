<script lang="ts">
    import "../app.postcss";

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
        Toast,
        Modal
    } from "@skeletonlabs/skeleton";
    import Header from "$lib/components/Header.svelte";

    storePopup.set({ computePosition, autoUpdate, flip, shift, offset, arrow });
    initializeStores();

    let { children, data } = $props();
</script>

<Toast />
<Modal />

<div class="w-screen h-screen">
    <AppShell>
        <svelte:fragment slot="header">
            <Header auth={data.auth} />
        </svelte:fragment>
        {@render children()}
    </AppShell>
</div>
