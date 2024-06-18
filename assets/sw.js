var cacheName = 'egui-template-pwa';
var filesToCache = [
  './',
  './index.html#dev',
  './cebra_efficiency.js',
  './cebra_efficiency_bg.wasm',
  '../etc/REU_2023.yaml'
];

/* Start the service worker and cache all of the app's content */
self.addEventListener('install', function (e) {
  e.waitUntil(
    caches.open(cacheName).then(function (cache) {
      return cache.addAll(filesToCache);
    })
  );
});

/* Serve cached content when offline */
self.addEventListener('fetch', function (e) {
  e.respondWith(
    caches.match(e.request).then(function (response) {
      return response || fetch(e.request);
    })
  );
});
