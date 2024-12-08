<script lang="ts">
    import { goto } from "$app/navigation";
    import LoginIcon from "$lib/assets/icons/LoginIcon.svelte";
    import { Authenticator } from "$lib/auth";
    import Form from "$lib/components/Form.svelte";
    import { joinErrorFields } from "$lib/utils";
    import { getToastStore } from "@skeletonlabs/skeleton";
    import { z } from "zod";

    const toastStore = getToastStore();
    const authenticator = Authenticator.getInstance();

    const formDataSchema = z.object({
        username: z.string().min(1),
        password: z.string().min(1),
        confirmPassword: z.string().min(1),
        serverKey: z.string().min(1).base64()
    });

    type FormDataType = z.infer<typeof formDataSchema>;

    async function handleLogin(data: FormDataType) {
        const res = await authenticator.signup(
            {
                username: data.username,
                password: data.password
            },
            data.serverKey
        );

        if (res.isOk()) {
            toastStore.trigger({
                message: `Successfully signed up as ${res.value.username}`,
                timeout: 5000,
                background: "variant-filled-success"
            });

            await goto("/", { invalidateAll: true });
        } else {
            if (res.error.errorCode == 0) {
                toastStore.trigger({
                    message:
                        "Something went wrong: " +
                        "check the browser console for more information",
                    timeout: 5000,
                    background: "variant-filled-error"
                });

                console.error("Sign up gone wrong:", res.error);
            } else {
                toastStore.trigger({
                    message: res.error.toString(),
                    timeout: 5000,
                    background: "variant-filled-error"
                });
            }
        }
    }

    function onSubmit(rawData: unknown, reset: () => void) {
        const res = formDataSchema.safeParse(rawData);

        if (res.success) {
            if (res.data.password != res.data.confirmPassword) {
                toastStore.trigger({
                    message: "Password and Confirm password must be equal",
                    timeout: 5000,
                    background: "variant-filled-error"
                });
                return;
            }

            reset();
            handleLogin(res.data).catch((e) => {
                console.error("handleLogin gone wrong:", e);
            });
        } else {
            const fmt = res.error.format();
            const error = joinErrorFields(";", ",", [
                { name: "Username", obj: fmt.username },
                { name: "Password", obj: fmt.password },
                { name: "Confirm password", obj: fmt.confirmPassword },
                { name: "Server key", obj: fmt.serverKey }
            ]);

            toastStore.trigger({
                message: error,
                timeout: 5000,
                background: "variant-filled-error"
            });
        }
    }
</script>

<Form {onSubmit} title="Sign Up">
    <label class="label w-full" for="username">
        <span>Username</span>
        <input
            name="username"
            class="input"
            type="text"
            placeholder="Username"
            required
        />
    </label>

    <label class="label w-full" for="password">
        <span>Password</span>
        <input
            name="password"
            class="input"
            type="password"
            placeholder="Password"
            required
        />
    </label>

    <label class="label w-full" for="confirmPassword">
        <span>Confirm password</span>
        <input
            name="confirmPassword"
            class="input"
            type="password"
            placeholder="Confirm password"
            required
        />
    </label>

    <label class="label w-full" for="serverKey">
        <span>Server key</span>
        <input
            name="serverKey"
            class="input"
            type="text"
            placeholder="Server key"
            required
        />
    </label>

    {#snippet bottom()}
        <button class="btn variant-filled w-full" type="submit">
            <span><LoginIcon /></span>
            <span>Sign up</span>
        </button>
        <p>
            Or <a class="anchor" href="/auth/login"
                >login to an existing account</a
            >
        </p>
    {/snippet}
</Form>
