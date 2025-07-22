// Static assets management for QMS web interface
// All assets are embedded at compile time to maintain stdlib-only constraint

use std::collections::HashMap;

/// Asset manager for embedded static files
#[derive(Clone)]
pub struct AssetManager {
    assets: HashMap<String, Asset>,
}

/// Represents a static asset
#[derive(Clone)]
pub struct Asset {
    pub content: Vec<u8>,
    pub content_type: String,
    pub etag: String,
    pub compressed: bool,
    pub original_size: usize,
}

impl AssetManager {
    pub fn new() -> Self {
        Self {
            assets: Self::load_all_assets(),
        }
    }

    /// Get asset by path
    pub fn get_asset(&self, path: &str) -> Option<&Asset> {
        self.assets.get(path)
    }

    /// Get asset with fallback to index.html for SPA routing
    pub fn get_asset_with_fallback(&self, path: &str) -> Option<&Asset> {
        self.get_asset(path).or_else(|| {
            // For SPA routing, return index.html for non-API routes
            if !path.starts_with("/api/") && !path.contains('.') {
                self.get_asset("/index.html")
            } else {
                None
            }
        })
    }

    /// List all available assets
    pub fn list_assets(&self) -> Vec<&String> {
        self.assets.keys().collect()
    }

    /// Get asset count
    pub fn asset_count(&self) -> usize {
        self.assets.len()
    }

    /// Calculate total asset size
    pub fn total_size(&self) -> usize {
        self.assets.values().map(|asset| asset.content.len()).sum()
    }

    /// Calculate total compressed size (if compression is enabled)
    pub fn total_compressed_size(&self) -> usize {
        self.assets.values().map(|asset| {
            if asset.compressed {
                asset.content.len()
            } else {
                asset.original_size
            }
        }).sum()
    }

    /// Get compression ratio as percentage
    pub fn compression_ratio(&self) -> f64 {
        let original = self.assets.values().map(|a| a.original_size).sum::<usize>();
        let compressed = self.total_size();
        
        if original > 0 {
            (1.0 - (compressed as f64 / original as f64)) * 100.0
        } else {
            0.0
        }
    }

    /// Get cache control header for asset
    pub fn get_cache_control(&self, path: &str) -> &'static str {
        if path.ends_with(".html") {
            "no-cache, must-revalidate"
        } else if path.ends_with(".css") || path.ends_with(".js") {
            "public, max-age=31536000, immutable" // 1 year for versioned assets
        } else if path.ends_with(".ico") || path.ends_with(".png") || path.ends_with(".jpg") {
            "public, max-age=2592000" // 30 days for images
        } else if path.ends_with(".json") {
            "public, max-age=300" // 5 minutes for manifests
        } else {
            "public, max-age=3600" // 1 hour default
        }
    }

    /// Load all embedded assets
    fn load_all_assets() -> HashMap<String, Asset> {
        let mut assets = HashMap::new();

        // Main application files
        assets.insert("/index.html".to_string(), Asset::new(
            include_str!("../web_assets/index.html").as_bytes().to_vec(),
            "text/html; charset=utf-8"
        ));

        assets.insert("/styles.css".to_string(), Asset::new(
            include_str!("../web_assets/styles.css").as_bytes().to_vec(),
            "text/css"
        ));

        // Load JavaScript with debug info
        let js_content = include_str!("../web_assets/app.js");
        println!("🔍 Loading app.js: {} bytes, starts with: '{}'",
                 js_content.len(),
                 &js_content[..std::cmp::min(50, js_content.len())]);

        assets.insert("/app.js".to_string(), Asset::new(
            js_content.as_bytes().to_vec(),
            "application/javascript"
        ));

        // Favicon (simple embedded version)
        assets.insert("/favicon.ico".to_string(), Asset::new(
            Self::get_embedded_favicon(),
            "image/x-icon"
        ));

        // PWA manifest
        assets.insert("/manifest.json".to_string(), Asset::new(
            Self::get_manifest_json().as_bytes().to_vec(),
            "application/json"
        ));

        // Service worker for offline support (PWA)
        assets.insert("/sw.js".to_string(), Asset::new(
            include_str!("../web_assets/sw.js").as_bytes().to_vec(),
            "application/javascript"
        ));

        assets
    }

    /// Get embedded favicon as bytes
    fn get_embedded_favicon() -> Vec<u8> {
        // Simple 16x16 ICO file (medical cross icon concept)
        // This is a placeholder - in production you'd embed a real favicon
        vec![
            0x00, 0x00, 0x01, 0x00, 0x01, 0x00, 0x10, 0x10, 0x00, 0x00, 0x01, 0x00, 0x20, 0x00,
            0x68, 0x04, 0x00, 0x00, 0x16, 0x00, 0x00, 0x00, 0x28, 0x00, 0x00, 0x00, 0x10, 0x00,
            0x00, 0x00, 0x20, 0x00, 0x00, 0x00, 0x01, 0x00, 0x20, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x04, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ]
    }

    /// Get PWA manifest JSON
    fn get_manifest_json() -> String {
        format!("{{\
  \"name\": \"QMS - Medical Device Quality Management\",\
  \"short_name\": \"QMS\",\
  \"description\": \"FDA 21 CFR Part 820 & ISO 13485 Compliant Quality Management System\",\
  \"start_url\": \"/\",\
  \"display\": \"standalone\",\
  \"background_color\": \"#{}\",\
  \"theme_color\": \"#{}\",\
  \"orientation\": \"portrait-primary\",\
  \"categories\": [\"medical\", \"business\", \"productivity\"],\
  \"icons\": [\
    {{\
      \"src\": \"/favicon.ico\",\
      \"sizes\": \"16x16\",\
      \"type\": \"image/x-icon\"\
    }}\
  ],\
  \"scope\": \"/\",\
  \"lang\": \"en-US\",\
  \"dir\": \"ltr\"\
}}", "2c3e50", "3498db")
    }

    /// Get service worker JavaScript
    const fn get_service_worker_js() -> &'static str {
        r#"// QMS Service Worker for offline support
const CACHE_NAME = 'qms-v1.0.0';
const STATIC_ASSETS = [
  '/',
  '/index.html',
  '/styles.css',
  '/app.js',
  '/manifest.json'
];

// Install event - cache static assets
self.addEventListener('install', event => {
  console.log('QMS Service Worker: Installing...');
  event.waitUntil(
    caches.open(CACHE_NAME)
      .then(cache => {
        console.log('QMS Service Worker: Caching static assets');
        return cache.addAll(STATIC_ASSETS);
      })
      .then(() => {
        console.log('QMS Service Worker: Installation complete');
        return self.skipWaiting();
      })
  );
});

// Activate event - clean up old caches
self.addEventListener('activate', event => {
  console.log('QMS Service Worker: Activating...');
  event.waitUntil(
    caches.keys()
      .then(cacheNames => {
        return Promise.all(
          cacheNames.map(cacheName => {
            if (cacheName !== CACHE_NAME) {
              console.log('QMS Service Worker: Deleting old cache', cacheName);
              return caches.delete(cacheName);
            }
          })
        );
      })
      .then(() => {
        console.log('QMS Service Worker: Activation complete');
        return self.clients.claim();
      })
  );
});

// Fetch event - serve from cache, fallback to network
self.addEventListener('fetch', event => {
  // Only handle GET requests
  if (event.request.method !== 'GET') {
    return;
  }

  // Skip API requests (always go to network for fresh data)
  if (event.request.url.includes('/api/')) {
    return;
  }

  event.respondWith(
    caches.match(event.request)
      .then(response => {
        // Return cached version if available
        if (response) {
          console.log('QMS Service Worker: Serving from cache', event.request.url);
          return response;
        }

        // Otherwise fetch from network
        console.log('QMS Service Worker: Fetching from network', event.request.url);
        return fetch(event.request)
          .then(response => {
            // Don't cache if not a valid response
            if (!response || response.status !== 200 || response.type !== 'basic') {
              return response;
            }

            // Clone response for caching
            const responseToCache = response.clone();
            caches.open(CACHE_NAME)
              .then(cache => {
                cache.put(event.request, responseToCache);
              });

            return response;
          })
          .catch(() => {
            // Offline fallback - serve index.html for navigation requests
            if (event.request.mode === 'navigate') {
              return caches.match('/index.html');
            }
          });
      })
  );
});

// Message event - handle commands from main app
self.addEventListener('message', event => {
  if (event.data.action === 'skipWaiting') {
    self.skipWaiting();
  }
});

console.log('QMS Service Worker: Script loaded');"#
    }
}

impl Asset {
    pub fn new(content: Vec<u8>, content_type: &str) -> Self {
        let original_size = content.len();
        let etag = Self::calculate_etag(&content);
        
        // Check if content should be compressed (placeholder for future compression)
        let (final_content, compressed) = if Self::should_compress_content(&content, content_type) {
            // Placeholder: In production, implement actual compression here
            // For medical device compliance, use certified compression
            (Self::simulate_compression(&content), true)
        } else {
            (content, false)
        };
        
        Self {
            content: final_content,
            content_type: content_type.to_string(),
            etag,
            compressed,
            original_size,
        }
    }

    /// Calculate ETag for cache validation
    fn calculate_etag(content: &[u8]) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        content.hash(&mut hasher);
        format!("\"{}\"", hasher.finish())
    }

    /// Check if content should be compressed
    fn should_compress_content(content: &[u8], content_type: &str) -> bool {
        // Compress text-based content larger than 1KB for medical device web interfaces
        // Maintains performance while ensuring data integrity
        content.len() > 1024 && (
            content_type.starts_with("text/html") ||
            content_type.starts_with("text/css") ||
            content_type.starts_with("application/javascript") ||
            content_type.starts_with("application/json")
        )
    }

    /// Simulate compression for stdlib-only implementation
    /// In production, this would use proper gzip compression
    fn simulate_compression(content: &[u8]) -> Vec<u8> {
        // For medical device compliance and stdlib-only constraint,
        // we simulate compression by creating a compressed header
        // Real implementation would use proper compression algorithm
        
        let mut compressed = Vec::with_capacity(content.len() + 10);
        
        // Add compression header (QMS-specific format)
        compressed.extend_from_slice(b"QMS_COMP");
        compressed.extend_from_slice(&(content.len() as u32).to_le_bytes());
        
        // Simple compression simulation: remove redundant whitespace for text content
        let mut last_was_space = false;
        for &byte in content {
            if byte == b' ' || byte == b'\t' || byte == b'\n' || byte == b'\r' {
                if !last_was_space {
                    compressed.push(b' ');
                    last_was_space = true;
                }
            } else {
                compressed.push(byte);
                last_was_space = false;
            }
        }
        
        // Ensure we actually achieved some compression
        if compressed.len() >= content.len() {
            content.to_vec()
        } else {
            compressed
        }
    }

    /// Get content length
    pub fn content_length(&self) -> usize {
        self.content.len()
    }

    /// Get original content length before compression
    pub fn original_length(&self) -> usize {
        self.original_size
    }

    /// Get compression ratio as percentage
    pub fn compression_ratio(&self) -> f64 {
        if self.compressed && self.original_size > 0 {
            (1.0 - (self.content.len() as f64 / self.original_size as f64)) * 100.0
        } else {
            0.0
        }
    }

    /// Check if content can be compressed
    pub fn is_compressible(&self) -> bool {
        Self::should_compress_content(&self.content, &self.content_type)
    }

    /// Get content as string (for text assets)
    /// Single Responsibility Principle: Handles both compressed and uncompressed content
    pub fn as_string(&self) -> Result<String, &'static str> {
        // Use decompressed content to handle both compressed and uncompressed assets
        let content = self.get_decompressed_content()?;
        String::from_utf8(content).map_err(|_| "Invalid UTF-8 content")
    }

    /// Get decompressed content (for future compression implementation)
    pub fn get_decompressed_content(&self) -> Result<Vec<u8>, &'static str> {
        if self.compressed {
            // Check for our compression header
            if self.content.len() >= 12 && &self.content[0..8] == b"QMS_COMP" {
                let _original_size = u32::from_le_bytes([
                    self.content[8], self.content[9], self.content[10], self.content[11]
                ]);
                
                // For now, return compressed content without header
                // In production, implement proper decompression
                Ok(self.content[12..].to_vec())
            } else {
                Err("Invalid compression format")
            }
        } else {
            Ok(self.content.clone())
        }
    }
}

impl Default for AssetManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_asset_manager_creation() {
        let manager = AssetManager::new();
        assert!(manager.asset_count() > 0);
        assert!(manager.total_size() > 0);
    }

    #[test]
    fn test_asset_retrieval() {
        let manager = AssetManager::new();
        
        // Should find main assets
        assert!(manager.get_asset("/index.html").is_some());
        assert!(manager.get_asset("/styles.css").is_some());
        assert!(manager.get_asset("/app.js").is_some());
        assert!(manager.get_asset("/favicon.ico").is_some());
        
        // Should not find non-existent asset
        assert!(manager.get_asset("/nonexistent.txt").is_none());
    }

    #[test]
    fn test_asset_fallback() {
        let manager = AssetManager::new();
        
        // Should fallback to index.html for SPA routes
        assert!(manager.get_asset_with_fallback("/dashboard").is_some());
        assert!(manager.get_asset_with_fallback("/documents").is_some());
        assert!(manager.get_asset_with_fallback("/some/deep/path").is_some());
        
        // Should not fallback for API routes
        assert!(manager.get_asset_with_fallback("/api/docs").is_none());
        
        // Should not fallback for file extensions
        assert!(manager.get_asset_with_fallback("/missing.png").is_none());
    }

    #[test]
    fn test_asset_properties() {
        let content = "Hello, World!".as_bytes().to_vec();
        let asset = Asset::new(content.clone(), "text/plain");
        
        assert_eq!(asset.content_length(), 13);
        assert_eq!(asset.original_length(), 13);
        assert_eq!(asset.content_type, "text/plain");
        assert!(!asset.etag.is_empty());
        assert!(!asset.compressed); // Small content shouldn't be compressed
        assert_eq!(asset.compression_ratio(), 0.0);
    }

    #[test]
    fn test_compressible_detection() {
        assert!(Asset::new(vec![0; 2000], "text/html").compressed); // Large text should be compressed
        assert!(Asset::new(vec![0; 2000], "text/css").compressed);
        assert!(Asset::new(vec![0; 2000], "application/javascript").compressed);
        assert!(Asset::new(vec![0; 2000], "application/json").compressed);
        assert!(!Asset::new(vec![0; 2000], "image/png").compressed); // Images not compressed
        assert!(!Asset::new(vec![0; 500], "text/html").compressed); // Small files not compressed
    }

    #[test]
    fn test_compression_simulation() {
        let large_text = "This is a large text file with lots of spaces    and    redundant    whitespace\n\n\n\n".repeat(50);
        let asset = Asset::new(large_text.as_bytes().to_vec(), "text/html");
        
        assert!(asset.compressed);
        assert!(asset.content_length() < asset.original_length());
        assert!(asset.compression_ratio() > 0.0);
    }

    #[test]
    fn test_cache_control_headers() {
        let manager = AssetManager::new();
        
        assert_eq!(manager.get_cache_control("/index.html"), "no-cache, must-revalidate");
        assert_eq!(manager.get_cache_control("/styles.css"), "public, max-age=31536000, immutable");
        assert_eq!(manager.get_cache_control("/app.js"), "public, max-age=31536000, immutable");
        assert_eq!(manager.get_cache_control("/favicon.ico"), "public, max-age=2592000");
        assert_eq!(manager.get_cache_control("/manifest.json"), "public, max-age=300");
        assert_eq!(manager.get_cache_control("/unknown.txt"), "public, max-age=3600");
    }

    #[test]
    fn test_compression_stats() {
        let manager = AssetManager::new();
        
        assert!(manager.total_size() > 0);
        assert!(manager.total_compressed_size() > 0);
        
        // Compression ratio should be reasonable (between 0-100%)
        let ratio = manager.compression_ratio();
        assert!(ratio >= 0.0 && ratio <= 100.0);
    }

    #[test]
    fn test_etag_calculation() {
        let content1 = "content1".as_bytes().to_vec();
        let content2 = "content2".as_bytes().to_vec();
        
        let asset1 = Asset::new(content1, "text/plain");
        let asset2 = Asset::new(content2, "text/plain");
        
        // Different content should have different ETags
        assert_ne!(asset1.etag, asset2.etag);
        
        // Same content should have same ETag
        let asset1_copy = Asset::new("content1".as_bytes().to_vec(), "text/plain");
        assert_eq!(asset1.etag, asset1_copy.etag);
    }

    #[test]
    fn test_asset_listing() {
        let manager = AssetManager::new();
        let assets = manager.list_assets();
        
        assert!(assets.len() >= 5); // At least our main assets
        assert!(assets.contains(&&"/index.html".to_string()));
        assert!(assets.contains(&&"/styles.css".to_string()));
        assert!(assets.contains(&&"/app.js".to_string()));
    }

    #[test]
    fn test_manifest_json_validity() {
        let manifest = AssetManager::get_manifest_json();
        assert!(manifest.contains("QMS"));
        assert!(manifest.contains("Medical Device"));
        assert!(manifest.contains("start_url"));
        assert!(manifest.contains("display"));
    }

    #[test]
    fn test_service_worker_included() {
        // Test that service worker is properly included from external file
        // Single Responsibility Principle: Test focuses on verifying SW content exists
        let manager = AssetManager::new();
        let sw_asset = manager.get_asset("/sw.js");
        assert!(sw_asset.is_some(), "Service worker asset should be available");

        let content = sw_asset.unwrap().as_string().expect("Should be able to get service worker content as string");
        assert!(!content.is_empty(), "Service worker content should not be empty");

        // Verify essential service worker components (more flexible than exact text matching)
        assert!(content.contains("CACHE_NAME"), "Service worker should define CACHE_NAME");
        assert!(content.contains("addEventListener"), "Service worker should have event listeners");
        assert!(content.contains("install"), "Service worker should handle install event");
        assert!(content.contains("activate"), "Service worker should handle activate event");
        assert!(content.contains("fetch"), "Service worker should handle fetch event");

        // Verify medical device compliance features
        assert!(content.contains("FDA 21 CFR Part 820"), "Should include FDA compliance reference");
        assert!(content.contains("ISO 13485"), "Should include ISO 13485 compliance reference");
        assert!(content.contains("audit"), "Should include audit functionality for medical device compliance");
    }
}
