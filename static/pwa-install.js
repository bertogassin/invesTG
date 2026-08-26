(() => {
  let deferredPrompt = null;

  const button = document.getElementById("resursmap-install-app");
  const hint = document.getElementById("resursmap-install-hint");

  if (!button) return;

  const ua = navigator.userAgent || "";
  const isIOS = /iphone|ipad|ipod/i.test(ua);
  const isStandalone =
    window.matchMedia("(display-mode: standalone)").matches ||
    window.navigator.standalone === true;

  if (isStandalone) {
    button.textContent = "✓ Приложение установлено";
    button.disabled = true;
    if (hint) hint.textContent = "ResursMap уже открыт как приложение.";
    return;
  }

  window.addEventListener("beforeinstallprompt", (event) => {
    event.preventDefault();
    deferredPrompt = event;
    button.hidden = false;
    button.textContent = "↓ Установить ResursMap";
  });

  button.addEventListener("click", async () => {
    if (deferredPrompt) {
      deferredPrompt.prompt();
      await deferredPrompt.userChoice;
      deferredPrompt = null;
      return;
    }

    if (isIOS) {
      alert(
        "Чтобы установить ResursMap на iPhone:\n\n" +
        "1. Нажмите кнопку «Поделиться» в Safari.\n" +
        "2. Выберите «На экран Домой».\n" +
        "3. Нажмите «Добавить»."
      );
      return;
    }

    alert(
      "Откройте меню браузера и выберите «Установить приложение» " +
      "или «Добавить на главный экран»."
    );
  });

  if (isIOS) {
    button.hidden = false;
    button.textContent = "↓ Установить на iPhone";
    if (hint) {
      hint.textContent = "Установка через Safari → Поделиться → На экран Домой.";
    }
  } else {
    button.hidden = false;
  }
})();
