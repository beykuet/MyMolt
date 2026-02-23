// Service Worker — background script for the MyMolt extension
// Handles: connection management, message routing, DNS Shield rules, vault coordination

import { testConnection, getDnsRules, askAgent, matchVaultCredentials, getCredentialPassword, logAutofill, getConfig } from './shared/api';
import type { ExtensionMessage, PageContext } from './shared/types';

// Store the latest page context per tab
const tabContexts = new Map<number, PageContext>();

// ─── Connection Health ──────────────────────────────────────────
let connectionAlive = false;

async function checkConnection() {
    connectionAlive = await testConnection();
    // Update badge
    chrome.action.setBadgeText({ text: connectionAlive ? '' : '!' });
    chrome.action.setBadgeBackgroundColor({ color: connectionAlive ? '#22c55e' : '#ef4444' });
}

// Check on startup and periodically
checkConnection();
setInterval(checkConnection, 30_000);

// ─── DNS Shield Rules ───────────────────────────────────────────
async function updateDnsRules() {
    try {
        const rules = await getDnsRules();
        if (rules.length > 0) {
            // Remove old rules first
            const oldRuleIds = rules.map(r => r.id);
            await chrome.declarativeNetRequest.updateDynamicRules({
                removeRuleIds: oldRuleIds,
                addRules: rules.map(rule => ({
                    id: rule.id,
                    priority: rule.priority,
                    action: rule.action as chrome.declarativeNetRequest.RuleAction,
                    condition: rule.condition as chrome.declarativeNetRequest.RuleCondition,
                })),
            });
        }
    } catch (e) {
        console.warn('[MyMolt] Failed to update DNS rules:', e);
    }
}

// Update DNS rules on startup and when config changes
updateDnsRules();
chrome.storage.onChanged.addListener((changes) => {
    if (changes['mymolt_config']) {
        updateDnsRules();
        checkConnection();
    }
});

// ─── Message Handling ───────────────────────────────────────────
chrome.runtime.onMessage.addListener((message: ExtensionMessage, sender, sendResponse) => {
    const tabId = sender.tab?.id;

    switch (message.type) {
        case 'PAGE_CONTEXT':
            if (tabId) {
                tabContexts.set(tabId, message.payload);
            }
            break;

        case 'GET_PAGE_CONTEXT':
            if (tabId && tabContexts.has(tabId)) {
                sendResponse(tabContexts.get(tabId));
            }
            return true; // async response

        case 'ASK_AGENT':
            (async () => {
                try {
                    const config = await getConfig();
                    const response = await askAgent({
                        url: message.payload.pageContext.url,
                        page_text: message.payload.pageContext.text.slice(0, 12000),
                        question: message.payload.question,
                        role: config.role,
                        conversation: [],
                    });
                    sendResponse({ type: 'AGENT_RESPONSE', payload: response });
                } catch (e) {
                    sendResponse({ type: 'AGENT_RESPONSE', payload: { answer: 'Sorry, I had trouble processing that.', sources: [], media: [] } });
                }
            })();
            return true; // async response

        case 'AUTOFILL_REQUEST':
            (async () => {
                try {
                    const creds = await matchVaultCredentials(message.payload.url);
                    if (creds.length > 0) {
                        const password = await getCredentialPassword(creds[0].id);
                        await logAutofill(message.payload.url, creds[0].username);
                        sendResponse({ type: 'AUTOFILL_RESPONSE', payload: { username: creds[0].username, password } });
                    } else {
                        sendResponse({ type: 'AUTOFILL_RESPONSE', payload: null });
                    }
                } catch {
                    sendResponse({ type: 'AUTOFILL_RESPONSE', payload: null });
                }
            })();
            return true;

        case 'OPEN_SIDEPANEL':
            if (tabId) {
                chrome.sidePanel.open({ tabId });
            }
            break;

        case 'CONNECTION_STATUS':
            sendResponse({ connected: connectionAlive });
            return true;
    }
});

// ─── Side Panel ─────────────────────────────────────────────────
// Open side panel when clicking the extension icon
chrome.action.onClicked.addListener((tab) => {
    if (tab.id) {
        chrome.sidePanel.open({ tabId: tab.id });
    }
});

// ─── Tab Navigation ─────────────────────────────────────────────
// Clear context when navigating to a new page
chrome.tabs.onUpdated.addListener((tabId, changeInfo) => {
    if (changeInfo.status === 'loading') {
        tabContexts.delete(tabId);
    }
});

chrome.tabs.onRemoved.addListener((tabId) => {
    tabContexts.delete(tabId);
});
