import React, { useState, useEffect, useRef } from 'react';
import type { PageContext, AskResponse, MyMoltConfig } from '../shared/types';

interface Message {
    id: string;
    sender: 'user' | 'agent';
    content: string;
    sources?: { title: string; url: string }[];
    media?: { type: string; url: string; caption: string }[];
}

export const Sidebar: React.FC = () => {
    const [config, setConfig] = useState<MyMoltConfig | null>(null);
    const [pageContext, setPageContext] = useState<PageContext | null>(null);
    const [messages, setMessages] = useState<Message[]>([]);
    const [input, setInput] = useState('');
    const [isThinking, setIsThinking] = useState(false);
    const [connected, setConnected] = useState(false);
    const scrollRef = useRef<HTMLDivElement>(null);

    // Load config and check connection
    useEffect(() => {
        chrome.storage.local.get('mymolt_config', (result) => {
            const cfg = result['mymolt_config'];
            if (cfg) {
                setConfig(cfg);
                setConnected(cfg.connected);
            }
        });

        // Listen for config changes
        chrome.storage.onChanged.addListener((changes) => {
            if (changes['mymolt_config']) {
                const cfg = changes['mymolt_config'].newValue;
                setConfig(cfg);
                setConnected(cfg?.connected || false);
            }
        });
    }, []);

    // Get page context from the active tab
    useEffect(() => {
        const getContext = async () => {
            const [tab] = await chrome.tabs.query({ active: true, currentWindow: true });
            if (tab?.id) {
                chrome.tabs.sendMessage(tab.id, { type: 'GET_PAGE_CONTEXT' }, (response) => {
                    if (response) setPageContext(response);
                });
            }
        };
        getContext();

        // Re-fetch when tab changes
        chrome.tabs.onActivated.addListener(getContext);
        chrome.tabs.onUpdated.addListener((_tabId, changeInfo) => {
            if (changeInfo.status === 'complete') getContext();
        });
    }, []);

    // Reset messages when page changes
    useEffect(() => {
        setMessages([]);
    }, [pageContext?.url]);

    // Auto-scroll
    useEffect(() => {
        if (scrollRef.current) {
            scrollRef.current.scrollTop = scrollRef.current.scrollHeight;
        }
    }, [messages]);

    const ask = async (question: string) => {
        if (!question.trim() || !pageContext) return;

        const userMsg: Message = {
            id: Math.random().toString(),
            sender: 'user',
            content: question,
        };
        setMessages(prev => [...prev, userMsg]);
        setInput('');
        setIsThinking(true);

        try {
            const response = await new Promise<AskResponse>((resolve) => {
                chrome.runtime.sendMessage(
                    { type: 'ASK_AGENT', payload: { question, pageContext } },
                    (res) => resolve(res?.payload || { answer: 'No response', sources: [], media: [] })
                );
            });

            setMessages(prev => [...prev, {
                id: Math.random().toString(),
                sender: 'agent',
                content: response.answer,
                sources: response.sources,
                media: response.media,
            }]);
        } catch {
            setMessages(prev => [...prev, {
                id: Math.random().toString(),
                sender: 'agent',
                content: 'Sorry, I had trouble processing that.',
            }]);
        }
        setIsThinking(false);
    };

    const isSeniorOrChild = config?.role === 'Senior' || config?.role === 'Child';
    const fontSize = isSeniorOrChild ? '15px' : '13px';

    return (
        <div style={{
            display: 'flex',
            flexDirection: 'column',
            height: '100vh',
            background: 'linear-gradient(180deg, #0a1225 0%, #0f1520 100%)',
            color: '#e2e8f0',
            fontFamily: "'Segoe UI', system-ui, -apple-system, sans-serif",
        }}>
            {/* Header */}
            <div style={{
                padding: '16px',
                borderBottom: '1px solid rgba(255,255,255,0.06)',
                background: 'linear-gradient(90deg, rgba(0,68,170,0.4), rgba(0,85,212,0.2))',
                display: 'flex',
                alignItems: 'center',
                gap: '10px',
            }}>
                <div style={{
                    width: 36,
                    height: 36,
                    borderRadius: 12,
                    background: 'linear-gradient(135deg, #ffcc00, #f59e0b)',
                    display: 'flex',
                    alignItems: 'center',
                    justifyContent: 'center',
                    fontSize: '18px',
                    fontWeight: 'bold',
                    color: '#000',
                    flexShrink: 0,
                }}>
                    M
                </div>
                <div style={{ flex: 1 }}>
                    <div style={{ fontWeight: 700, fontSize: '14px' }}>MyMolt</div>
                    <div style={{
                        fontSize: '9px',
                        textTransform: 'uppercase',
                        letterSpacing: '0.15em',
                        color: connected ? 'rgba(52,211,153,0.7)' : 'rgba(239,68,68,0.7)',
                        fontWeight: 700,
                    }}>
                        {connected ? '● Connected' : '○ Disconnected'}
                    </div>
                </div>
            </div>

            {/* Page info */}
            {pageContext && (
                <div style={{
                    padding: '12px 16px',
                    borderBottom: '1px solid rgba(255,255,255,0.04)',
                    background: 'rgba(255,255,255,0.02)',
                }}>
                    <div style={{ fontSize: '11px', fontWeight: 600, color: 'rgba(255,255,255,0.5)', marginBottom: 2 }}>
                        Currently viewing
                    </div>
                    <div style={{ fontSize: '12px', fontWeight: 600, color: 'rgba(255,255,255,0.8)', overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap' }}>
                        {pageContext.title || pageContext.url}
                    </div>
                    <div style={{ fontSize: '10px', color: 'rgba(255,255,255,0.2)', marginTop: 2 }}>
                        {pageContext.wordCount.toLocaleString()} words · {pageContext.lang}
                    </div>
                </div>
            )}

            {/* Messages */}
            <div ref={scrollRef} style={{
                flex: 1,
                overflowY: 'auto',
                padding: '16px',
                display: 'flex',
                flexDirection: 'column',
                gap: '12px',
            }}>
                {messages.length === 0 && (
                    <div style={{
                        display: 'flex',
                        flexDirection: 'column',
                        alignItems: 'center',
                        justifyContent: 'center',
                        height: '100%',
                        textAlign: 'center',
                        gap: '12px',
                    }}>
                        <div style={{
                            width: 56, height: 56, borderRadius: 20,
                            background: 'rgba(255,204,0,0.08)',
                            border: '1px solid rgba(255,204,0,0.1)',
                            display: 'flex', alignItems: 'center', justifyContent: 'center',
                            fontSize: '24px',
                        }}>
                            🤖
                        </div>
                        <div style={{ fontSize: isSeniorOrChild ? '16px' : '13px', color: 'rgba(255,255,255,0.4)', fontWeight: 500 }}>
                            Ask me about this page
                        </div>
                        <div style={{ display: 'flex', flexDirection: 'column', gap: '6px', width: '100%' }}>
                            {['Summarize this page', 'What are the key points?', 'Is this trustworthy?'].map(hint => (
                                <button
                                    key={hint}
                                    onClick={() => ask(hint)}
                                    style={{
                                        background: 'rgba(255,255,255,0.03)',
                                        border: '1px solid rgba(255,255,255,0.06)',
                                        borderRadius: 10,
                                        padding: '8px 12px',
                                        color: 'rgba(255,255,255,0.35)',
                                        cursor: 'pointer',
                                        fontSize: isSeniorOrChild ? '14px' : '12px',
                                        textAlign: 'left',
                                    }}
                                >
                                    {hint}
                                </button>
                            ))}
                        </div>
                    </div>
                )}

                {messages.map(msg => (
                    <div key={msg.id} style={{
                        display: 'flex',
                        justifyContent: msg.sender === 'user' ? 'flex-end' : 'flex-start',
                    }}>
                        <div style={{
                            maxWidth: '90%',
                            padding: '10px 14px',
                            borderRadius: 16,
                            fontSize,
                            lineHeight: 1.5,
                            ...(msg.sender === 'user' ? {
                                background: 'linear-gradient(135deg, #0055d4, #003da0)',
                                color: '#fff',
                                borderTopRightRadius: 4,
                            } : {
                                background: 'rgba(255,255,255,0.05)',
                                border: '1px solid rgba(255,255,255,0.06)',
                                color: 'rgba(255,255,255,0.85)',
                                borderTopLeftRadius: 4,
                            }),
                        }}>
                            <div style={{ whiteSpace: 'pre-wrap' }}>{msg.content}</div>

                            {/* Sources */}
                            {msg.sources && msg.sources.length > 0 && (
                                <div style={{ marginTop: 8, paddingTop: 8, borderTop: '1px solid rgba(255,255,255,0.04)' }}>
                                    <div style={{ fontSize: '9px', fontWeight: 700, textTransform: 'uppercase', letterSpacing: '0.15em', color: 'rgba(255,255,255,0.25)', marginBottom: 4 }}>
                                        Sources
                                    </div>
                                    {msg.sources.map((s, i) => (
                                        <a key={i} href={s.url} target="_blank" rel="noopener noreferrer"
                                            style={{ display: 'block', fontSize: '11px', color: 'rgba(96,165,250,0.7)', textDecoration: 'none', marginBottom: 2 }}>
                                            🔗 {s.title}
                                        </a>
                                    ))}
                                </div>
                            )}
                        </div>
                    </div>
                ))}

                {isThinking && (
                    <div style={{ display: 'flex', justifyContent: 'flex-start' }}>
                        <div style={{
                            background: 'rgba(255,255,255,0.05)',
                            border: '1px solid rgba(255,255,255,0.06)',
                            borderRadius: 16, borderTopLeftRadius: 4,
                            padding: '10px 16px',
                            display: 'flex', gap: 4, alignItems: 'center',
                        }}>
                            <span style={{ animation: 'pulse 1.2s infinite', width: 6, height: 6, borderRadius: '50%', background: 'rgba(255,204,0,0.6)' }} />
                            <span style={{ animation: 'pulse 1.2s infinite 0.15s', width: 6, height: 6, borderRadius: '50%', background: 'rgba(96,165,250,0.6)' }} />
                            <span style={{ animation: 'pulse 1.2s infinite 0.3s', width: 6, height: 6, borderRadius: '50%', background: 'rgba(255,204,0,0.6)' }} />
                        </div>
                    </div>
                )}
            </div>

            {/* Input */}
            <div style={{
                padding: '12px',
                borderTop: '1px solid rgba(255,255,255,0.06)',
                background: 'rgba(10,18,37,0.8)',
            }}>
                <form onSubmit={(e) => { e.preventDefault(); ask(input); }} style={{ position: 'relative' }}>
                    <input
                        type="text"
                        value={input}
                        onChange={(e) => setInput(e.target.value)}
                        placeholder="Ask about this page..."
                        disabled={!connected || !pageContext}
                        style={{
                            width: '100%',
                            background: 'rgba(255,255,255,0.04)',
                            border: '1px solid rgba(255,255,255,0.08)',
                            borderRadius: 12,
                            padding: '10px 48px 10px 14px',
                            color: '#e2e8f0',
                            fontSize: isSeniorOrChild ? '15px' : '13px',
                            outline: 'none',
                        }}
                    />
                    <button
                        type="submit"
                        disabled={!input.trim() || isThinking || !connected}
                        style={{
                            position: 'absolute',
                            right: 4,
                            top: 4,
                            bottom: 4,
                            width: 36,
                            borderRadius: 10,
                            border: 'none',
                            background: input.trim() ? 'linear-gradient(135deg, #ffcc00, #f59e0b)' : 'rgba(255,255,255,0.04)',
                            color: input.trim() ? '#000' : 'rgba(255,255,255,0.15)',
                            cursor: input.trim() ? 'pointer' : 'default',
                            display: 'flex',
                            alignItems: 'center',
                            justifyContent: 'center',
                            fontSize: '14px',
                        }}
                    >
                        ➤
                    </button>
                </form>
                <div style={{
                    display: 'flex',
                    justifyContent: 'center',
                    gap: 8,
                    marginTop: 8,
                    fontSize: '8px',
                    textTransform: 'uppercase',
                    letterSpacing: '0.15em',
                    fontWeight: 700,
                    color: 'rgba(255,255,255,0.1)',
                }}>
                    <span>🔒 E2E Encrypted</span>
                    <span>·</span>
                    <span>Sovereign Runtime</span>
                </div>
            </div>

            <style>{`
                @keyframes pulse {
                    0%, 100% { transform: scale(0.8); opacity: 0.4; }
                    50% { transform: scale(1.2); opacity: 1; }
                }
                input::placeholder { color: rgba(255,255,255,0.2); }
                input:focus { border-color: rgba(255,204,0,0.3) !important; }
                button:hover { filter: brightness(1.1); }
                ::-webkit-scrollbar { width: 4px; }
                ::-webkit-scrollbar-track { background: transparent; }
                ::-webkit-scrollbar-thumb { background: rgba(255,255,255,0.08); border-radius: 4px; }
            `}</style>
        </div>
    );
};
