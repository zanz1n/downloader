<script lang="ts">
    import LoginIcon from "$lib/assets/icons/LoginIcon.svelte";
    import Form from "$lib/components/Form.svelte";
</script>

<Form title="Login">
    <label class="label" for="">
        <span>Username</span>
        <input class="input" type="text" placeholder="Username" required />
    </label>

    <label class="label" for="">
        <span>Password</span>
        <input class="input" type="password" placeholder="Password" required />
    </label>

    <svelte:fragment slot="bottom">
        <button class="btn variant-filled w-full" type="submit">
            <span><LoginIcon /></span>
            <span>Log in</span>
        </button>
        <p>Or <a class="anchor" href="/auth/signup">create an account</a></p>
    </svelte:fragment>
</Form>
