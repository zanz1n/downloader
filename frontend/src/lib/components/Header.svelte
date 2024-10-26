<script lang="ts">
    import BugIcon from "$lib/assets/icons/BugIcon.svelte";
    import LogoutIcon from "$lib/assets/icons/LogoutIcon.svelte";
    import UserIcon from "$lib/assets/icons/UserIcon.svelte";
    import UserSettingsIcon from "$lib/assets/icons/UserSettingsIcon.svelte";
    import type { Auth } from "$lib/auth";
    import { AppBar, LightSwitch, popup, type PopupSettings } from "@skeletonlabs/skeleton";
    import { type Option } from "ts-results-es";
    import Avatar from "./Avatar.svelte";
    import GithubIcon from "$lib/assets/icons/GithubIcon.svelte";

    export let auth: Option<Auth>;

    const popupAccount: PopupSettings = {
        event: "click",
        target: "popupAccount",
        placement: "bottom",
        closeQuery: ".listbox-item"
    };
</script>

<div class="card p-4 w-60 shadow-xl" data-popup="popupAccount">
    {#if auth.isSome()}
        <div class="space-y-4">
            <div class="flex flex-col justify-center items-center gap-4">
                <Avatar username={auth.value.username} width="w-16" />

                <h4 class="h4">Izan Rodrigues</h4>

                <div class="sm:hidden space-y-4">
                    <LightSwitch />
                </div>
            </div>
            <hr />
            <nav class="list-nav">
                <ul>
                    <li>
                        <a href="/account">
                            <span><UserIcon /></span>
                            <span>Account</span>
                        </a>
                    </li>

                    <li>
                        <a href="/settings">
                            <span><UserSettingsIcon /></span>
                            <span>Client settings</span>
                        </a>
                    </li>

                    <li>
                        <a href="https://github.com/zanz1n/downloader/issues" target="_blank">
                            <span><BugIcon /></span>
                            <span>Report bug</span>
                        </a>
                    </li>
                </ul>
            </nav>
            <hr />
            <div>
                <button class="btn variant-filled w-full">
                    <span><LogoutIcon /></span>
                    <span>Log out</span>
                </button>
            </div>
        </div>
    {/if}
</div>

<AppBar gridColumns="grid-cols-3 " slotDefault="place-self-center" slotTrail="place-content-end">
    <svelte:fragment slot="lead">
        <a href="/">
            <h3 class="h3">Downloader</h3>
        </a>
    </svelte:fragment>

    <svelte:fragment slot="trail">
        <div class="flex items-center content-center flex-row gap-4">
            <div class="hidden sm:block">
                <LightSwitch />
            </div>
            <div class="hidden sm:inline-flex space-x-1">
                <a
                    class="btn-icon hover:variant-soft-primary"
                    href="https://github.com/zanz1n/downloader"
                    target="_blank"
                >
                    <GithubIcon />
                </a>
                <a
                    class="btn-icon hover:variant-soft-primary"
                    href="https://github.com/zanz1n/downloader"
                    target="_blank"
                >
                    <UserSettingsIcon />
                </a>
            </div>

            {#if auth.isSome()}
                <button use:popup={popupAccount}>
                    <Avatar username={auth.value.username} width="w-11" />
                </button>
            {:else}
                <a class="btn-icon variant-filled w-24 h-10" href="/auth/login">
                    <span class="w-7"><UserIcon /></span>
                    Login
                </a>
            {/if}
        </div>
    </svelte:fragment>
</AppBar>
