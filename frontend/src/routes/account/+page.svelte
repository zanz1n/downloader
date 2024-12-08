<script lang="ts">
    import { goto } from "$app/navigation";
    import { Authenticator } from "$lib/auth";
    import { getToastStore } from "@skeletonlabs/skeleton";

    const toastStore = getToastStore();
    const authenticator = Authenticator.getInstance();

    let { data } = $props();

    async function getUserData() {
        if (data.auth.isNone()) {
            toastStore.trigger({
                message: "Not logged in!",
                timeout: 5000,
                background: "variant-filled-error"
            });

            return goto("/", { invalidateAll: true });
        }

        const user = await authenticator.getUser();
        if (user.isErr()) {
            toastStore.trigger({
                message:
                    "Something went wrong: " +
                    "check the browser console for more information",
                timeout: 5000,
                background: "variant-filled-error"
            });

            console.error("Get user information gone wrong:", user.error);
            return goto("/", { invalidateAll: true });
        }

        return { user: user.value, auth: data.auth.value };
    }

    let pageData = getUserData();
</script>

<div class="w-full p-8 flex flex-row justify-center gap-8 flex-wrap">
    {#await pageData then data}
        {#if data}
            <div
                class="card size-fit flex flex-col items-center md:gap-6 gap-4 p-6"
            >
                <h2 class="h2">User info</h2>
                <ul class="text-lg">
                    <li>
                        <b>ID</b>:
                        <code class="code text-xs sm:text-base">
                            {data.user.id}
                        </code>
                    </li>
                    <li>
                        <b>Username</b>:
                        <code class="code text-base">{data.user.username}</code>
                    </li>
                    <li>
                        <b>Created at</b>:
                        <code class="code text-base">
                            {data.user.createdAt.toLocaleString()}
                        </code>
                    </li>
                    <li>
                        <b>Updated at</b>:
                        <code class="code text-base">
                            {data.user.updatedAt.toLocaleString()}
                        </code>
                    </li>
                    <li>
                        <b>Permission</b>:
                        <code class="code text-base">
                            {data.user.permission}
                        </code>
                    </li>
                </ul>
            </div>

            <div
                class="card size-fit flex flex-col items-center md:gap-6 gap-4 p-6"
            >
                <h2 class="h2">Auth info</h2>
                <ul class="text-lg">
                    <li>
                        <b>Issuer</b>:
                        <code class="code text-base">{data.auth.issuer}</code>
                    </li>
                    <li>
                        <b>Logged at</b>:
                        <code class="code text-base">
                            {data.auth.createdAt.toLocaleString()}
                        </code>
                    </li>
                    <li>
                        <b>Expiration</b>:
                        <code class="code text-base">
                            {data.auth.expiresAt.toLocaleString()}
                        </code>
                    </li>
                    <li>
                        <b>Permission</b>:
                        <code class="code text-base"
                            >{data.auth.permission}</code
                        >
                    </li>
                </ul>
            </div>
        {/if}
    {/await}
</div>
