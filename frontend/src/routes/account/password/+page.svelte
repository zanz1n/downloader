<script lang="ts">
    import { Authenticator } from "$lib/auth";
    import Form from "$lib/components/Form.svelte";
    import { joinErrorFields } from "$lib/utils";
    import { getToastStore } from "@skeletonlabs/skeleton";
    import { z } from "zod";

    const toastStore = getToastStore();
    const authenticator = Authenticator.getInstance();

    const formDataSchema = z.object({
        currentPassword: z.string().min(1),
        newPassword: z.string().min(1),
        confirmNewPassword: z.string().min(1)
    });

    async function handleChangePassword(oldpass: string, newpass: string) {
        const res = await authenticator.changePassword(oldpass, newpass);

        if (res.isOk()) {
            toastStore.trigger({
                message: "Successfully changed password",
                timeout: 5000,
                background: "variant-filled-success"
            });
        } else {
            toastStore.trigger({
                message: res.error.toString(),
                timeout: 5000,
                background: "variant-filled-error"
            });
        }
    }

    function onSubmit(rawData: unknown, reset: () => void) {
        const res = formDataSchema.safeParse(rawData);

        if (res.success) {
            if (res.data.newPassword != res.data.confirmNewPassword) {
                toastStore.trigger({
                    message: "Password and Confirm password must be equal",
                    timeout: 5000,
                    background: "variant-filled-error"
                });
                return;
            }

            reset();
            handleChangePassword(
                res.data.currentPassword,
                res.data.newPassword
            ).catch((e) => {
                console.error("handleChangePassword gone wrong:", e);
            });
        } else {
            const fmt = res.error.format();
            const error = joinErrorFields(";", ",", [
                { name: "Current password", obj: fmt.currentPassword },
                { name: "New password", obj: fmt.newPassword },
                { name: "Confirm new password", obj: fmt.confirmNewPassword }
            ]);

            toastStore.trigger({
                message: error,
                timeout: 5000,
                background: "variant-filled-error"
            });
        }
    }
</script>

<div
    class="container sm:bg-transparent sm:py-8 mx-auto bg-surface-800 w-full flex justify-center items-center"
>
    <Form title="Change password" {onSubmit}>
        <label class="label w-full" for="currentPassword">
            <span>Current password</span>
            <input
                name="currentPassword"
                class="input"
                type="password"
                placeholder="Current password"
                required
            />
        </label>

        <label class="label w-full" for="newPassword">
            <span>New password</span>
            <input
                name="newPassword"
                class="input"
                type="password"
                placeholder="New password"
                required
            />
        </label>

        <label class="label w-full" for="confirmNewPassword">
            <span>Confirm new password</span>
            <input
                name="confirmNewPassword"
                class="input"
                type="password"
                placeholder="Confirm new password"
                required
            />
        </label>

        {#snippet bottom()}
            <button class="btn variant-filled w-full" type="submit">
                <span>Change password</span>
            </button>
        {/snippet}
    </Form>
</div>
