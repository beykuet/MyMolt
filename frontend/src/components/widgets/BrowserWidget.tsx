import React, { useState, useRef, useCallback } from 'react';
import { Globe, AlertTriangle, ShieldOff, Bot, MessageSquare } from 'lucide-react';
import { BrowserNavBar } from './BrowserNavBar';
import { BrowserAnswerPanel } from './BrowserAnswerPanel';
import { apiClient } from '../../api/client';
import { useAuth } from '../../context/AuthContext';
import { motion, AnimatePresence } from 'framer-motion';
import type { UserRole } from '../../types';

interface BrowserState {
    url: string;
    title: string;
    isLoading: boolean;
    isSecure: boolean;
    isBlocked: boolean;
    blockReason: string;
    pageText: string;
    htmlContent: string;
}

export const BrowserWidget: React.FC = () => {
    const { user } = useAuth();
    const role: UserRole = user?.role || 'Adult';
    const iframeRef = useRef<HTMLIFrameElement>(null);

    const [browser, setBrowser] = useState<BrowserState>({
        url: '',
        title: '',
        isLoading: false,
        isSecure: false,
        isBlocked: false,
        blockReason: '',
        pageText: '',
        htmlContent: '',
    });

    const [history, setHistory] = useState<string[]>([]);
    const [historyIndex, setHistoryIndex] = useState(-1);
    const [isAnswerOpen, setAnswerOpen] = useState(false);

    const isChildRole = role === 'Child';
    const isSenior = role === 'Senior';

    const navigate = useCallback(async (url: string) => {
        setBrowser(prev => ({ ...prev, url, isLoading: true, isBlocked: false, blockReason: '' }));

        // Push to history
        setHistory(prev => [...prev.slice(0, historyIndex + 1), url]);
        setHistoryIndex(prev => prev + 1);

        try {
            const res = await apiClient.fetch(`/browse/proxy?url=${encodeURIComponent(url)}&role=${role}`);
            if (res.ok) {
                const data = await res.json();
                if (data.blocked) {
                    setBrowser(prev => ({
                        ...prev,
                        isLoading: false,
                        isBlocked: true,
                        blockReason: data.reason || 'Content filtered',
                    }));
                } else {
                    setBrowser(prev => ({
                        ...prev,
                        isLoading: false,
                        isSecure: url.startsWith('https://'),
                        title: data.title || url,
                        pageText: data.text || '',
                        htmlContent: data.html || '',
                    }));
                }
            } else {
                setBrowser(prev => ({
                    ...prev,
                    isLoading: false,
                    isBlocked: true,
                    blockReason: 'Failed to load page',
                }));
            }
        } catch {
            setBrowser(prev => ({
                ...prev,
                isLoading: false,
                isBlocked: true,
                blockReason: 'Network error',
            }));
        }
    }, [role, historyIndex]);

    const goBack = () => {
        if (historyIndex > 0) {
            const newIdx = historyIndex - 1;
            setHistoryIndex(newIdx);
            navigate(history[newIdx]);
        }
    };

    const goForward = () => {
        if (historyIndex < history.length - 1) {
            const newIdx = historyIndex + 1;
            setHistoryIndex(newIdx);
            navigate(history[newIdx]);
        }
    };

    const refresh = () => {
        if (browser.url) navigate(browser.url);
    };

    const bookmark = async () => {
        if (!browser.url) return;
        try {
            await apiClient.fetch('/browse/bookmark', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ url: browser.url, title: browser.title }),
            });
        } catch { }
    };

    // Render blocked page
    const renderBlockedContent = () => {
        if (isChildRole) {
            return (
                <div className="flex flex-col items-center justify-center h-full text-center space-y-4 p-8">
                    <div className="w-24 h-24 rounded-full bg-gradient-to-br from-amber-500/20 to-orange-500/10 flex items-center justify-center">
                        <ShieldOff size={40} className="text-amber-400/70" />
                    </div>
                    <h2 className="text-2xl font-bold text-white/80">Oops! This page isn't for you</h2>
                    <p className="text-lg text-white/40 max-w-md">
                        This website has been blocked to keep you safe. Try visiting a different page!
                    </p>
                    <button
                        onClick={() => navigate('https://kids.kiddle.co')}
                        className="mt-4 px-6 py-3 bg-gradient-to-r from-green-500 to-emerald-500 text-white font-bold rounded-xl shadow-lg shadow-green-500/20 hover:shadow-green-500/40 transition text-lg"
                    >
                        🌟 Go to Kiddle Search
                    </button>
                </div>
            );
        }

        if (isSenior) {
            return (
                <div className="flex flex-col items-center justify-center h-full text-center space-y-4 p-8">
                    <AlertTriangle size={48} className="text-amber-400/70" />
                    <h2 className="text-2xl font-bold text-white/80">Page not available</h2>
                    <p className="text-lg text-white/40 max-w-md">{browser.blockReason}</p>
                    <button
                        onClick={() => setBrowser(prev => ({ ...prev, url: '', isBlocked: false }))}
                        className="mt-4 px-6 py-3 bg-white/[0.06] border border-white/[0.1] text-white rounded-xl hover:bg-white/[0.1] transition text-lg"
                    >
                        Go Back
                    </button>
                </div>
            );
        }

        return (
            <div className="flex flex-col items-center justify-center h-full text-center space-y-3 p-8">
                <AlertTriangle size={36} className="text-amber-400/60" />
                <h3 className="text-lg font-bold text-white/70">Blocked</h3>
                <p className="text-sm text-white/40">{browser.blockReason}</p>
                <button
                    onClick={() => setBrowser(prev => ({ ...prev, url: '', isBlocked: false }))}
                    className="mt-2 text-sm text-mymolt-yellow hover:text-yellow-300 transition"
                >
                    ← Go back
                </button>
            </div>
        );
    };

    // Render start page
    const renderStartPage = () => (
        <div className="flex flex-col items-center justify-center h-full text-center space-y-6 p-8">
            <div className="relative">
                <div className="w-20 h-20 rounded-3xl bg-gradient-to-br from-blue-500/15 to-indigo-500/10 flex items-center justify-center border border-blue-500/10">
                    <Globe size={36} strokeWidth={1.2} className="text-blue-400/50" />
                </div>
                <div className="absolute -inset-6 bg-gradient-to-r from-blue-500/5 via-transparent to-yellow-500/5 rounded-full blur-2xl -z-10" />
            </div>
            <div>
                <h2 className={`font-bold text-white/60 mb-1 ${isSenior || isChildRole ? 'text-2xl' : 'text-xl'}`}>
                    Sovereign Browser
                </h2>
                <p className={`text-white/25 ${isSenior || isChildRole ? 'text-lg' : 'text-sm'}`}>
                    Browse safely. Ask MyMolt about any page.
                </p>
            </div>
            <div className="flex flex-wrap gap-2 justify-center max-w-lg">
                {[
                    { label: '📰 News', url: 'https://news.ycombinator.com' },
                    { label: '📖 Wikipedia', url: 'https://en.wikipedia.org' },
                    { label: '🔍 Search', url: 'https://search.brave.com' },
                    { label: '🌤️ Weather', url: 'https://wttr.in' },
                ].map(item => (
                    <button
                        key={item.url}
                        onClick={() => navigate(item.url)}
                        className={`px-4 py-2 rounded-xl bg-white/[0.04] border border-white/[0.06] text-white/35 hover:text-white/70 hover:bg-white/[0.08] hover:border-white/[0.1] transition ${isSenior || isChildRole ? 'text-lg' : 'text-sm'}`}
                    >
                        {item.label}
                    </button>
                ))}
            </div>
        </div>
    );

    return (
        <div className="flex h-full rounded-2xl overflow-hidden bg-[#0a0e18] border border-white/[0.06]">
            {/* Main browser area */}
            <div className="flex-1 flex flex-col min-w-0">
                <BrowserNavBar
                    url={browser.url}
                    onNavigate={navigate}
                    onBack={goBack}
                    onForward={goForward}
                    onRefresh={refresh}
                    onBookmark={bookmark}
                    isLoading={browser.isLoading}
                    isSecure={browser.isSecure}
                    isFiltered={isChildRole}
                />

                {/* Content area */}
                <div className="flex-1 relative bg-[#0d1117] overflow-hidden">
                    {!browser.url && !browser.isBlocked && renderStartPage()}
                    {browser.isBlocked && renderBlockedContent()}

                    {browser.url && !browser.isBlocked && browser.htmlContent && (
                        <iframe
                            ref={iframeRef}
                            srcDoc={browser.htmlContent}
                            title={browser.title}
                            className="w-full h-full border-0 bg-white"
                            sandbox="allow-same-origin"
                        />
                    )}

                    {browser.isLoading && (
                        <div className="absolute inset-0 bg-[#0d1117]/80 flex items-center justify-center">
                            <div className="w-8 h-8 border-2 border-mymolt-yellow/30 border-t-mymolt-yellow rounded-full animate-spin" />
                        </div>
                    )}
                </div>

                {/* Ask MyMolt bar — visible when a page is loaded */}
                <AnimatePresence>
                    {browser.url && !browser.isBlocked && !browser.isLoading && !isAnswerOpen && (
                        <motion.div
                            initial={{ y: 50, opacity: 0 }}
                            animate={{ y: 0, opacity: 1 }}
                            exit={{ y: 50, opacity: 0 }}
                            className="p-3 border-t border-white/[0.06] bg-gradient-to-t from-[#0a1225] to-[#0f1520]"
                        >
                            <button
                                onClick={() => setAnswerOpen(true)}
                                className="w-full flex items-center gap-3 px-4 py-3 bg-white/[0.03] border border-white/[0.06] rounded-xl hover:bg-white/[0.06] hover:border-mymolt-yellow/20 transition group"
                            >
                                <div className="w-7 h-7 rounded-lg bg-gradient-to-br from-mymolt-yellow/20 to-amber-500/10 flex items-center justify-center border border-mymolt-yellow/10 group-hover:from-mymolt-yellow/30">
                                    <Bot size={14} className="text-mymolt-yellow/70" />
                                </div>
                                <span className={`text-white/30 group-hover:text-white/50 transition ${isSenior || isChildRole ? 'text-base' : 'text-sm'}`}>
                                    Ask MyMolt about this page...
                                </span>
                                <MessageSquare size={14} className="ml-auto text-white/15 group-hover:text-mymolt-yellow/50 transition" />
                            </button>
                        </motion.div>
                    )}
                </AnimatePresence>
            </div>

            {/* Answer panel — slides in from right */}
            <BrowserAnswerPanel
                isOpen={isAnswerOpen}
                onClose={() => setAnswerOpen(false)}
                pageUrl={browser.url}
                pageText={browser.pageText}
                role={role}
            />
        </div>
    );
};
