// DNS Shield — generates declarativeNetRequest rules from MyMolt backend blocklists
// Rules are fetched per-role and applied dynamically

import type { DnsRule } from '../shared/types';

// Default blocklist categories per role
const ROLE_BLOCKLISTS: Record<string, string[]> = {
    Child: [
        // Adult content domains (sample — real list from backend)
        '*://adult*/*',
        '*://*.xxx/*',
        '*://*.porn*/*',
        '*://gambling*/*',
        '*://*.casino*/*',
        '*://*.bet*/*',
        // Social media (optional, configurable)
        '*://*.tiktok.com/*',
        '*://*.snapchat.com/*',
    ],
    Senior: [],
    Adult: [],
    Root: [],
};

// SafeSearch enforcement for Child role
const SAFESEARCH_RULES: DnsRule[] = [
    {
        id: 90001,
        priority: 1,
        action: { type: 'block' },
        condition: {
            urlFilter: '*://www.google.*/search*&safe=off*',
            resourceTypes: ['main_frame'],
        },
    },
];

export function generateLocalRules(role: string): DnsRule[] {
    const patterns = ROLE_BLOCKLISTS[role] || [];
    const rules: DnsRule[] = patterns.map((pattern, i) => ({
        id: 80000 + i,
        priority: 1,
        action: { type: 'block' as const },
        condition: {
            urlFilter: pattern,
            resourceTypes: ['main_frame', 'sub_frame'],
        },
    }));

    // Add SafeSearch enforcement for children
    if (role === 'Child') {
        rules.push(...SAFESEARCH_RULES);
    }

    return rules;
}

export async function applyDnsRules(rules: DnsRule[]) {
    try {
        // Get existing dynamic rules
        const existing = await chrome.declarativeNetRequest.getDynamicRules();
        const existingIds = existing.map(r => r.id);

        await chrome.declarativeNetRequest.updateDynamicRules({
            removeRuleIds: existingIds,
            addRules: rules.map(rule => ({
                id: rule.id,
                priority: rule.priority,
                action: rule.action as chrome.declarativeNetRequest.RuleAction,
                condition: rule.condition as chrome.declarativeNetRequest.RuleCondition,
            })),
        });

        console.log(`[MyMolt DNS Shield] Applied ${rules.length} rules`);
    } catch (e) {
        console.error('[MyMolt DNS Shield] Failed to apply rules:', e);
    }
}
