<script lang="ts">
    import { Avatar } from "@skeletonlabs/skeleton";

    type Props = {
        width: string;
        username: string;
    };

    let { width, username }: Props = $props();

    function getInitials(name: string): string {
        const split = name.split(" ");
        if (split.length >= 2) {
            return split[0].at(0)! + split[1].at(0)!;
        } else if (name.length >= 2) {
            return name.at(0)! + name.at(1)!;
        }
        return "";
    }

    const initials = $derived(getInitials(username));
</script>

<Avatar
    {initials}
    border="border-2 border-surface-300-600-token hover:!border-primary-500"
    fontSize={200}
    {width}
/>
