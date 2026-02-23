// Shared types between extension components and MyMolt backend

export interface MyMoltConfig {
    host: string;         // e.g. "http://localhost:3000" or remote URL
    token: string;        // Authentication token
    role: UserRole;
    userName: string;
    connected: boolean;
}

export type UserRole = 'Root' | 'Adult' | 'Senior' | 'Child';

export interface VaultCredential {
    id: string;
    url_pattern: string;
    username: string;
    // password is never stored in extension — fetched on demand
}

export interface DnsRule {
    id: number;
    priority: number;
    action: { type: 'block' };
    condition: {
        urlFilter?: string;
        domains?: string[];
        resourceTypes?: string[];
    };
}

export interface PageContext {
    url: string;
    title: string;
    text: string;           // Readable text extracted by Readability
    lang: string;
    wordCount: number;
}

export interface AskRequest {
    url: string;
    page_text: string;
    question: string;
    role: UserRole;
    conversation: { role: string; content: string }[];
}

export interface AskResponse {
    answer: string;
    sources: { title: string; url: string }[];
    media: { type: 'image' | 'video' | 'audio'; url: string; caption: string }[];
}

export interface AutofillEntry {
    url: string;
    username: string;
    timestamp: string;
}

// Messages between content script ↔ service worker ↔ side panel
export type ExtensionMessage =
    | { type: 'PAGE_CONTEXT'; payload: PageContext }
    | { type: 'ASK_AGENT'; payload: { question: string; pageContext: PageContext } }
    | { type: 'AGENT_RESPONSE'; payload: AskResponse }
    | { type: 'AUTOFILL_REQUEST'; payload: { url: string } }
    | { type: 'AUTOFILL_RESPONSE'; payload: { username: string; password: string } | null }
    | { type: 'CONNECTION_STATUS'; payload: { connected: boolean; host: string } }
    | { type: 'GET_PAGE_CONTEXT' }
    | { type: 'OPEN_SIDEPANEL' };
