<script lang="ts">
    import { page } from "$app/stores";
    import { TabAnchor, TabGroup } from "@skeletonlabs/skeleton";

    const selected = (s: string) => $page.url.pathname === s;

    let { children } = $props();
</script>

<div class="size-full flex flex-col">
    <TabGroup justify="justify-center" class="bg-surface-100-800-token w-full">
        <TabAnchor href="/account" selected={selected("/account")}>
            Account
        </TabAnchor>

        <TabAnchor
            href="/account/password"
            selected={selected("/account/password")}
        >
            Change password
        </TabAnchor>

        <!-- <TabAnchor href="/account/api" selected={selected("/account/api")}>
            API key
        </TabAnchor> -->
    </TabGroup>

    {@render children()}
</div>
