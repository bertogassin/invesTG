// Переключатель темы ResursMap — работает в любом месте
(function() {
    function themeColorMeta() {
        return document.querySelector('meta[name="theme-color"]');
    }

    function setButtonLabels(isLight) {
        var label = isLight ? "Тёмная тема" : "Светлая тема";
        var labels = document.querySelectorAll(".theme-toggle-label");
        if (labels.length) {
            for (var i = 0; i < labels.length; i++) {
                labels[i].textContent = label;
            }
            return;
        }

        var allBtns = document.querySelectorAll(".theme-toggle-btn");
        for (var j = 0; j < allBtns.length; j++) {
            allBtns[j].textContent = label;
        }
    }

    function applyTheme(isLight) {
        document.documentElement.classList.toggle("light-theme", isLight);
        document.body.classList.toggle("light-theme", isLight);
        document.documentElement.style.colorScheme = isLight ? "light" : "dark";

        var meta = themeColorMeta();
        if (meta) {
            meta.setAttribute("content", isLight ? "#f4f1ea" : "#080a0d");
        }

        setButtonLabels(isLight);
    }

    function savedIsLight() {
        try {
            return localStorage.getItem("resursmap-theme") === "light";
        } catch (e) {
            return false;
        }
    }

    applyTheme(savedIsLight());

    document.addEventListener("click", function(event) {
        var btn = event.target.closest(".theme-toggle-btn");
        if (!btn) return;

        event.preventDefault();

        var isLight = !document.body.classList.contains("light-theme");

        try {
            localStorage.setItem("resursmap-theme", isLight ? "light" : "dark");
        } catch (e) {}

        applyTheme(isLight);
    });

    document.addEventListener("DOMContentLoaded", function() {
        applyTheme(savedIsLight());
    });
})();
