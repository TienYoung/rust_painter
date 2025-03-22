const CACHE_NAME = "rust-painter-cache-v1";
const ASSETS = [
  "/",
  "/icon-192x192.png",
  "/icon-512x512.png",
  "/index.html",
  "/rust_painter_bg.wasm",
  "/rust_painter_bg.wasm.d.ts",
  "/rust_painter.d.ts",
  "/rust_painter.js",
  "/manifest.json",
  "/assets/models/barycentric.gltf",
  "/assets/shaders/custom_gltf_2d.wgsl"
];

self.addEventListener("install", event => {
  event.waitUntil(
    caches.open(CACHE_NAME).then(cache => cache.addAll(ASSETS))
  );
});

self.addEventListener("fetch", event => {
  event.respondWith(
    caches.match(event.request).then(response =>
      response || fetch(event.request)
    )
  );
});