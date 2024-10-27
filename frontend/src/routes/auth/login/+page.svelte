<script lang="ts">
    import { goto } from "$app/navigation";
    import LoginIcon from "$lib/assets/icons/LoginIcon.svelte";
    import { Authenticator, type LoginData } from "$lib/auth";
    import Form from "$lib/components/Form.svelte";
    import { joinErrorFields } from "$lib/utils";
    import { getToastStore } from "@skeletonlabs/skeleton";
    import { z } from "zod";

    const toastStore = getToastStore();
    const authenticator = Authenticator.getInstance();

    const formDataSchema = z.object({
        username: z.string().min(1),
        password: z.string().min(1)
    });

    type FormDataType = z.infer<typeof formDataSchema>;

    async function handleSignin(data: FormDataType) {
        const res = await authenticator.login(data satisfies LoginData);

        if (res.isOk()) {
            toastStore.trigger({
                message: `Successfully logged in as ${res.value.username}`,
                timeout: 5000,
                background: "variant-filled-success"
            });

            await goto("/", { invalidateAll: true });
        } else {
            if (res.error.errorCode == 0) {
                toastStore.trigger({
                    message:
                        "Something went wrong: " + "check the browser console for more information",
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

    function onSubmit(rawData: any) {
        const res = formDataSchema.safeParse(rawData);

        if (res.success) {
            handleSignin(res.data).catch((e) => {
                console.error("handleSignin gone wrong:", e);
            });
        } else {
            const fmt = res.error.format();
            const error = joinErrorFields(";", ",", [
                { name: "Username", obj: fmt.username },
                { name: "Password", obj: fmt.password }
            ]);

            toastStore.trigger({
                message: error,
                timeout: 5000,
                background: "variant-filled-error"
            });
        }
    }
</script>

<Form {onSubmit} title="Login">
    <label class="label w-full" for="username">
        <span>Username</span>
        <input name="username" class="input" type="text" placeholder="Username" required />
    </label>

    <label class="label w-full" for="password">
        <span>Password</span>
        <input name="password" class="input" type="password" placeholder="Password" required />
    </label>

    <svelte:fragment slot="bottom">
        <button class="btn variant-filled w-full" type="submit">
            <span><LoginIcon /></span>
            <span>Log in</span>
        </button>
        <p>Or <a class="anchor" href="/auth/signup">create an account</a></p>
    </svelte:fragment>
</Form>
