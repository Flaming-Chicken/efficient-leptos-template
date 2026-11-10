// Service worker registration and the PWA install prompt bridge. Loaded as a file (not inline) so the Content Security Policy
// does not need 'unsafe-inline' for scripts.
if ('serviceWorker' in navigator && window.location.protocol.startsWith('http')) {
    window.addEventListener('load', () => {
        navigator.serviceWorker.register('/sw.js', { scope: '/' })
            .then(reg => {
                console.log('[PWA] Service Worker registered successfully:', reg.scope);
            })
            .catch(err => {
                console.warn('[PWA] Service Worker registration failed:', err);
            });
    });
}

// PWA Install Prompt Engine
window.__pwaInstallPrompt = null;
window.__pwaInstallAvailable = false;
window.__pwaInstalled = window.matchMedia('(display-mode: standalone)').matches || window.navigator.standalone === true;

window.addEventListener('beforeinstallprompt', (e) => {
    e.preventDefault();
    window.__pwaInstallPrompt = e;
    window.__pwaInstallAvailable = true;
    window.dispatchEvent(new CustomEvent('pwa-install-available'));
    console.log('PWA installation prompt captured and ready');
});

window.addEventListener('appinstalled', () => {
    window.__pwaInstallPrompt = null;
    window.__pwaInstallAvailable = false;
    window.__pwaInstalled = true;
    window.dispatchEvent(new CustomEvent('pwa-installed'));
    console.log('PWA successfully installed by user');
});

window.__triggerPWAInstall = async function () {
    if (!window.__pwaInstallPrompt) {
        console.warn('PWA install prompt is not available at this time');
        return false;
    }
    try {
        window.__pwaInstallPrompt.prompt();
        const { outcome } = await window.__pwaInstallPrompt.userChoice;
        console.log(`User response to PWA install prompt: ${outcome}`);
        if (outcome === 'accepted') {
            window.__pwaInstallPrompt = null;
            window.__pwaInstallAvailable = false;
            return true;
        }
        return false;
    } catch (err) {
        console.error('Error triggering PWA install:', err);
        return false;
    }
};
