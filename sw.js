const CACHE_NAME = 'oxidize-v33';

// Install: skip waiting to activate immediately
self.addEventListener('install', event => {
    self.skipWaiting();
});

// Activate: claim all clients and clean old caches
self.addEventListener('activate', event => {
    event.waitUntil(
        caches.keys().then(keys => {
            return Promise.all(
                keys.filter(key => key !== CACHE_NAME)
                    .map(key => caches.delete(key))
            );
        }).then(() => self.clients.claim())
    );
});

// Fetch: cache-first strategy with network fallback
self.addEventListener('fetch', event => {
    // Only handle GET requests
    if (event.request.method !== 'GET') return;
    
    // Skip cross-origin requests (like fonts)
    if (!event.request.url.startsWith(self.location.origin)) {
        // But still try to cache fonts
        if (event.request.url.includes('fonts.googleapis.com') || 
            event.request.url.includes('fonts.gstatic.com')) {
            event.respondWith(
                caches.match(event.request).then(cached => {
                    return cached || fetch(event.request).then(response => {
                        const clone = response.clone();
                        caches.open(CACHE_NAME).then(cache => cache.put(event.request, clone));
                        return response;
                    });
                })
            );
        }
        return;
    }
    
    event.respondWith(
        caches.match(event.request).then(cached => {
            if (cached) {
                return cached;
            }
            
            return fetch(event.request).then(response => {
                // Don't cache non-OK responses
                if (!response.ok) {
                    return response;
                }
                
                // Cache the response
                const clone = response.clone();
                caches.open(CACHE_NAME).then(cache => {
                    cache.put(event.request, clone);
                });
                
                return response;
            });
        }).catch(() => {
            // Offline fallback - serve index.html for navigation requests
            if (event.request.mode === 'navigate') {
                return caches.match('./index.html') || caches.match('index.html');
            }
            return new Response('Offline', { status: 503 });
        })
    );
});
