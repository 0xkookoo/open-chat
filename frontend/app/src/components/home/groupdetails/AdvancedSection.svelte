<script lang="ts">
    import { _ } from "svelte-i18n";
    import { createEventDispatcher } from "svelte";
    import Button from "../../Button.svelte";
    import ButtonGroup from "../../ButtonGroup.svelte";
    import type { MultiUserChat } from "openchat-client";
    import { interpolateLevel } from "utils/i18n";

    export let group: MultiUserChat;

    const dispatch = createEventDispatcher();

    function deleteGroup() {
        dispatch("deleteGroup", {
            kind: "delete",
            chatId: group.id,
            level: group.level,
            doubleCheck: {
                challenge: $_("typeGroupName", { values: { name: group.name } }),
                response: group.name,
            },
        });
    }
</script>

<ButtonGroup align="start">
    <Button on:click={deleteGroup}>{interpolateLevel("deleteGroup", group.level)}</Button>
</ButtonGroup>
