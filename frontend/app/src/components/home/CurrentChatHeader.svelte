<script lang="ts">
    import { AvatarSize, OpenChat, TypersByKey } from "openchat-client";
    import { mobileWidth } from "../../stores/screenDimensions";
    import CurrentChatMenu from "./CurrentChatMenu.svelte";
    import SectionHeader from "../SectionHeader.svelte";
    import ChatSubtext from "./ChatSubtext.svelte";
    import ArrowLeft from "svelte-material-icons/ArrowLeft.svelte";
    import ArrowRight from "svelte-material-icons/ArrowRight.svelte";
    import Avatar from "../Avatar.svelte";
    import HoverIcon from "../HoverIcon.svelte";
    import { createEventDispatcher, getContext } from "svelte";
    import { _ } from "svelte-i18n";
    import { rtlStore } from "../../stores/rtl";
    import type { ChatSummary } from "openchat-client";
    import Typing from "../Typing.svelte";
    import { iconSize } from "../../stores/iconSize";
    import { now } from "../../stores/time";
    import ViewUserProfile from "./profile/ViewUserProfile.svelte";
    import SuspendModal from "./SuspendModal.svelte";
    import { rightPanelHistory } from "../../stores/rightPanel";

    const client = getContext<OpenChat>("client");
    const dispatch = createEventDispatcher();

    export let selectedChatSummary: ChatSummary;
    export let blocked: boolean;
    export let readonly: boolean;
    export let unreadMessages: number;
    export let hasPinned: boolean;

    let viewProfile = false;
    let showSuspendUserModal = false;

    $: typersByContext = client.typersByContext;
    $: userStore = client.userStore;
    $: userId = selectedChatSummary.kind === "direct_chat" ? selectedChatSummary.them.userId : "";
    $: isMultiUser =
        selectedChatSummary.kind === "group_chat" || selectedChatSummary.kind === "channel";
    $: isBot = $userStore[userId]?.kind === "bot";
    $: hasUserProfile = !isMultiUser && !isBot;
    $: selectedChatId = client.selectedChatId;

    function clearSelection() {
        dispatch("clearSelection");
    }

    function showGroupDetails() {
        if ($selectedChatId !== undefined) {
            rightPanelHistory.set([
                {
                    kind: "group_details",
                },
            ]);
        }
    }

    function showGroupMembers() {
        dispatch("showGroupMembers");
    }

    function normaliseChatSummary(now: number, chatSummary: ChatSummary, typing: TypersByKey) {
        switch (chatSummary.kind) {
            case "direct_chat":
                const them = $userStore[chatSummary.them.userId];
                return {
                    name: client.usernameAndIcon(them),
                    avatarUrl: client.userAvatarUrl(them),
                    userId: chatSummary.them.userId,
                    typing: client.getTypingString(
                        $_,
                        $userStore,
                        { chatId: chatSummary.id },
                        typing
                    ),
                };
            default:
                return {
                    name: chatSummary.name,
                    avatarUrl: client.groupAvatarUrl(chatSummary),
                    userId: undefined,
                    typing: client.getTypingString(
                        $_,
                        $userStore,
                        { chatId: chatSummary.id },
                        typing
                    ),
                };
        }
    }

    function openUserProfile() {
        if (hasUserProfile) {
            viewProfile = true;
        }
    }

    function closeUserProfile() {
        viewProfile = false;
    }

    $: chat = normaliseChatSummary($now, selectedChatSummary, $typersByContext);
</script>

{#if showSuspendUserModal}
    <SuspendModal {userId} on:close={() => (showSuspendUserModal = false)} />
{/if}

<SectionHeader shadow flush>
    {#if $mobileWidth}
        <div class="back" class:rtl={$rtlStore} on:click={clearSelection}>
            <HoverIcon>
                {#if $rtlStore}
                    <ArrowRight size={$iconSize} color={"var(--icon-txt)"} />
                {:else}
                    <ArrowLeft size={$iconSize} color={"var(--icon-txt)"} />
                {/if}
            </HoverIcon>
        </div>
    {/if}
    {#if viewProfile}
        <ViewUserProfile {userId} chatButton={false} on:close={closeUserProfile} />
    {/if}

    <div class="avatar" class:has-user-profile={hasUserProfile} on:click={openUserProfile}>
        <Avatar
            statusBorder={"var(--section-bg)"}
            {blocked}
            showStatus={true}
            userId={chat.userId}
            url={chat.avatarUrl}
            size={AvatarSize.Default} />
    </div>
    <div class="chat-details">
        <div class="chat-name" title={chat.name}>
            {#if isMultiUser && !readonly}
                <span on:click={showGroupDetails} class="group-details">
                    {chat.name}
                </span>
            {:else if hasUserProfile}
                <span on:click={openUserProfile} class="user-link">
                    {chat.name}
                </span>
            {:else}
                {chat.name}
            {/if}
        </div>
        <div class="chat-subtext">
            {#if blocked}
                {$_("blocked")}
            {:else if readonly}
                <ChatSubtext chat={selectedChatSummary} />
            {:else if chat.typing !== undefined}
                {chat.typing} <Typing />
            {:else if isMultiUser}
                <div class="members" on:click={showGroupMembers}>
                    <ChatSubtext chat={selectedChatSummary} />
                </div>
            {:else}
                <ChatSubtext chat={selectedChatSummary} />
            {/if}
        </div>
    </div>
    {#if !readonly}
        <CurrentChatMenu
            bind:showSuspendUserModal
            {hasPinned}
            {unreadMessages}
            {selectedChatSummary}
            {blocked}
            on:convertGroupToCommunity
            on:importToCommunity
            on:toggleMuteNotifications
            on:showGroupDetails={showGroupDetails}
            on:searchChat
            on:showProposalFilters
            on:showGroupMembers
            on:createPoll
            on:markAllRead
            on:upgrade
            on:showInviteGroupUsers
            on:leaveGroup />
    {/if}
</SectionHeader>

<style lang="scss">
    .chat-name {
        @include font(book, normal, fs-120);
        @include ellipsis();
        margin-bottom: $sp1;
    }

    .chat-subtext {
        @include font(book, normal, fs-80);
        @include ellipsis();
        color: var(--txt-light);

        .members {
            cursor: pointer;
        }
    }

    .avatar {
        flex: 0 0 55px;

        &.has-user-profile {
            cursor: pointer;
        }
    }

    .group-details {
        cursor: pointer;
    }

    .user-link {
        cursor: pointer;
        &:hover {
            text-decoration: underline;
        }
    }

    .chat-details {
        flex: 1;
        overflow: auto;
        padding: 0 $sp2;
    }

    .back {
        flex: 0 0 10px;
        margin-right: 5px;

        &.rtl {
            margin-right: 0;
            margin-left: 5px;
        }
    }
</style>
