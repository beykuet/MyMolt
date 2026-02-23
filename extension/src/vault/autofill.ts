// Vault Autofill — detects login forms and fills credentials from MyMolt vault
// This module is used by the content script

export interface FormCandidate {
    form: HTMLFormElement | null;
    usernameField: HTMLInputElement;
    passwordField: HTMLInputElement;
}

// Find all login form candidates on the page
export function detectLoginForms(): FormCandidate[] {
    const candidates: FormCandidate[] = [];
    const passwordFields = document.querySelectorAll<HTMLInputElement>('input[type="password"]');

    passwordFields.forEach(passwordField => {
        if (!passwordField.offsetParent) return; // Skip hidden fields

        const form = passwordField.closest('form');

        // Find username/email field — search in form first, then nearby
        const container = form || passwordField.parentElement?.parentElement || document.body;
        const usernameField = container.querySelector<HTMLInputElement>(
            [
                'input[type="email"]',
                'input[autocomplete="username"]',
                'input[autocomplete="email"]',
                'input[name="username"]',
                'input[name="user"]',
                'input[name="email"]',
                'input[name="login"]',
                'input[name="id"]',
                'input[type="text"][name*="user"]',
                'input[type="text"][name*="email"]',
                'input[type="text"][name*="login"]',
            ].join(', ')
        );

        if (usernameField) {
            candidates.push({ form, usernameField, passwordField });
        }
    });

    return candidates;
}

// Securely fill a form with credentials
export function fillCredentials(
    usernameField: HTMLInputElement,
    passwordField: HTMLInputElement,
    username: string,
    password: string
) {
    // Use native setter to bypass React/Vue/Angular controlled inputs
    const nativeInputValueSetter = Object.getOwnPropertyDescriptor(
        window.HTMLInputElement.prototype, 'value'
    )?.set;

    if (nativeInputValueSetter) {
        nativeInputValueSetter.call(usernameField, username);
        nativeInputValueSetter.call(passwordField, password);
    } else {
        usernameField.value = username;
        passwordField.value = password;
    }

    // Dispatch events to trigger framework reactivity
    const events = ['input', 'change', 'blur'];
    events.forEach(eventType => {
        usernameField.dispatchEvent(new Event(eventType, { bubbles: true }));
        passwordField.dispatchEvent(new Event(eventType, { bubbles: true }));
    });
}

// Match URL patterns (simple glob matching)
export function matchUrlPattern(pattern: string, url: string): boolean {
    const regex = new RegExp(
        '^' + pattern
            .replace(/[.+?^${}()|[\]\\]/g, '\\$&')
            .replace(/\*/g, '.*')
        + '$',
        'i'
    );
    return regex.test(url);
}
