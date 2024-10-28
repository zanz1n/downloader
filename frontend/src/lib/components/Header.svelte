<script lang="ts">
    import BugIcon from "$lib/assets/icons/BugIcon.svelte";
    import LogoutIcon from "$lib/assets/icons/LogoutIcon.svelte";
    import UserIcon from "$lib/assets/icons/UserIcon.svelte";
    import UserSettingsIcon from "$lib/assets/icons/UserSettingsIcon.svelte";
    import { Authenticator, type Auth } from "$lib/auth";
    import {
        AppBar,
        LightSwitch,
        popup,
        type PopupSettings,
        getToastStore
    } from "@skeletonlabs/skeleton";
    import { type Option } from "ts-results-es";
    import Avatar from "./Avatar.svelte";
    import GithubIcon from "$lib/assets/icons/GithubIcon.svelte";
    import { goto } from "$app/navigation";

    const toastStore = getToastStore();
    const authenticator = Authenticator.getInstance();

    export let auth: Option<Auth>;

    const popupAccount: PopupSettings = {
        event: "click",
        target: "popupAccount",
        placement: "bottom",
        closeQuery: ".popup-account-close"
    };

    function logout() {
        authenticator.logout();

        toastStore.trigger({
            message: `Successfully logged out`,
            timeout: 5000,
            background: "variant-filled-success"
        });

        goto("/", { invalidateAll: true });
    }
</script>

<div class="card p-4 w-60 shadow-xl" data-popup="popupAccount">
    {#if auth.isSome()}
        <div class="space-y-4">
            <div class="flex flex-col justify-center items-center gap-4">
                <Avatar username={auth.value.username} width="w-16" />

                <h4 class="h4">{auth.value.username}</h4>

                <div class="sm:hidden space-y-4">
                    <LightSwitch />
                </div>
            </div>
            <hr />
            <nav class="list-nav">
                <ul>
                    <li>
                        <a class="popup-account-close" href="/account">
                            <span><UserIcon /></span>
                            <span>Account</span>
                        </a>
                    </li>

                    <li>
                        <a class="popup-account-close" href="/settings">
                            <span><UserSettingsIcon /></span>
                            <span>Client settings</span>
                        </a>
                    </li>

                    <li>
                        <a
                            class="popup-account-close"
                            href="https://github.com/zanz1n/downloader/issues"
                            target="_blank"
                        >
                            <span><BugIcon /></span>
                            <span>Report bug</span>
                        </a>
                    </li>
                </ul>
            </nav>
            <hr />
            <div>
                <button
                    on:click={logout}
                    class="btn variant-filled w-full popup-account-close"
                >
                    <span><LogoutIcon /></span>
                    <span>Log out</span>
                </button>
            </div>
        </div>
    {/if}
</div>

<AppBar
    gridColumns="grid-cols-3"
    slotDefault="place-self-center"
    slotTrail="place-content-end"
>
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
                <a class="btn variant-filled" href="/auth/login">
                    <span><UserIcon /></span>
                    <span>Login</span>
                </a>
            {/if}
        </div>
    </svelte:fragment>
</AppBar>
