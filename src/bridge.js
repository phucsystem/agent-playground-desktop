(function () {
  const MAX_RETRIES = 50;
  const RETRY_INTERVAL_MS = 100;

  function initBridge() {
    const invoke = window.__TAURI__.core.invoke;

    (async function requestNotificationPermission() {
      try {
        const granted = await invoke("plugin:notification|is_permission_granted");
        if (!granted) {
          await invoke("plugin:notification|request_permission");
        }
      } catch (error) {
        console.warn("[Agent Playground Desktop] Notification permission check failed:", error);
      }
    })();

    window.addEventListener("tauri:new-message", function (event) {
      const detail = event.detail || {};
      invoke("notify_new_message", {
        payload: {
          sender_name: detail.sender || "Unknown",
          message_text: detail.text || "",
          conversation_id: detail.conversationId || "",
          conversation_name: detail.conversationName || null,
          is_group: detail.isGroup || false,
          force: detail.force || false,
        },
      });
    });

    window.addEventListener("tauri:unread-count", function (event) {
      const detail = event.detail || {};
      invoke("update_badge_count", { count: detail.count || 0 });
    });

    window.addEventListener("tauri:active-conversation", function (event) {
      const detail = event.detail || {};
      invoke("report_user_active", {
        conversation_id: detail.conversationId || null,
      });
    });

    document.addEventListener("visibilitychange", function () {
      if (document.hidden) {
        invoke("report_user_active", { conversation_id: null });
      }
    });

    window.addEventListener("focus", function () {
      invoke("update_badge_count", { count: 0 });
    });

    var notificationCounter = 0;
    var OriginalNotification = window.Notification;
    window.Notification = function (title, options) {
      notificationCounter++;
      var opts = options || {};
      invoke("notify_new_message", {
        payload: {
          sender_name: title || "Unknown",
          message_text: opts.body || "",
          conversation_id: opts.tag || "notif-" + notificationCounter,
          conversation_name: null,
          is_group: false,
          force: true,
        },
      });
      return { close: function () {} };
    };
    window.Notification.permission = "granted";
    window.Notification.requestPermission = function (callback) {
      if (callback) callback("granted");
      return Promise.resolve("granted");
    };

    console.log("[Agent Playground Desktop] Bridge loaded (Notification override active)");
  }

  function waitForTauri(attempt) {
    if (window.__TAURI__) {
      initBridge();
      return;
    }
    if (attempt >= MAX_RETRIES) {
      console.warn("[Agent Playground Desktop] __TAURI__ not available after " + (MAX_RETRIES * RETRY_INTERVAL_MS) + "ms, bridge not loaded");
      return;
    }
    setTimeout(function () { waitForTauri(attempt + 1); }, RETRY_INTERVAL_MS);
  }

  waitForTauri(0);
})();
