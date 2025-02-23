<script lang="ts">
    import ThreadHeader from "./ThreadHeader.svelte";
    import Footer from "../Footer.svelte";
    import type {
        ChatSummary,
        ChatEvent as ChatEventType,
        EnhancedReplyContext,
        EventWrapper,
        FailedMessages,
        Message,
        MessageContent,
        OpenChat,
        User,
    } from "openchat-client";
    import { ICP_SYMBOL } from "openchat-client";
    import { getContext, onMount } from "svelte";
    import { _ } from "svelte-i18n";
    import Loading from "../../Loading.svelte";
    import { derived, readable } from "svelte/store";
    import PollBuilder from "../PollBuilder.svelte";
    import GiphySelector from "../GiphySelector.svelte";
    import CryptoTransferBuilder from "../CryptoTransferBuilder.svelte";
    import { toastStore } from "stores/toast";
    import ChatEvent from "../ChatEvent.svelte";
    import ChatEventList from "../ChatEventList.svelte";
    import { randomSentence } from "../../../utils/randomMsg";

    const client = getContext<OpenChat>("client");
    const user = client.user;

    export let rootEvent: EventWrapper<Message>;
    export let chat: ChatSummary;

    let chatEventList: ChatEventList | undefined;
    let observer: IntersectionObserver = new IntersectionObserver(() => {});
    let pollBuilder: PollBuilder;
    let giphySelector: GiphySelector;
    let creatingPoll = false;
    let creatingCryptoTransfer: { ledger: string; amount: bigint } | undefined = undefined;
    let selectingGif = false;
    let initialised = false;
    let messagesDiv: HTMLDivElement | undefined;
    let messagesDivHeight: number;
    let previousLatestEventIndex: number | undefined = undefined;

    $: selectedMessageContext = client.selectedMessageContext;
    $: focusMessageIndex = client.focusThreadMessageIndex;
    $: currentChatMembers = client.currentChatMembers;
    $: lastCryptoSent = client.lastCryptoSent;
    $: draftThreadMessages = client.draftThreadMessages;
    $: unconfirmed = client.unconfirmed;
    $: currentChatBlockedUsers = client.currentChatBlockedUsers;
    $: threadEvents = client.threadEvents;
    $: failedMessagesStore = client.failedMessagesStore;
    $: threadRootMessageIndex = rootEvent.event.messageIndex;
    $: threadRootMessage = rootEvent.event;
    $: blocked = chat.kind === "direct_chat" && $currentChatBlockedUsers.has(chat.them.userId);
    $: draftMessage = readable(draftThreadMessages.get(threadRootMessageIndex), (set) =>
        draftThreadMessages.subscribe((d) => set(d[threadRootMessageIndex] ?? {}))
    );
    $: textContent = derived(draftMessage, (d) => d.textContent);
    $: replyingTo = derived(draftMessage, (d) => d.replyingTo);
    $: fileToAttach = derived(draftMessage, (d) => d.attachment);
    $: editingEvent = derived(draftMessage, (d) => d.editingEvent);
    $: canSend = client.canReplyInThread(chat.id);
    $: canReact = client.canReactToMessages(chat.id);
    $: expandedDeletedMessages = client.expandedDeletedMessages;
    $: atRoot = $threadEvents.length === 0 || $threadEvents[0]?.index === 0;
    $: events = atRoot ? [rootEvent, ...$threadEvents] : $threadEvents;
    $: messages = client.groupEvents(
        events,
        user.userId,
        $expandedDeletedMessages
    ) as EventWrapper<Message>[][][];
    $: readonly = client.isChatReadOnly(chat.id);
    $: thread = rootEvent.event.thread;
    $: loading = !initialised && $threadEvents.length === 0 && thread !== undefined;

    onMount(() => (previousLatestEventIndex = thread?.latestEventIndex));

    $: {
        if (initialised) {
            if (
                thread !== undefined &&
                previousLatestEventIndex !== undefined &&
                thread.latestEventIndex > previousLatestEventIndex
            ) {
                client.loadNewMessages(chat.id, rootEvent);
                previousLatestEventIndex = thread.latestEventIndex;
            }
        }
    }

    function createTestMessages(ev: CustomEvent<number>): void {
        if (process.env.NODE_ENV === "production") return;

        function send(n: number) {
            if (n === ev.detail) return;

            sendMessageWithAttachment(randomSentence(), [], undefined);

            setTimeout(() => send(n + 1), 500);
        }

        send(0);
    }

    function dateGroupKey(group: EventWrapper<Message>[][]): string {
        const first = group[0] && group[0][0] && group[0][0].timestamp;
        return first ? new Date(Number(first)).toDateString() : "unknown";
    }

    function sendMessage(ev: CustomEvent<[string | undefined, User[]]>) {
        if (!canSend) return;
        let [text, mentioned] = ev.detail;
        if ($editingEvent !== undefined) {
            client
                .editMessageWithAttachment(
                    chat.id,
                    text,
                    $fileToAttach,
                    $editingEvent,
                    threadRootMessageIndex
                )
                .then((success) => {
                    if (!success) {
                        toastStore.showFailureToast("errorEditingMessage");
                    }
                });
        } else {
            sendMessageWithAttachment(text, mentioned, $fileToAttach);
        }
        draftThreadMessages.delete(threadRootMessageIndex);
    }

    function editEvent(ev: EventWrapper<Message>): void {
        draftThreadMessages.setEditing(threadRootMessageIndex, ev);
    }

    function retrySend(ev: CustomEvent<EventWrapper<Message>>): void {
        client.retrySendMessage(chat.id, ev.detail, $threadEvents, threadRootMessageIndex);
    }

    function sendMessageWithAttachment(
        textContent: string | undefined,
        mentioned: User[],
        fileToAttach: MessageContent | undefined
    ) {
        client.sendMessageWithAttachment(
            chat.id,
            $threadEvents,
            textContent,
            mentioned,
            fileToAttach,
            $replyingTo,
            threadRootMessageIndex
        );
    }

    function cancelReply() {
        draftThreadMessages.setReplyingTo(threadRootMessageIndex, undefined);
    }

    function clearAttachment() {
        draftThreadMessages.setAttachment(threadRootMessageIndex, undefined);
    }

    function cancelEditEvent() {
        draftThreadMessages.delete(threadRootMessageIndex);
    }

    function setTextContent(ev: CustomEvent<string | undefined>) {
        draftThreadMessages.setTextContent(threadRootMessageIndex, ev.detail);
    }

    function onStartTyping() {
        client.startTyping(chat, user.userId, threadRootMessageIndex);
    }

    function onStopTyping() {
        client.stopTyping(chat, user.userId, threadRootMessageIndex);
    }

    function fileSelected(ev: CustomEvent<MessageContent>) {
        draftThreadMessages.setAttachment(threadRootMessageIndex, ev.detail);
    }

    function tokenTransfer(ev: CustomEvent<{ ledger: string; amount: bigint } | undefined>) {
        creatingCryptoTransfer = ev.detail ?? {
            ledger: $lastCryptoSent ?? client.ledgerCanisterId(ICP_SYMBOL),
            amount: BigInt(0),
        };
    }

    function createPoll() {
        if (!client.canCreatePolls(chat.id)) return;

        if (pollBuilder !== undefined) {
            pollBuilder.resetPoll();
        }
        creatingPoll = true;
    }

    function attachGif(ev: CustomEvent<string>) {
        selectingGif = true;
        if (giphySelector !== undefined) {
            giphySelector.reset(ev.detail);
        }
    }

    function sendMessageWithContent(ev: CustomEvent<[MessageContent, string | undefined]>) {
        sendMessageWithAttachment(ev.detail[1], [], ev.detail[0]);
    }

    function replyTo(ev: CustomEvent<EnhancedReplyContext>) {
        draftThreadMessages.setReplyingTo(threadRootMessageIndex, ev.detail);
    }

    function defaultCryptoTransferReceiver(): string | undefined {
        return $replyingTo?.sender?.userId;
    }

    function eventKey(e: EventWrapper<ChatEventType>): string {
        if (e.event.kind === "message") {
            return e.event.messageId.toString();
        } else {
            return e.index.toString();
        }
    }

    function isConfirmed(_unconf: unknown, evt: EventWrapper<ChatEventType>): boolean {
        if (evt.event.kind === "message" && $selectedMessageContext) {
            return !unconfirmed.contains($selectedMessageContext, evt.event.messageId);
        }
        return true;
    }

    function isFailed(_failed: FailedMessages, evt: EventWrapper<ChatEventType>): boolean {
        if (evt.event.kind === "message" && $selectedMessageContext) {
            return failedMessagesStore.contains($selectedMessageContext, evt.event.messageId);
        }
        return false;
    }

    function goToMessageIndex(index: number) {
        chatEventList?.scrollToMessageIndex(chat.id, index, false);
    }

    function onGoToMessageIndex(
        ev: CustomEvent<{ index: number; preserveFocus: boolean; messageId: bigint }>
    ) {
        goToMessageIndex(ev.detail.index);
    }
</script>

<PollBuilder
    bind:this={pollBuilder}
    on:sendPoll={sendMessageWithContent}
    bind:open={creatingPoll} />

<GiphySelector
    bind:this={giphySelector}
    bind:open={selectingGif}
    on:sendGiphy={sendMessageWithContent} />

{#if creatingCryptoTransfer !== undefined}
    <CryptoTransferBuilder
        {chat}
        ledger={creatingCryptoTransfer.ledger}
        draftAmount={creatingCryptoTransfer.amount}
        defaultReceiver={defaultCryptoTransferReceiver()}
        on:sendTransfer={sendMessageWithContent}
        on:upgrade
        on:close={() => (creatingCryptoTransfer = undefined)} />
{/if}

<ThreadHeader
    {threadRootMessageIndex}
    on:createPoll={createPoll}
    on:closeThread
    {rootEvent}
    chatSummary={chat} />

<ChatEventList
    selectedMessageContext={$selectedMessageContext}
    threadRootEvent={rootEvent}
    rootSelector={"thread-messages"}
    maintainScroll={false}
    bind:this={chatEventList}
    {readonly}
    unreadMessages={0}
    firstUnreadMention={undefined}
    setFocusMessageIndex={(idx) => client.setFocusThreadMessageIndex(chat.id, idx)}
    footer
    {events}
    {chat}
    bind:initialised
    bind:messagesDiv
    bind:messagesDivHeight>
    {#if loading}
        <Loading />
    {:else}
        {#each messages as dayGroup, _di (dateGroupKey(dayGroup))}
            <div class="day-group">
                <div class="date-label">
                    {client.formatMessageDate(
                        dayGroup[0][0]?.timestamp,
                        $_("today"),
                        $_("yesterday")
                    )}
                </div>
                {#each dayGroup as userGroup}
                    {#each userGroup as evt, i (eventKey(evt))}
                        <ChatEvent
                            chatId={chat.id}
                            chatType={chat.kind}
                            {user}
                            event={evt}
                            first={i === 0}
                            last={i + 1 === userGroup.length}
                            me={evt.event.sender === user.userId}
                            confirmed={isConfirmed($unconfirmed, evt)}
                            failed={isFailed($failedMessagesStore, evt)}
                            readByThem
                            readByMe
                            {observer}
                            focused={evt.event.kind === "message" &&
                                $focusMessageIndex === evt.event.messageIndex}
                            {readonly}
                            {threadRootMessage}
                            pinned={false}
                            supportsEdit={evt.event.messageId !== rootEvent.event.messageId}
                            supportsReply={evt.event.messageId !== rootEvent.event.messageId}
                            canPin={client.canPinMessages(chat.id)}
                            canBlockUser={client.canBlockUsers(chat.id)}
                            canDelete={client.canDeleteOtherUsersMessages(chat.id)}
                            publicGroup={(chat.kind === "group_chat" || chat.kind === "channel") &&
                                chat.public}
                            editing={$editingEvent === evt}
                            {canSend}
                            {canReact}
                            canInvite={false}
                            canReplyInThread={false}
                            collapsed={false}
                            on:chatWith
                            on:goToMessageIndex={onGoToMessageIndex}
                            on:replyPrivatelyTo
                            on:replyTo={replyTo}
                            on:editEvent={() => editEvent(evt)}
                            on:replyTo={replyTo}
                            on:upgrade
                            on:retrySend={retrySend}
                            on:forward />
                    {/each}
                {/each}
            </div>
        {/each}
    {/if}
</ChatEventList>

{#if !readonly}
    <Footer
        {chat}
        fileToAttach={$fileToAttach}
        editingEvent={$editingEvent}
        replyingTo={$replyingTo}
        textContent={$textContent}
        members={$currentChatMembers}
        blockedUsers={$currentChatBlockedUsers}
        {user}
        joining={undefined}
        preview={false}
        mode={"thread"}
        {blocked}
        on:joinGroup
        on:cancelPreview
        on:upgrade
        on:cancelReply={cancelReply}
        on:clearAttachment={clearAttachment}
        on:cancelEditEvent={cancelEditEvent}
        on:setTextContent={setTextContent}
        on:startTyping={onStartTyping}
        on:stopTyping={onStopTyping}
        on:fileSelected={fileSelected}
        on:audioCaptured={fileSelected}
        on:sendMessage={sendMessage}
        on:attachGif={attachGif}
        on:tokenTransfer={tokenTransfer}
        on:createTestMessages={createTestMessages}
        on:createPoll={createPoll} />
{/if}

<style lang="scss">
    .day-group {
        position: relative;

        .date-label {
            padding: $sp2;
            background-color: var(--currentChat-date-bg);
            position: sticky;
            top: 0;
            width: 200px;
            margin: auto;
            border-radius: $sp4;
            @include z-index("date-label");
            @include font(book, normal, fs-70);
            text-align: center;
            margin-bottom: $sp4;
        }
    }
</style>
