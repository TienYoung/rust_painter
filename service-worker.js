self.addEventListener('install', (event) => {
  event.waitUntil(
    caches.open('rust-painter-cache').then((cache) => {
      return cache.addAll([
        '/',
        '/index.html',
        '/styles.css',
        '/main.js',
        // Add other assets that need to be cached
      ]);
    })
  );
});

self.addEventListener('fetch', (event) => {
  event.respondWith(
    caches.match(event.request).then((response) => {
      return response || fetch(event.request);
    })
  );
});
