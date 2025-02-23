<script lang="ts">
    import { createEventDispatcher, getContext } from "svelte";
    import type { OpenChat } from "openchat-client";
    import ShareIcon from "svelte-material-icons/ShareVariant.svelte";
    import CopyIcon from "svelte-material-icons/ContentCopy.svelte";
    import QR from "svelte-qr";
    import { _ } from "svelte-i18n";
    import Link from "../../Link.svelte";
    import { iconSize } from "../../../stores/iconSize";
    import { toastStore } from "../../../stores/toast";
    import ViewUserProfile from "../profile/ViewUserProfile.svelte";
    import { AvatarSize } from "openchat-client";
    import Avatar from "../../Avatar.svelte";
    import LinkButton from "../../LinkButton.svelte";
    import { canShare, shareLink } from "../../../utils/share";

    const dispatch = createEventDispatcher();

    const client = getContext<OpenChat>("client");
    const user = client.user;

    $: userStore = client.userStore;

    let link = `${window.location.origin}/?ref=${user.userId}`;
    let viewedUserId: string | undefined = undefined;

    function onCopy() {
        navigator.clipboard.writeText(link).then(
            () => {
                toastStore.showSuccessToast("linkCopiedToClipboard");
            },
            () => {
                toastStore.showFailureToast("failedToCopyLinkToClipboard");
            }
        );
    }

    function onShare() {
        shareLink(link);
    }

    function showUserProfile(userId: string) {
        viewedUserId = userId;
    }

    function closeUserProfile() {
        viewedUserId = undefined;
    }

    function onChat() {
        if (viewedUserId !== undefined) {
            closeUserProfile();
            dispatch("chatWith", { kind: "direct_chat", userId: viewedUserId });
        }
    }
</script>

<div class="container">
    <div class="link">{link}</div>
    <div class="qr-wrapper">
        <div class="qr">
            <QR text={link} />
        </div>
    </div>
    <div class="message">
        {$_("userReferralMessage")}
    </div>
    <div class="action">
        <CopyIcon size={$iconSize} color={"var(--icon-txt)"} />
        <Link on:click={onCopy}>
            {$_("copy")}
        </Link>
    </div>
    {#if canShare()}
        <div class="action">
            <ShareIcon size={$iconSize} color={"var(--icon-txt)"} />
            <Link on:click={onShare}>
                {$_("share")}
            </Link>
        </div>
    {/if}
    {#if user.referrals.length > 0}
        <div class="referrals-section">
            <h4>{$_("invitedUsers")}</h4>
            <div class="referrals">
                {#each user.referrals as userId}
                    <div class="referral" on:click={() => showUserProfile(userId)}>
                        <div>
                            <Avatar
                                url={client.userAvatarUrl($userStore[userId])}
                                {userId}
                                size={AvatarSize.Default} />
                        </div>
                        <LinkButton underline="hover">
                            {client.usernameAndIcon($userStore[userId])}
                        </LinkButton>
                    </div>
                {/each}
            </div>
        </div>
    {/if}
</div>

{#if viewedUserId !== undefined}
    <ViewUserProfile userId={viewedUserId} on:openDirectChat={onChat} on:close={closeUserProfile} />
{/if}

<style lang="scss">
    .qr-wrapper {
        border: 1px solid var(--bd);
        .qr {
            background-color: #fff;
            margin: $sp5 auto;
            width: 200px;

            @include mobile() {
                width: 100%;
                margin: 0;
            }
        }
    }

    .link,
    .message {
        @include font(book, normal, fs-80);
    }

    .message {
        color: var(--txt-light);
    }

    .link {
        color: var(--link-underline);
    }

    .container {
        display: flex;
        flex-direction: column;
        gap: $sp4;
    }

    .action {
        display: flex;
        gap: $sp4;
        align-items: center;
    }

    .referrals-section {
        margin-top: $sp3;

        .referrals {
            display: flex;
            flex-direction: column;
            gap: $sp3;
            margin-top: $sp4;
            width: fit-content;

            .referral {
                cursor: pointer;
                align-items: center;
                display: flex;
                gap: $sp3;
            }
        }
    }
</style>
