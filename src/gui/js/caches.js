// Cache Browser JavaScript

const { invoke } = window.__TAURI__.core;

// Load all cached regions
async function loadCaches() {
    const loadingState = document.getElementById('loading-state');
    const emptyState = document.getElementById('empty-state');
    const cacheGrid = document.getElementById('cache-grid');

    // Show loading state
    loadingState.style.display = 'block';
    emptyState.style.display = 'none';
    cacheGrid.innerHTML = '';

    try {
        // Fetch caches from backend
        const cachesData = await invoke('gui_list_caches');
        window.caches = cachesData;

        // Hide loading state
        loadingState.style.display = 'none';

        if (cachesData.length === 0) {
            // Show empty state
            emptyState.style.display = 'block';
        } else {
            // Render cache cards
            await renderCaches(cachesData);

            // Update statistics
            updateStatistics(cachesData);
        }
    } catch (error) {
        console.error('Failed to load caches:', error);
        loadingState.style.display = 'none';
        showError('Failed to load cached regions: ' + error);
    }
}

// Render cache cards in the grid
async function renderCaches(cachesData) {
    const cacheGrid = document.getElementById('cache-grid');
    cacheGrid.innerHTML = '';

    for (const cache of cachesData) {
        const card = await createCacheCard(cache);
        cacheGrid.appendChild(card);
    }
}

// Create a single cache card element
async function createCacheCard(cache) {
    const card = document.createElement('div');
    card.className = 'cache-card';

    // Check if cache is expired
    const isExpired = cache.expires_at && new Date(cache.expires_at) < new Date();
    if (isExpired) {
        card.classList.add('expired');
    }

    // Create preview section
    const preview = document.createElement('div');
    preview.className = 'cache-preview';

    if (cache.has_preview) {
        try {
            const previewData = await invoke('gui_get_cache_preview', { cacheId: cache.id });
            if (previewData) {
                const img = document.createElement('img');
                img.src = previewData;
                img.alt = 'Cache preview';
                preview.appendChild(img);
            } else {
                preview.innerHTML = '<div class="no-preview">🗺️</div>';
            }
        } catch (error) {
            console.error('Failed to load preview:', error);
            preview.innerHTML = '<div class="no-preview">🗺️</div>';
        }
    } else {
        preview.innerHTML = '<div class="no-preview">🗺️</div>';
    }

    // Add expired badge if applicable
    if (isExpired) {
        const expiredBadge = document.createElement('div');
        expiredBadge.className = 'expired-badge';
        expiredBadge.textContent = 'EXPIRED';
        preview.appendChild(expiredBadge);
    }

    // Create info section
    const info = document.createElement('div');
    info.className = 'cache-info';

    const name = document.createElement('h3');
    name.className = 'cache-name';
    name.textContent = cache.name;
    name.title = cache.name;

    const details = document.createElement('div');
    details.className = 'cache-details';

    // Format date
    const createdDate = new Date(cache.created_at);
    const dateStr = createdDate.toLocaleDateString() + ' ' + createdDate.toLocaleTimeString();

    // Format size
    const sizeStr = formatBytes(cache.size_bytes);

    // Format expiration
    let expiresStr = 'Never';
    if (cache.expires_at) {
        const expiresDate = new Date(cache.expires_at);
        if (isExpired) {
            expiresStr = 'Expired';
        } else {
            const daysLeft = Math.ceil((expiresDate - new Date()) / (1000 * 60 * 60 * 24));
            expiresStr = `${daysLeft} day${daysLeft !== 1 ? 's' : ''}`;
        }
    }

    details.innerHTML = `
        <div class="cache-detail-row">
            <span class="cache-detail-label">Created:</span>
            <span class="cache-detail-value">${dateStr}</span>
        </div>
        <div class="cache-detail-row">
            <span class="cache-detail-label">Elements:</span>
            <span class="cache-detail-value">${cache.element_count.toLocaleString()}</span>
        </div>
        <div class="cache-detail-row">
            <span class="cache-detail-label">Size:</span>
            <span class="cache-detail-value">${sizeStr}</span>
        </div>
        <div class="cache-detail-row">
            <span class="cache-detail-label">Scale:</span>
            <span class="cache-detail-value">${cache.scale.toFixed(2)}x</span>
        </div>
        <div class="cache-detail-row">
            <span class="cache-detail-label">Terrain:</span>
            <span class="cache-detail-value ${cache.has_terrain ? 'terrain-yes' : 'terrain-no'}">
                ${cache.has_terrain ? 'Yes' : 'No'}
            </span>
        </div>
        <div class="cache-detail-row">
            <span class="cache-detail-label">Expires:</span>
            <span class="cache-detail-value ${isExpired ? 'terrain-no' : ''}">${expiresStr}</span>
        </div>
    `;

    info.appendChild(name);
    info.appendChild(details);

    // Create actions section
    const actions = document.createElement('div');
    actions.className = 'cache-card-actions';

    const generateBtn = document.createElement('button');
    generateBtn.className = 'generate-btn';
    generateBtn.textContent = 'Generate';
    generateBtn.onclick = (e) => {
        e.stopPropagation();
        openGenerateModal(cache);
    };

    const deleteBtn = document.createElement('button');
    deleteBtn.className = 'delete-btn';
    deleteBtn.textContent = 'Delete';
    deleteBtn.onclick = (e) => {
        e.stopPropagation();
        openDeleteModal(cache);
    };

    actions.appendChild(generateBtn);
    actions.appendChild(deleteBtn);

    // Assemble card
    card.appendChild(preview);
    card.appendChild(info);
    card.appendChild(actions);

    return card;
}

// Update statistics display
function updateStatistics(cachesData) {
    const totalCaches = cachesData.length;
    const totalSize = cachesData.reduce((sum, cache) => sum + cache.size_bytes, 0);
    const expiredCount = cachesData.filter(cache =>
        cache.expires_at && new Date(cache.expires_at) < new Date()
    ).length;

    document.getElementById('total-caches').textContent = totalCaches;
    document.getElementById('total-size').textContent = formatBytes(totalSize);
    document.getElementById('expired-caches').textContent = expiredCount;
}

// Format bytes to human-readable string
function formatBytes(bytes) {
    if (bytes === 0) return '0 B';
    const k = 1024;
    const sizes = ['B', 'KB', 'MB', 'GB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
}

// Open generate from cache modal
function openGenerateModal(cache) {
    window.selectedCacheId = cache.id;
    window.selectedWorldPath = null;

    document.getElementById('modal-cache-name').textContent = cache.name;
    document.getElementById('modal-cache-details').textContent =
        `${cache.bbox} • ${cache.element_count.toLocaleString()} elements • ${formatBytes(cache.size_bytes)}`;

    document.getElementById('modal-selected-world').textContent = 'No world selected';
    document.getElementById('generate-modal').style.display = 'block';
}

// Close generate modal
function closeGenerateModal() {
    document.getElementById('generate-modal').style.display = 'none';
    window.selectedCacheId = null;
    window.selectedWorldPath = null;
}

// Select world for cache generation
async function selectWorldForCache() {
    try {
        const worldPath = await invoke('gui_select_world', { generateNew: false });
        window.selectedWorldPath = worldPath;
        document.getElementById('modal-selected-world').textContent = worldPath;
        document.getElementById('modal-selected-world').style.color = '#7bd864';
    } catch (error) {
        if (error !== 4) { // 4 = No world selected (user cancelled)
            console.error('Failed to select world:', error);
            showError('Failed to select world');
        }
    }
}

// Start generation from cache
async function startGenerationFromCache() {
    if (!window.selectedCacheId) {
        showError('No cache selected');
        return;
    }

    if (!window.selectedWorldPath) {
        showError('Please select a world first');
        return;
    }

    const interior = document.getElementById('modal-interior').checked;
    const roof = document.getElementById('modal-roof').checked;
    const fillground = document.getElementById('modal-fillground').checked;

    closeGenerateModal();
    showProgressModal();

    try {
        await invoke('gui_generate_from_cache', {
            cacheId: window.selectedCacheId,
            selectedWorld: window.selectedWorldPath,
            groundLevel: -62,
            floodfillTimeout: 20,
            interiorEnabled: interior,
            roofEnabled: roof,
            fillgroundEnabled: fillground,
            isNewWorld: false,
            spawnPoint: null,
            telemetryConsent: true,
            worldFormat: 'java'
        });

        hideProgressModal();
        showSuccess('World generated successfully from cache!');

    } catch (error) {
        hideProgressModal();
        console.error('Generation failed:', error);
        showError('Failed to generate world: ' + error);
    }
}

// Open delete confirmation modal
function openDeleteModal(cache) {
    window.cacheToDelete = cache;

    document.getElementById('delete-cache-name').textContent = cache.name;
    document.getElementById('delete-cache-details').textContent =
        `Created: ${new Date(cache.created_at).toLocaleDateString()} • Size: ${formatBytes(cache.size_bytes)}`;

    document.getElementById('delete-modal').style.display = 'block';
}

// Close delete modal
function closeDeleteModal() {
    document.getElementById('delete-modal').style.display = 'none';
    window.cacheToDelete = null;
}

// Confirm delete
async function confirmDelete() {
    if (!window.cacheToDelete) return;

    const cacheId = window.cacheToDelete.id;
    closeDeleteModal();

    try {
        await invoke('gui_delete_cache', { cacheId });
        showSuccess('Cache deleted successfully');
        loadCaches(); // Reload the cache list
    } catch (error) {
        console.error('Failed to delete cache:', error);
        showError('Failed to delete cache: ' + error);
    }
}

// Clear all caches
async function clearAllCaches() {
    if (!confirm('Are you sure you want to delete ALL cached regions? This action cannot be undone.')) {
        return;
    }

    try {
        await invoke('gui_clear_caches');
        showSuccess('All caches cleared successfully');
        loadCaches(); // Reload the cache list
    } catch (error) {
        console.error('Failed to clear caches:', error);
        showError('Failed to clear caches: ' + error);
    }
}

// Cleanup expired caches
async function cleanupExpiredCaches() {
    try {
        const count = await invoke('gui_cleanup_expired_caches');
        if (count > 0) {
            showSuccess(`Cleaned up ${count} expired cache${count !== 1 ? 's' : ''}`);
            loadCaches(); // Reload the cache list
        } else {
            showInfo('No expired caches found');
        }
    } catch (error) {
        console.error('Failed to cleanup caches:', error);
        showError('Failed to cleanup caches: ' + error);
    }
}

// Show progress modal
function showProgressModal() {
    document.getElementById('progress-modal').style.display = 'block';
    document.getElementById('modal-progress-bar').style.width = '0%';
    document.getElementById('modal-progress-detail').textContent = '0%';
    document.getElementById('modal-progress-message').textContent = 'Starting...';
}

// Hide progress modal
function hideProgressModal() {
    document.getElementById('progress-modal').style.display = 'none';
}

// Update progress
function updateProgress(percent, message) {
    document.getElementById('modal-progress-bar').style.width = percent + '%';
    document.getElementById('modal-progress-detail').textContent = Math.round(percent) + '%';
    if (message) {
        document.getElementById('modal-progress-message').textContent = message;
    }
}

// Show error notification
function showError(message) {
    alert('Error: ' + message);
}

// Show success notification
function showSuccess(message) {
    alert('Success: ' + message);
}

// Show info notification
function showInfo(message) {
    alert(message);
}

// Listen for progress updates from backend
window.addEventListener('DOMContentLoaded', () => {
    // Set up event listener for progress updates if available
    if (window.__TAURI__?.event) {
        window.__TAURI__.event.listen('progress-update', (event) => {
            updateProgress(event.payload.progress, event.payload.message);
        });
    }
});
