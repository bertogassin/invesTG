(function () {
  "use strict";

  function qs(sel, root) { return (root || document).querySelector(sel); }
  function qsa(sel, root) { return Array.from((root || document).querySelectorAll(sel)); }

  async function fetchJson(url, options) {
    var response = await fetch(url, Object.assign({
      credentials: "same-origin",
      cache: "no-store",
      headers: { "Accept": "application/json" }
    }, options || {}));
    var data = {};
    try { data = await response.json(); } catch (_) {}
    if (!response.ok || data.ok === false) {
      var err = new Error(data.error || ("HTTP " + response.status));
      err.status = response.status;
      err.code = data.error || "";
      throw err;
    }
    return data;
  }

  function fmtTime(ts) {
    if (!ts) return "";
    try {
      return new Intl.DateTimeFormat("ru", {hour:"2-digit", minute:"2-digit"}).format(new Date(Number(ts) * 1000));
    } catch (_) { return ""; }
  }

  function uuid() {
    if (crypto && crypto.randomUUID) return crypto.randomUUID();
    return String(Date.now()) + "-" + Math.random().toString(36).slice(2) + "-msg";
  }

  function toast(root, text) {
    var el = qs("[data-chat-toast]", root);
    if (!el) return;
    el.textContent = text;
    el.hidden = false;
    clearTimeout(el._t);
    el._t = setTimeout(function(){ el.hidden = true; }, 2600);
  }

  function initInbox(root) {
    var input = qs("[data-inbox-search]", root);
    if (input) {
      input.addEventListener("input", function () {
        var needle = input.value.trim().toLowerCase();
        qsa(".mn-thread", root).forEach(function (row) {
          row.hidden = needle && !row.textContent.toLowerCase().includes(needle);
        });
      });
    }
    var refresh = qs("[data-refresh-inbox]", root);
    if (refresh) refresh.addEventListener("click", function(){ location.reload(); });
    qsa("time[data-unix]", root).forEach(function (el) {
      el.textContent = fmtTime(el.dataset.unix);
    });
  }

  function initChat(root) {
    var peerId = String(root.dataset.peerId || "");
    var list = qs("[data-message-list]", root);
    var input = qs("[data-chat-input]", root);
    var form = qs("[data-chat-form]", root);
    var sendButton = qs("[data-send-button]", root);
    var loading = qs("[data-chat-loading]", root);
    var jump = qs("[data-jump-bottom]", root);
    var peerStatus = qs("[data-peer-status]", root);
    var replyStrip = qs("[data-reply-strip]", root);
    var replyText = qs("[data-reply-text]", root);
    var replyAuthor = qs("[data-reply-author]", root);
    var imageInput = qs("[data-image-input]", root);
    var emojiPanel = qs("[data-emoji-panel]", root);

    var lastId = 0;
    var firstId = 0;
    var syncing = false;
    var initialLoaded = false;
    var socket = null;
    var reconnectTimer = null;
    var reply = null;
    var pending = new Map();

    function nearBottom() {
      return list.scrollHeight - list.scrollTop - list.clientHeight < 140;
    }
    function toBottom(smooth) {
      list.scrollTo({ top: list.scrollHeight, behavior: smooth ? "smooth" : "auto" });
    }
    function updateJump() { if (jump) jump.hidden = nearBottom(); }

    function setReply(message) {
      reply = message || null;
      if (!reply) {
        replyStrip.hidden = true;
        return;
      }
      replyAuthor.textContent = reply.is_mine ? "Ответ себе" : "Ответ";
      replyText.textContent = reply.deleted_at ? "Сообщение удалено" : (reply.message || "Вложение");
      replyStrip.hidden = false;
      input.focus();
    }

    function renderReactions(row, message) {
      var box = qs(".mn-reactions", row);
      box.textContent = "";
      (message.reactions || []).forEach(function (r) {
        var b = document.createElement("button");
        b.type = "button";
        b.className = "mn-reaction" + (r.mine ? " is-mine" : "");
        b.textContent = r.emoji + (r.count > 1 ? " " + r.count : "");
        b.addEventListener("click", function(){ react(message.id, r.emoji); });
        box.appendChild(b);
      });
    }

    function createRow(message) {
      var row = document.createElement("article");
      row.className = "mn-message " + (message.is_mine ? "is-mine" : "is-peer");
      row.dataset.messageId = String(message.id);

      var bubble = document.createElement("div");
      bubble.className = "mn-bubble";

      if (message.reply_to_message_id) {
        var quote = document.createElement("div");
        quote.className = "mn-quote";
        quote.textContent = message.reply_message || "Сообщение";
        bubble.appendChild(quote);
      }

      if (message.deleted_at) {
        var deleted = document.createElement("div");
        deleted.className = "mn-deleted";
        deleted.textContent = "Сообщение удалено";
        bubble.appendChild(deleted);
      } else if (message.attachment_kind === "image" && message.attachment_url) {
        var img = document.createElement("img");
        img.className = "mn-image";
        img.loading = "lazy";
        img.src = message.attachment_url;
        img.alt = "Фото";
        bubble.appendChild(img);
        if (message.message) {
          var cap = document.createElement("div");
          cap.className = "mn-text";
          cap.textContent = message.message;
          bubble.appendChild(cap);
        }
      } else if (message.attachment_kind === "voice" && message.attachment_url) {
        var audio = document.createElement("audio");
        audio.controls = true;
        audio.preload = "metadata";
        audio.src = message.attachment_url;
        bubble.appendChild(audio);
      } else {
        var text = document.createElement("div");
        text.className = "mn-text";
        text.textContent = message.message || "";
        bubble.appendChild(text);
      }

      var meta = document.createElement("div");
      meta.className = "mn-meta";
      var edited = message.edited_at ? " · изм." : "";
      var ticks = message.is_mine ? (message.read_at ? " ✓✓" : (message.delivered_at ? " ✓✓" : " ✓")) : "";
      meta.textContent = fmtTime(message.created_at) + edited + ticks;
      bubble.appendChild(meta);

      var reactions = document.createElement("div");
      reactions.className = "mn-reactions";
      bubble.appendChild(reactions);

      var actions = document.createElement("div");
      actions.className = "mn-message-actions";

      var replyBtn = document.createElement("button");
      replyBtn.type = "button";
      replyBtn.textContent = "↩";
      replyBtn.title = "Ответить";
      replyBtn.addEventListener("click", function(){ setReply(message); });
      actions.appendChild(replyBtn);

      ["👍","❤️","😂","🔥"].forEach(function (emoji) {
        var r = document.createElement("button");
        r.type = "button";
        r.textContent = emoji;
        r.addEventListener("click", function(){ react(message.id, emoji); });
        actions.appendChild(r);
      });

      if (message.is_mine && !message.deleted_at) {
        if (!message.attachment_kind) {
          var edit = document.createElement("button");
          edit.type = "button";
          edit.textContent = "✎";
          edit.title = "Изменить";
          edit.addEventListener("click", function(){ editMessage(message); });
          actions.appendChild(edit);
        }
        var del = document.createElement("button");
        del.type = "button";
        del.textContent = "⌫";
        del.title = "Удалить";
        del.addEventListener("click", function(){ deleteMessage(message.id); });
        actions.appendChild(del);
      }

      bubble.appendChild(actions);
      row.appendChild(bubble);
      renderReactions(row, message);
      return row;
    }

    function upsert(message) {
      var existing = qs('[data-message-id="' + message.id + '"]', list);
      var fresh = createRow(message);
      if (existing) existing.replaceWith(fresh);
      else list.appendChild(fresh);
      lastId = Math.max(lastId, Number(message.id) || 0);
      firstId = firstId ? Math.min(firstId, Number(message.id) || firstId) : (Number(message.id) || 0);
    }

    async function sync(forceRead) {
      if (syncing || document.visibilityState === "hidden") return;
      syncing = true;
      var stick = nearBottom() || !initialLoaded;
      try {
        var data = await fetchJson(
          "/api/chat/" + peerId + "/messages?after_id=" + lastId +
          "&limit=120&mark_read=" + ((forceRead || nearBottom()) ? "true" : "false")
        );
        (data.messages || []).forEach(upsert);
        if (loading) loading.remove();
        initialLoaded = true;
        if (stick && (data.messages || []).length) toBottom(false);
        updateMineStatuses(Number(data.peer_read_through_id || 0));
        peerStatus.textContent = "в сети";
      } catch (e) {
        peerStatus.textContent = e.status === 401 ? "нужен вход" : "переподключение…";
      } finally {
        syncing = false;
        updateJump();
      }
    }

    function updateMineStatuses(readThrough) {
      if (!readThrough) return;
      qsa(".mn-message.is-mine", list).forEach(function(row){
        var id = Number(row.dataset.messageId || 0);
        if (id <= readThrough) {
          var meta = qs(".mn-meta", row);
          if (meta && !meta.textContent.includes("✓✓")) meta.textContent += " ✓✓";
        }
      });
    }

    async function sendText() {
      var text = input.value.trim();
      if (!text) return;
      var clientId = uuid();
      sendButton.disabled = true;
      try {
        var data = await fetchJson("/api/chat/" + peerId + "/send", {
          method: "POST",
          headers: {"Content-Type":"application/json","Accept":"application/json"},
          body: JSON.stringify({
            message: text,
            reply_to_message_id: reply ? reply.id : null,
            client_message_id: clientId
          })
        });
        input.value = "";
        autoGrow();
        setReply(null);
        if (data.message) upsert(data.message);
        toBottom(true);
      } catch (e) {
        toast(root, e.code === "user_blocked" ? "Диалог заблокирован" : "Не удалось отправить. Повторите.");
      } finally {
        sendButton.disabled = false;
        input.focus();
      }
    }

    async function react(id, emoji) {
      try {
        var data = await fetchJson("/api/chat/" + peerId + "/messages/" + id + "/react", {
          method:"POST",
          headers:{"Content-Type":"application/json","Accept":"application/json"},
          body:JSON.stringify({emoji:emoji})
        });
        if (data.message) upsert(data.message);
      } catch (_) { toast(root, "Не удалось поставить реакцию"); }
    }

    async function editMessage(message) {
      var next = prompt("Изменить сообщение:", message.message || "");
      if (next === null) return;
      next = next.trim();
      if (!next) return;
      try {
        var data = await fetchJson("/api/chat/" + peerId + "/messages/" + message.id + "/edit", {
          method:"POST",
          headers:{"Content-Type":"application/json","Accept":"application/json"},
          body:JSON.stringify({message:next})
        });
        if (data.message) upsert(data.message);
      } catch (_) { toast(root, "Не удалось изменить сообщение"); }
    }

    async function deleteMessage(id) {
      if (!confirm("Удалить сообщение?")) return;
      try {
        var data = await fetchJson("/api/chat/" + peerId + "/messages/" + id + "/delete", {
          method:"POST",
          headers:{"Accept":"application/json"}
        });
        if (data.message) upsert(data.message);
      } catch (_) { toast(root, "Не удалось удалить сообщение"); }
    }

    async function uploadImage(file) {
      if (!file) return;
      var fd = new FormData();
      fd.append("image", file, file.name || "image");
      fd.append("client_message_id", uuid());
      if (reply) fd.append("reply_to_message_id", String(reply.id));
      try {
        sendButton.disabled = true;
        await fetchJson("/api/chat/" + peerId + "/send-image", {method:"POST", body:fd, headers:{"Accept":"application/json"}});
        setReply(null);
        await sync(true);
        toBottom(true);
      } catch (e) {
        toast(root, e.code === "file_too_large" ? "Фото слишком большое" : "Не удалось отправить фото");
      } finally {
        sendButton.disabled = false;
        imageInput.value = "";
      }
    }

    function autoGrow() {
      input.style.height = "auto";
      input.style.height = Math.min(input.scrollHeight, 144) + "px";
    }

    function connect() {
      clearTimeout(reconnectTimer);
      try {
        var proto = location.protocol === "https:" ? "wss:" : "ws:";
        socket = new WebSocket(proto + "//" + location.host + "/api/chat/realtime");
      } catch (_) {
        reconnectTimer = setTimeout(connect, 2500);
        return;
      }
      socket.addEventListener("open", function(){ peerStatus.textContent = "в сети"; });
      socket.addEventListener("message", function(ev){
        try {
          var data = JSON.parse(ev.data);
          if (data.type === "sync" || data.type === "ready") sync(false);
        } catch (_) {}
      });
      socket.addEventListener("close", function(){
        peerStatus.textContent = "переподключение…";
        reconnectTimer = setTimeout(connect, 1800);
      });
      socket.addEventListener("error", function(){ try { socket.close(); } catch (_) {} });
    }

    form.addEventListener("submit", function(e){ e.preventDefault(); sendText(); });
    input.addEventListener("input", autoGrow);
    input.addEventListener("keydown", function(e){
      if (e.key === "Enter" && !e.shiftKey && !e.isComposing) {
        e.preventDefault();
        sendText();
      }
    });
    list.addEventListener("scroll", function(){
      updateJump();
      if (nearBottom()) sync(true);
    }, {passive:true});
    jump.addEventListener("click", function(){ toBottom(true); sync(true); });
    qs("[data-reply-cancel]", root).addEventListener("click", function(){ setReply(null); });
    imageInput.addEventListener("change", function(){ uploadImage(imageInput.files && imageInput.files[0]); });

    var emojiToggle = qs("[data-emoji-toggle]", root);
    emojiToggle.addEventListener("click", function(){ emojiPanel.hidden = !emojiPanel.hidden; });
    qsa("button", emojiPanel).forEach(function(btn){
      btn.addEventListener("click", function(){
        input.setRangeText(btn.textContent, input.selectionStart, input.selectionEnd, "end");
        emojiPanel.hidden = true;
        input.focus();
        autoGrow();
      });
    });

    document.addEventListener("visibilitychange", function(){ if (!document.hidden) sync(true); });
    window.addEventListener("online", function(){ connect(); sync(true); });

    autoGrow();
    sync(true);
    connect();
    setInterval(function(){ sync(false); }, 3500);
  }

  document.addEventListener("DOMContentLoaded", function () {
    var inbox = qs("[data-messenger-inbox]");
    if (inbox) initInbox(inbox);
    var chat = qs("[data-messenger-chat]");
    if (chat) initChat(chat);
  });
})();
