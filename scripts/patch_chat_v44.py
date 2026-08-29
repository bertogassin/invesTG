#!/usr/bin/env python3
import re, sys, shutil, datetime

PATH = "static/chat-v2.js"

def fail(msg):
    print("PATCH_ABORTED: " + msg)
    sys.exit(1)

with open(PATH, "r", encoding="utf-8") as f:
    src = f.read()
original_src = src

def extract_block(s, sig):
    m = re.search(sig, s)
    if not m:
        return None
    start = m.start()
    brace_open = s.index("{", m.end() - 1)
    depth, i = 0, brace_open
    while i < len(s):
        if s[i] == "{":
            depth += 1
        elif s[i] == "}":
            depth -= 1
            if depth == 0:
                return start, i + 1
        i += 1
    return None

flush_sig = r"async function flushPendingQueue\(\)\s*\{"
if len(re.findall(flush_sig, src)) != 1:
    fail("flushPendingQueue signature not found exactly once")
if not extract_block(src, flush_sig):
    fail("flushPendingQueue braces unbalanced")

error_anchor = r'if \(item\.state === "error"\) \{'
error_matches = list(re.finditer(error_anchor, src))
if len(error_matches) != 1:
    fail('error-state anchor found %d times' % len(error_matches))

reset_anchor = re.compile(
    r'pendingQueue\.forEach\(function \(item\) \{\s*'
    r'if \(item\.state === "sending"\) \{\s*'
    r'item\.state = "queued";\s*\}\s*\}\);'
)
if len(list(reset_anchor.finditer(src))) != 1:
    fail("reload-reset block not found exactly once")

item_anchor = re.compile(
    r'(var item = \{\s*clientMessageId:\s*createClientMessageId\(\),.*?state: "queued"\s*\};)',
    re.DOTALL
)
if len(list(item_anchor.finditer(src))) != 1:
    fail("sendMessage() item literal not found exactly once")

print("ALL_ANCHORS_OK")

m = error_matches[0]
src = src[:m.start()] + 'if (item.state === "error" || item.state === "failed") {' + src[m.end():]

m = list(reset_anchor.finditer(src))[0]
new_reset = ('pendingQueue.forEach(function (item) {\n'
             '                if (item.state === "sending" || item.state === "retrying") {\n'
             '                    item.state = "queued";\n'
             '                }\n'
             '            });')
src = src[:m.start()] + new_reset + src[m.end():]

m = list(item_anchor.finditer(src))[0]
old_block = m.group(1)
new_block = re.sub(r'state: "queued"(\s*)\};', r'state: "queued",\1attempts: 0\1};', old_block, count=1)
if new_block == old_block:
    fail("could not inject attempts:0 — manual review needed")
src = src[:m.start()] + new_block + src[m.end():]

span = extract_block(src, flush_sig)
if not span:
    fail("flushPendingQueue vanished after edits B/C")

NEW_FLUSH = '''async function flushPendingQueue() {
        if (sending || navigator.onLine === false) {
            return;
        }
        sending = true;
        updateComposer();
        try {
            var index = 0;
            while (index < pendingQueue.length) {
                var item = pendingQueue[index];
                if (item.state !== "queued") {
                    index += 1;
                    continue;
                }
                item.attempts = (item.attempts || 0) + 1;
                item.state = "sending";
                savePendingQueue();
                renderPendingItem(item);
                sendState.textContent =
                    pendingQueue.length > 1
                        ? "Отправка \\u00b7 осталось " + pendingQueue.length
                        : "Отправка\\u2026";
                try {
                    var data = await fetchJson(
                        "/api/chat/" + otherUserId + "/send",
                        {
                            method: "POST",
                            headers: { "Content-Type": "application/json" },
                            body: JSON.stringify({
                                message: item.message,
                                reply_to_message_id: item.replyToMessageId,
                                client_message_id: item.clientMessageId
                            })
                        }
                    );
                    if (data.message) {
                        data.message.reply_message = item.replyMessage || "";
                        data.message.reply_sender_user_id = item.replySenderUserId || null;
                    }
                    clearItemRetryTimer(item.clientMessageId);
                    removePendingRow(item.clientMessageId);
                    var sentIndex = pendingQueue.indexOf(item);
                    if (sentIndex !== -1) {
                        pendingQueue.splice(sentIndex, 1);
                    }
                    savePendingQueue();
                    if (data.message) {
                        appendMessages([data.message]);
                    }
                    window.dispatchEvent(new CustomEvent("resursmap:chat-message-sent"));
                    setConnection("\\u0412 \\u0441\\u0435\\u0442\\u0438", "is-online");
                    index = 0;
                } catch (error) {
                    var recoverable = isRecoverableError(error);
                    var exhausted = item.attempts >= MAX_AUTO_RETRIES;
                    if (recoverable && !exhausted) {
                        if (error.status === 429 && error.retryAfter > 0) {
                            sendState.textContent =
                                "\\u041b\\u0438\\u043c\\u0438\\u0442 \\u00b7 \\u043f\\u043e\\u0432\\u0442\\u043e\\u0440 \\u0447\\u0435\\u0440\\u0435\\u0437 " + error.retryAfter + " \\u0441\\u0435\\u043a.";
                            scheduleRetryWithDelay(item, error.retryAfter * 1000);
                        } else {
                            sendState.textContent = "\\u041f\\u043e\\u0432\\u0442\\u043e\\u0440 \\u0447\\u0435\\u0440\\u0435\\u0437 \\u043d\\u0435\\u0441\\u043a\\u043e\\u043b\\u044c\\u043a\\u043e \\u0441\\u0435\\u043a\\u0443\\u043d\\u0434\\u2026";
                            scheduleRetry(item);
                        }
                        setConnection("\\u041e\\u0448\\u0438\\u0431\\u043a\\u0430 \\u043e\\u0442\\u043f\\u0440\\u0430\\u0432\\u043a\\u0438", "is-error");
                    } else {
                        item.state = "failed";
                        savePendingQueue();
                        renderPendingItem(item);
                        if (error.status === 401) {
                            sendState.textContent = "\\u0421\\u0435\\u0441\\u0441\\u0438\\u044f \\u0438\\u0441\\u0442\\u0435\\u043a\\u043b\\u0430";
                        } else {
                            sendState.textContent = "\\u041d\\u0435 \\u043e\\u0442\\u043f\\u0440\\u0430\\u0432\\u043b\\u0435\\u043d\\u043e \\u00b7 \\u043d\\u0430\\u0436\\u043c\\u0438\\u0442\\u0435 !";
                        }
                        setConnection("\\u041e\\u0448\\u0438\\u0431\\u043a\\u0430 \\u043e\\u0442\\u043f\\u0440\\u0430\\u0432\\u043a\\u0438", "is-error");
                    }
                    index += 1;
                }
            }
            if (pendingQueue.length === 0) {
                sendState.textContent = "\\u041e\\u0442\\u043f\\u0440\\u0430\\u0432\\u043b\\u0435\\u043d\\u043e \\u00b7 Enter \\u2014 \\u043e\\u0442\\u043f\\u0440\\u0430\\u0432\\u0438\\u0442\\u044c";
            }
        } finally {
            sending = false;
            updateComposer();
        }
    }'''

src = src[:span[0]] + NEW_FLUSH + src[span[1]:]

anchor = re.search(flush_sig, src)
if not anchor:
    fail("flushPendingQueue not found after replacement")

HELPERS = '''var MAX_AUTO_RETRIES = 5;
        var RETRY_BASE_DELAY_MS = 1000;
        var RETRY_MAX_DELAY_MS = 20000;
        var retryTimers = {};

        function clearItemRetryTimer(clientMessageId) {
            if (retryTimers[clientMessageId]) {
                window.clearTimeout(retryTimers[clientMessageId]);
                delete retryTimers[clientMessageId];
            }
        }

        function isRecoverableError(error) {
            if (!error || typeof error.status !== "number") {
                return true;
            }
            if (error.status === 429) {
                return true;
            }
            return error.status >= 500;
        }

        function scheduleRetryWithDelay(item, delayMs) {
            clearItemRetryTimer(item.clientMessageId);
            item.state = "retrying";
            savePendingQueue();
            renderPendingItem(item);
            retryTimers[item.clientMessageId] = window.setTimeout(function () {
                delete retryTimers[item.clientMessageId];
                item.state = "queued";
                savePendingQueue();
                renderPendingItem(item);
                flushPendingQueue();
            }, delayMs);
        }

        function scheduleRetry(item) {
            var delay = Math.min(
                RETRY_BASE_DELAY_MS * Math.pow(2, item.attempts),
                RETRY_MAX_DELAY_MS
            );
            delay = delay + Math.floor(Math.random() * 300);
            scheduleRetryWithDelay(item, delay);
        }

        '''

src = src[:anchor.start()] + HELPERS + src[anchor.start():]

if src == original_src:
    fail("no changes produced — refusing to write")

backup = PATH + ".before-v44-" + datetime.datetime.utcnow().strftime("%Y%m%dT%H%M%SZ")
shutil.copyfile(PATH, backup)
with open(PATH, "w", encoding="utf-8") as f:
    f.write(src)

print("PATCH_OK")
print("BACKUP=" + backup)
