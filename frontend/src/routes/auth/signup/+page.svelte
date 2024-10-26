<script lang="ts">
    import LoginIcon from "$lib/assets/icons/LoginIcon.svelte";
    import Form from "$lib/components/Form.svelte";
</script>

<Form title="Sign Up">
    <label class="label w-full" for="username">
        <span>Username</span>
        <input class="input" type="text" placeholder="Username" required />
    </label>

    <label class="label w-full" for="password">
        <span>Password</span>
        <input class="input" type="password" placeholder="Password" required />
    </label>

    <label class="label w-full" for="password">
        <span>Confirm password</span>
        <input class="input" type="password" placeholder="Confirm password" required />
    </label>

    <label class="label w-full" for="">
        <span>Server Key</span>
        <input class="input" type="password" placeholder="Server Key" required />
    </label>

    <svelte:fragment slot="bottom">
        <button class="btn variant-filled w-full" type="submit">
            <span><LoginIcon /></span>
            <span>Sign up</span>
        </button>
        <p>Or <a class="anchor" href="/auth/login">login to an existing account</a></p>
    </svelte:fragment>
</Form>
