<script lang="ts">
    import { Avatar } from "@skeletonlabs/skeleton";

    function getInitials(name: string): string {
        const split = name.split(" ");
        if (split.length >= 2) {
            return split[0].at(0)! + split[1].at(0)!;
        } else if (name.length >= 2) {
            return name.at(0)! + name.at(1)!;
        }
        return "";
    }

    export let width: string;
    export let username: string;
</script>

<Avatar
    initials={getInitials(username)}
    border="border-2 border-surface-300-600-token hover:!border-primary-500"
    fontSize={200}
    {width}
/>
