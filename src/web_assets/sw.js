// QMS Web Interface - Medical Device Quality Management System
// Progressive Web App Service Worker for Offline Support
// Regulatory Compliance: FDA 21 CFR Part 820, ISO 13485

const CACHE_NAME = 'qms-v1.0.0';
const CACHE_STATIC_NAME = 'qms-static-v1.0.0';
const CACHE_DYNAMIC_NAME = 'qms-dynamic-v1.0.0';

// Assets to cache for offline functionality
const STATIC_ASSETS = [
    '/',
    '/styles.css',
    '/manifest.json',
    '/favicon.ico'
];

// API endpoints that should be cached with network-first strategy
const API_ENDPOINTS = [
    '/api/health',
    '/api/system/status',
    '/api/audit/summary',
    '/api/compliance/badges'
];

// Cache strategies
const CACHE_STRATEGIES = {
    CACHE_FIRST: 'cache-first',
    NETWORK_FIRST: 'network-first',
    CACHE_ONLY: 'cache-only',
    NETWORK_ONLY: 'network-only',
    STALE_WHILE_REVALIDATE: 'stale-while-revalidate'
};

// Medical device audit logging
function logAuditEvent(event, details) {
    const auditEntry = {
        timestamp: new Date().toISOString(),
        event: event,
        details: details,
        source: 'service-worker',
        version: CACHE_NAME
    };
    
    // Store audit entry for later sync
    if ('indexedDB' in self) {
        storeAuditEntry(auditEntry);
    }
}

// Store audit entries in IndexedDB for regulatory compliance
async function storeAuditEntry(entry) {
    try {
        const request = indexedDB.open('qms-audit', 1);
        
        request.onupgradeneeded = function(event) {
            const db = event.target.result;
            if (!db.objectStoreNames.contains('audit_log')) {
                const store = db.createObjectStore('audit_log', { 
                    keyPath: 'id', 
                    autoIncrement: true 
                });
                store.createIndex('timestamp', 'timestamp', { unique: false });
                store.createIndex('event', 'event', { unique: false });
            }
        };
        
        request.onsuccess = function(event) {
            const db = event.target.result;
            const transaction = db.transaction(['audit_log'], 'readwrite');
            const store = transaction.objectStore('audit_log');
            store.add(entry);
        };
    } catch (error) {
        console.error('Failed to store audit entry:', error);
    }
}

// Service Worker Installation
self.addEventListener('install', event => {
    console.log('QMS Service Worker installing...');
    logAuditEvent('sw_install', { version: CACHE_NAME });
    
    event.waitUntil(
        Promise.all([
            // Cache static assets
            caches.open(CACHE_STATIC_NAME).then(cache => {
                console.log('Caching static assets...');
                return cache.addAll(STATIC_ASSETS);
            }),
            
            // Pre-cache critical API endpoints
            caches.open(CACHE_DYNAMIC_NAME).then(cache => {
                console.log('Pre-caching API endpoints...');
                const requests = API_ENDPOINTS.map(url => 
                    fetch(url).then(response => {
                        if (response.ok) {
                            cache.put(url, response.clone());
                        }
                        return response;
                    }).catch(error => {
                        console.warn(`Failed to pre-cache ${url}:`, error);
                        return null;
                    })
                );
                return Promise.allSettled(requests);
            })
        ]).then(() => {
            console.log('QMS Service Worker installation complete');
            logAuditEvent('sw_install_complete', { cached_assets: STATIC_ASSETS.length });
            
            // Skip waiting to activate immediately
            return self.skipWaiting();
        })
    );
});

// Service Worker Activation
self.addEventListener('activate', event => {
    console.log('QMS Service Worker activating...');
    logAuditEvent('sw_activate', { version: CACHE_NAME });
    
    event.waitUntil(
        Promise.all([
            // Clean up old caches
            caches.keys().then(cacheNames => {
                const deletePromises = cacheNames
                    .filter(name => 
                        name.startsWith('qms-') && 
                        name !== CACHE_NAME && 
                        name !== CACHE_STATIC_NAME && 
                        name !== CACHE_DYNAMIC_NAME
                    )
                    .map(name => {
                        console.log('Deleting old cache:', name);
                        logAuditEvent('cache_cleanup', { deleted_cache: name });
                        return caches.delete(name);
                    });
                
                return Promise.all(deletePromises);
            }),
            
            // Take control of all pages immediately
            self.clients.claim()
        ]).then(() => {
            console.log('QMS Service Worker activation complete');
            logAuditEvent('sw_activate_complete', { version: CACHE_NAME });
        })
    );
});

// Fetch Event Handler - Main request interceptor
self.addEventListener('fetch', event => {
    const { request } = event;
    const { url, method } = request;
    
    // Only handle GET requests for caching
    if (method !== 'GET') {
        logAuditEvent('fetch_passthrough', { 
            method: method, 
            url: url,
            reason: 'non-get-request'
        });
        return;
    }
    
    // Skip chrome-extension and other non-http(s) requests
    if (!url.startsWith('http')) {
        return;
    }
    
    event.respondWith(handleFetchRequest(request));
});

// Main fetch request handler with caching strategies
async function handleFetchRequest(request) {
    const url = new URL(request.url);
    const path = url.pathname;
    
    try {
        // Determine caching strategy based on request type
        if (isStaticAsset(path)) {
            return await handleStaticAsset(request);
        } else if (isAPIEndpoint(path)) {
            return await handleAPIRequest(request);
        } else if (isHTMLPage(path)) {
            return await handleHTMLPage(request);
        } else {
            // Default: network first with cache fallback
            return await networkFirstWithFallback(request);
        }
    } catch (error) {
        console.error('Fetch handler error:', error);
        logAuditEvent('fetch_error', { 
            url: request.url, 
            error: error.message 
        });
        
        // Return offline page or cached response if available
        return await getCachedResponseOrOfflinePage(request);
    }
}

// Check if request is for static asset
function isStaticAsset(path) {
    const staticExtensions = ['.css', '.js', '.png', '.jpg', '.jpeg', '.gif', '.ico', '.svg', '.woff', '.woff2'];
    return staticExtensions.some(ext => path.endsWith(ext));
}

// Check if request is for API endpoint
function isAPIEndpoint(path) {
    return path.startsWith('/api/');
}

// Check if request is for HTML page
function isHTMLPage(path) {
    return path === '/' || path.endsWith('.html') || (!path.includes('.'));
}

// Handle static assets with cache-first strategy
async function handleStaticAsset(request) {
    const cachedResponse = await caches.match(request);
    
    if (cachedResponse) {
        // Check if cached response is still fresh (24 hours for static assets)
        const cachedDate = new Date(cachedResponse.headers.get('date'));
        const now = new Date();
        const hoursSinceCached = (now - cachedDate) / (1000 * 60 * 60);
        
        if (hoursSinceCached < 24) {
            logAuditEvent('cache_hit', { url: request.url, strategy: 'static_asset' });
            return cachedResponse;
        }
    }
    
    // Fetch fresh version and update cache
    try {
        const networkResponse = await fetch(request);
        
        if (networkResponse.ok) {
            const cache = await caches.open(CACHE_STATIC_NAME);
            cache.put(request, networkResponse.clone());
            logAuditEvent('cache_update', { url: request.url, strategy: 'static_asset' });
        }
        
        return networkResponse;
    } catch (error) {
        // Return cached version if network fails
        if (cachedResponse) {
            logAuditEvent('cache_fallback', { 
                url: request.url, 
                strategy: 'static_asset',
                reason: 'network_failed'
            });
            return cachedResponse;
        }
        throw error;
    }
}

// Handle API requests with network-first strategy
async function handleAPIRequest(request) {
    try {
        // Always try network first for API requests
        const networkResponse = await fetch(request, {
            // Add timeout for medical device compliance
            signal: AbortSignal.timeout(10000) // 10 second timeout
        });
        
        if (networkResponse.ok) {
            // Cache successful API responses
            const cache = await caches.open(CACHE_DYNAMIC_NAME);
            cache.put(request, networkResponse.clone());
            logAuditEvent('api_success', { 
                url: request.url, 
                status: networkResponse.status 
            });
        }
        
        return networkResponse;
    } catch (error) {
        console.warn('API request failed, trying cache:', error);
        
        // Fallback to cached response
        const cachedResponse = await caches.match(request);
        if (cachedResponse) {
            logAuditEvent('api_cache_fallback', { 
                url: request.url,
                reason: error.message
            });
            
            // Add header to indicate this is from cache
            const headers = new Headers(cachedResponse.headers);
            headers.set('X-QMS-Cache', 'offline-fallback');
            
            return new Response(cachedResponse.body, {
                status: cachedResponse.status,
                statusText: cachedResponse.statusText,
                headers: headers
            });
        }
        
        // No cache available, return error response
        logAuditEvent('api_failure', { 
            url: request.url, 
            error: error.message 
        });
        
        return new Response(
            JSON.stringify({ 
                error: 'Service unavailable - offline mode',
                timestamp: new Date().toISOString(),
                cached: false
            }), 
            {
                status: 503,
                statusText: 'Service Unavailable',
                headers: {
                    'Content-Type': 'application/json',
                    'X-QMS-Cache': 'no-cache-available'
                }
            }
        );
    }
}

// Handle HTML pages with network-first, cache fallback
async function handleHTMLPage(request) {
    try {
        const networkResponse = await fetch(request);
        
        if (networkResponse.ok) {
            // Cache HTML pages
            const cache = await caches.open(CACHE_DYNAMIC_NAME);
            cache.put(request, networkResponse.clone());
            logAuditEvent('page_served', { url: request.url, source: 'network' });
        }
        
        return networkResponse;
    } catch (error) {
        // Fallback to cached page
        const cachedResponse = await caches.match(request);
        if (cachedResponse) {
            logAuditEvent('page_served', { 
                url: request.url, 
                source: 'cache',
                reason: 'network_failed'
            });
            return cachedResponse;
        }
        
        // Return offline page if available
        const offlinePage = await caches.match('/');
        if (offlinePage) {
            logAuditEvent('offline_page_served', { 
                requested_url: request.url,
                served_url: '/'
            });
            return offlinePage;
        }
        
        throw error;
    }
}

// Network first with cache fallback strategy
async function networkFirstWithFallback(request) {
    try {
        const networkResponse = await fetch(request);
        
        if (networkResponse.ok) {
            const cache = await caches.open(CACHE_DYNAMIC_NAME);
            cache.put(request, networkResponse.clone());
        }
        
        return networkResponse;
    } catch (error) {
        const cachedResponse = await caches.match(request);
        if (cachedResponse) {
            return cachedResponse;
        }
        throw error;
    }
}

// Get cached response or return offline page
async function getCachedResponseOrOfflinePage(request) {
    const cachedResponse = await caches.match(request);
    if (cachedResponse) {
        return cachedResponse;
    }
    
    // Return a basic offline response
    return new Response(
        `<!DOCTYPE html>
        <html>
        <head>
            <title>QMS - Offline</title>
            <meta charset="utf-8">
            <meta name="viewport" content="width=device-width, initial-scale=1">
            <style>
                body { 
                    font-family: Arial, sans-serif; 
                    text-align: center; 
                    padding: 2rem;
                    background: #f8f9fa;
                    color: #2c3e50;
                }
                .offline-message {
                    max-width: 500px;
                    margin: 2rem auto;
                    padding: 2rem;
                    background: white;
                    border-radius: 8px;
                    box-shadow: 0 2px 10px rgba(0,0,0,0.1);
                }
                .retry-btn {
                    background: #3498db;
                    color: white;
                    border: none;
                    padding: 0.75rem 1.5rem;
                    border-radius: 4px;
                    cursor: pointer;
                    margin-top: 1rem;
                }
            </style>
        </head>
        <body>
            <div class="offline-message">
                <h1>🔒 QMS - Offline Mode</h1>
                <p>Quality Management System is currently offline.</p>
                <p>This medical device system requires network connectivity for full functionality.</p>
                <p>Please check your connection and try again.</p>
                <button class="retry-btn" onclick="window.location.reload()">Retry Connection</button>
            </div>
        </body>
        </html>`,
        {
            status: 503,
            statusText: 'Service Unavailable',
            headers: {
                'Content-Type': 'text/html',
                'X-QMS-Cache': 'offline-page'
            }
        }
    );
}

// Handle background sync for audit data
self.addEventListener('sync', event => {
    if (event.tag === 'audit-sync') {
        event.waitUntil(syncAuditData());
    }
});

// Sync audit data with server when online
async function syncAuditData() {
    try {
        const request = indexedDB.open('qms-audit', 1);
        
        request.onsuccess = async function(event) {
            const db = event.target.result;
            const transaction = db.transaction(['audit_log'], 'readonly');
            const store = transaction.objectStore('audit_log');
            const getAllRequest = store.getAll();
            
            getAllRequest.onsuccess = async function() {
                const auditEntries = getAllRequest.result;
                
                if (auditEntries.length > 0) {
                    try {
                        const response = await fetch('/api/audit/sync', {
                            method: 'POST',
                            headers: {
                                'Content-Type': 'application/json'
                            },
                            body: JSON.stringify({ entries: auditEntries })
                        });
                        
                        if (response.ok) {
                            // Clear synced entries
                            const clearTransaction = db.transaction(['audit_log'], 'readwrite');
                            const clearStore = clearTransaction.objectStore('audit_log');
                            clearStore.clear();
                            
                            console.log('Audit data synced successfully');
                        }
                    } catch (error) {
                        console.error('Failed to sync audit data:', error);
                    }
                }
            };
        };
    } catch (error) {
        console.error('Failed to access audit database:', error);
    }
}

// Handle push notifications for system alerts
self.addEventListener('push', event => {
    if (!event.data) {
        return;
    }
    
    try {
        const data = event.data.json();
        logAuditEvent('push_received', { type: data.type, priority: data.priority });
        
        const options = {
            body: data.message,
            icon: '/favicon.ico',
            badge: '/favicon.ico',
            tag: data.type,
            requireInteraction: data.priority === 'high',
            data: data
        };
        
        event.waitUntil(
            self.registration.showNotification(data.title, options)
        );
    } catch (error) {
        console.error('Failed to process push notification:', error);
    }
});

// Handle notification clicks
self.addEventListener('notificationclick', event => {
    event.notification.close();
    
    const data = event.notification.data;
    logAuditEvent('notification_click', { type: data.type, action: event.action });
    
    event.waitUntil(
        clients.openWindow(data.url || '/')
    );
});

// Handle messages from main thread
self.addEventListener('message', event => {
    const { type, data } = event.data;
    
    switch (type) {
        case 'SKIP_WAITING':
            self.skipWaiting();
            break;
            
        case 'GET_CACHE_STATUS':
            getCacheStatus().then(status => {
                event.ports[0].postMessage({ type: 'CACHE_STATUS', data: status });
            });
            break;
            
        case 'CLEAR_CACHE':
            clearAllCaches().then(result => {
                event.ports[0].postMessage({ type: 'CACHE_CLEARED', data: result });
            });
            break;
            
        case 'AUDIT_EVENT':
            logAuditEvent(data.event, data.details);
            break;
            
        default:
            console.warn('Unknown message type:', type);
    }
});

// Get cache status information
async function getCacheStatus() {
    const cacheNames = await caches.keys();
    const status = {
        caches: [],
        totalSize: 0
    };
    
    for (const cacheName of cacheNames) {
        const cache = await caches.open(cacheName);
        const requests = await cache.keys();
        
        let cacheSize = 0;
        for (const request of requests) {
            const response = await cache.match(request);
            if (response && response.headers.get('content-length')) {
                cacheSize += parseInt(response.headers.get('content-length'));
            }
        }
        
        status.caches.push({
            name: cacheName,
            entries: requests.length,
            size: cacheSize
        });
        
        status.totalSize += cacheSize;
    }
    
    return status;
}

// Clear all caches
async function clearAllCaches() {
    const cacheNames = await caches.keys();
    const deletePromises = cacheNames.map(name => caches.delete(name));
    await Promise.all(deletePromises);
    
    logAuditEvent('all_caches_cleared', { 
        cleared_caches: cacheNames.length 
    });
    
    return { cleared: cacheNames.length, caches: cacheNames };
}

// Error handling for uncaught errors
self.addEventListener('error', event => {
    console.error('Service Worker error:', event.error);
    logAuditEvent('sw_error', { 
        error: event.error.message,
        filename: event.filename,
        lineno: event.lineno
    });
});

// Handle unhandled promise rejections
self.addEventListener('unhandledrejection', event => {
    console.error('Service Worker unhandled rejection:', event.reason);
    logAuditEvent('sw_unhandled_rejection', { 
        reason: event.reason 
    });
});

console.log('QMS Service Worker loaded - Medical Device Quality Management System');
console.log('Regulatory Compliance: FDA 21 CFR Part 820, ISO 13485, ISO 14971');
console.log('Version:', CACHE_NAME);
