<script lang="ts">
    import type { File } from "$lib/file";

    type Props = {
        file: File;
    };

    let { file }: Props = $props();
</script>

<h1 class="h1">File "{file.data.name}"</h1>
