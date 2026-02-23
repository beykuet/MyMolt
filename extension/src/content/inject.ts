// Content Script — injected into every page
// Responsibilities:
// 1. Extract readable content → send to service worker
// 2. Detect login forms → trigger vault autofill
// 3. Inject floating MyMolt button

import type { PageContext } from '../shared/types';

// ─── Page Content Extraction ────────────────────────────────────
function extractPageContent(): PageContext {
    // Use a simple readability extraction (Readability lib loaded separately if needed)
    // For now, extract main text content intelligently
    const article = document.querySelector('article, [role="main"], main, .content, #content');
    const textSource = article || document.body;

    // Remove scripts, styles, nav, footer from clone
    const clone = textSource.cloneNode(true) as HTMLElement;
    clone.querySelectorAll('script, style, nav, footer, header, aside, [role="navigation"], [role="banner"]').forEach(el => el.remove());

    const text = clone.innerText || clone.textContent || '';
    const cleanText = text
        .replace(/\s+/g, ' ')
        .replace(/\n{3,}/g, '\n\n')
        .trim();

    return {
        url: window.location.href,
        title: document.title,
        text: cleanText,
        lang: document.documentElement.lang || 'en',
        wordCount: cleanText.split(/\s+/).length,
    };
}

// Extract and send page context after DOM is ready
function sendPageContext() {
    const context = extractPageContent();
    chrome.runtime.sendMessage({ type: 'PAGE_CONTEXT', payload: context });
}

// Wait for page to be fully loaded, then extract
if (document.readyState === 'complete') {
    sendPageContext();
} else {
    window.addEventListener('load', sendPageContext);
}

// ─── Login Form Detection ───────────────────────────────────────
function detectLoginForms() {
    const passwordFields = document.querySelectorAll<HTMLInputElement>('input[type="password"]');
    if (passwordFields.length === 0) return;

    passwordFields.forEach(passwordField => {
        // Find the corresponding username field
        const form = passwordField.closest('form');
        const usernameField = form?.querySelector<HTMLInputElement>(
            'input[type="email"], input[type="text"][name*="user"], input[type="text"][name*="email"], input[type="text"][autocomplete="username"], input[name="login"], input[name="username"]'
        );

        if (!usernameField) return;

        // Check if we've already processed this form
        if (form?.dataset.mymoltProcessed) return;
        if (form) form.dataset.mymoltProcessed = 'true';

        // Request autofill from service worker
        chrome.runtime.sendMessage(
            { type: 'AUTOFILL_REQUEST', payload: { url: window.location.href } },
            (response) => {
                if (response?.payload) {
                    // Add a small MyMolt badge next to the username field
                    injectAutofillBadge(usernameField, passwordField, response.payload);
                }
            }
        );
    });
}

function injectAutofillBadge(
    usernameField: HTMLInputElement,
    passwordField: HTMLInputElement,
    credentials: { username: string; password: string }
) {
    // Create the autofill badge
    const badge = document.createElement('div');
    badge.className = 'mymolt-autofill-badge';
    badge.innerHTML = `
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <rect x="3" y="11" width="18" height="11" rx="2" ry="2"/>
            <path d="M7 11V7a5 5 0 0 1 10 0v4"/>
        </svg>
    `;
    badge.title = `MyMolt Vault: ${credentials.username}`;

    badge.addEventListener('click', (e) => {
        e.preventDefault();
        e.stopPropagation();
        usernameField.value = credentials.username;
        passwordField.value = credentials.password;
        // Trigger input events so frameworks pick up the change
        usernameField.dispatchEvent(new Event('input', { bubbles: true }));
        passwordField.dispatchEvent(new Event('input', { bubbles: true }));
        usernameField.dispatchEvent(new Event('change', { bubbles: true }));
        passwordField.dispatchEvent(new Event('change', { bubbles: true }));

        // Visual feedback
        badge.style.background = '#22c55e';
        setTimeout(() => { badge.style.background = ''; }, 1000);
    });

    // Position next to the username field
    const wrapper = usernameField.parentElement;
    if (wrapper) {
        wrapper.style.position = 'relative';
        wrapper.appendChild(badge);
    }
}

// ─── Floating MyMolt Button ─────────────────────────────────────
function injectFloatingButton() {
    const btn = document.createElement('div');
    btn.className = 'mymolt-floating-btn';
    btn.innerHTML = `
        <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M12 8V4H8"/>
            <rect x="2" y="2" width="20" height="20" rx="5"/>
            <path d="M8 14s1.5 2 4 2 4-2 4-2"/>
            <line x1="9" y1="9" x2="9.01" y2="9"/>
            <line x1="15" y1="9" x2="15.01" y2="9"/>
        </svg>
    `;
    btn.title = 'Ask MyMolt about this page';

    btn.addEventListener('click', () => {
        chrome.runtime.sendMessage({ type: 'OPEN_SIDEPANEL' });
    });

    document.body.appendChild(btn);
}

// Run detections
setTimeout(() => {
    detectLoginForms();
    injectFloatingButton();
}, 500);

// Re-detect login forms on DOM mutations (for SPAs)
const observer = new MutationObserver(() => {
    detectLoginForms();
});
observer.observe(document.body, { childList: true, subtree: true });
