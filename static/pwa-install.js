(() => {
  let deferredPrompt = null;

  const ready = () => {
    const androidButton =
      document.getElementById("resursmap-install-android");

    const iosButton =
      document.getElementById("resursmap-install-ios");

    const hint =
      document.getElementById("resursmap-install-hint");

    const isIOS =
      /iphone|ipad|ipod/i.test(navigator.userAgent);

    const isStandalone =
      window.matchMedia("(display-mode: standalone)").matches ||
      window.navigator.standalone === true;

    if (isStandalone) {
      if (hint) {
        hint.textContent = "ResursMap уже установлен на этом устройстве.";
      }

      if (androidButton) androidButton.disabled = true;
      if (iosButton) iosButton.disabled = true;
    }

    window.addEventListener("beforeinstallprompt", (event) => {
      event.preventDefault();
      deferredPrompt = event;
    });

    if (androidButton) {
      androidButton.addEventListener("click", async () => {
        if (isStandalone) return;

        if (deferredPrompt) {
          deferredPrompt.prompt();
          await deferredPrompt.userChoice;
          deferredPrompt = null;
          return;
        }

        alert("Установка недоступна в этом браузере.");
      });
    }

    if (iosButton) {
      iosButton.addEventListener("click", () => {
        if (isStandalone) return;

        alert(
          "На iPhone откройте ResursMap в Safari.\n\n" +
          "1. Нажмите «Поделиться».\n" +
          "2. Выберите «На экран Домой».\n" +
          "3. Нажмите «Добавить»."
        );
      });
    }

    if (isIOS && hint && !isStandalone) {
      hint.textContent =
        "Android — установка приложения. iPhone — через Safari.";
    }

    const splash = document.getElementById("resursmap-splash");

    if (splash) {
      window.setTimeout(() => {
        splash.style.opacity = "0";
        splash.style.visibility = "hidden";

        window.setTimeout(() => {
          splash.remove();
        }, 500);
      }, 900);
    }
  };

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", ready);
  } else {
    ready();
  }
})();
