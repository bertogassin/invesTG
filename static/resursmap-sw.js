"use strict";

const CACHE_VERSION = "resursmap-shell-v4.9.31";

const STATIC_ASSETS = [
    "/static/manifest.webmanifest",
    "/static/app-icon.svg",
    "/static/nav-badge.js",
];

self.addEventListener("install", function (event) {
    event.waitUntil(
        caches.open(CACHE_VERSION)
            .then(function (cache) {
                return cache.addAll(STATIC_ASSETS);
            })
            .then(function () {
                return self.skipWaiting();
            })
    );
});

self.addEventListener("activate", function (event) {
    event.waitUntil(
        caches.keys()
            .then(function (keys) {
                return Promise.all(
                    keys
                        .filter(function (key) {
                            return (
                                key.startsWith(
                                    "resursmap-shell-"
                                ) &&
                                key !== CACHE_VERSION
                            );
                        })
                        .map(function (key) {
                            return caches.delete(key);
                        })
                );
            })
            .then(function () {
                return self.clients.claim();
            })
    );
});

self.addEventListener("fetch", function (event) {
    const request = event.request;
    const url = new URL(request.url);

    if (
        request.method !== "GET" ||
        url.origin !== self.location.origin ||
        url.pathname.startsWith("/api/") ||
        url.pathname.startsWith("/app/")
    ) {
        return;
    }

    if (url.pathname.startsWith("/static/")) {
        event.respondWith(
            fetch(request).catch(function () {
                return caches.match(request, {
                    ignoreSearch: true
                });
            })
        );
    }
});
