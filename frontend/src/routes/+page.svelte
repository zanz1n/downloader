<script lang="ts">
    import { goto } from "$app/navigation";

    let { data } = $props();

    if (data.auth.isSome()) {
        goto("/files");
    } else {
        goto("/auth/login");
    }
</script>
