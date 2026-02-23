import React, { useState } from 'react';
import { ArrowLeft, ArrowRight, RotateCw, Lock, Shield, Star, Search } from 'lucide-react';

interface BrowserNavBarProps {
    url: string;
    onNavigate: (url: string) => void;
    onBack: () => void;
    onForward: () => void;
    onRefresh: () => void;
    onBookmark: () => void;
    isLoading: boolean;
    isSecure: boolean;
    isFiltered: boolean;
}

export const BrowserNavBar: React.FC<BrowserNavBarProps> = ({
    url,
    onNavigate,
    onBack,
    onForward,
    onRefresh,
    onBookmark,
    isLoading,
    isSecure,
    isFiltered,
}) => {
    const [inputValue, setInputValue] = useState(url);
    const [isFocused, setIsFocused] = useState(false);

    const handleSubmit = (e: React.FormEvent) => {
        e.preventDefault();
        let nav = inputValue.trim();
        if (!nav) return;
        // Auto-prefix with https if no protocol
        if (!/^https?:\/\//i.test(nav)) {
            // If it looks like a URL, add https
            if (/\.\w{2,}/.test(nav)) {
                nav = 'https://' + nav;
            } else {
                // Treat as search query
                nav = `https://search.brave.com/search?q=${encodeURIComponent(nav)}`;
            }
        }
        onNavigate(nav);
        setInputValue(nav);
    };

    // Sync external url changes
    React.useEffect(() => {
        if (!isFocused) setInputValue(url);
    }, [url, isFocused]);

    return (
        <div className="flex items-center gap-2 px-3 py-2.5 bg-[#141822] border-b border-white/[0.06]">
            {/* Nav buttons */}
            <div className="flex items-center gap-0.5">
                <button
                    onClick={onBack}
                    className="p-2 rounded-lg hover:bg-white/[0.06] text-white/30 hover:text-white/60 transition"
                >
                    <ArrowLeft size={16} />
                </button>
                <button
                    onClick={onForward}
                    className="p-2 rounded-lg hover:bg-white/[0.06] text-white/30 hover:text-white/60 transition"
                >
                    <ArrowRight size={16} />
                </button>
                <button
                    onClick={onRefresh}
                    className={`p-2 rounded-lg hover:bg-white/[0.06] text-white/30 hover:text-white/60 transition ${isLoading ? 'animate-spin' : ''}`}
                >
                    <RotateCw size={16} />
                </button>
            </div>

            {/* URL bar */}
            <form onSubmit={handleSubmit} className="flex-1 relative">
                <div className={`flex items-center gap-2 bg-[#0d1117] rounded-xl px-3 py-2 border transition-all ${isFocused ? 'border-mymolt-yellow/30 ring-1 ring-mymolt-yellow/10' : 'border-white/[0.06]'
                    }`}>
                    {!isFocused && url && (
                        <span className="flex items-center gap-1 flex-shrink-0">
                            {isSecure ? (
                                <Lock size={12} className="text-emerald-400" />
                            ) : (
                                <Search size={12} className="text-white/20" />
                            )}
                        </span>
                    )}
                    <input
                        type="text"
                        value={inputValue}
                        onChange={(e) => setInputValue(e.target.value)}
                        onFocus={() => { setIsFocused(true); }}
                        onBlur={() => setIsFocused(false)}
                        placeholder="Search or enter URL..."
                        className="flex-1 bg-transparent text-sm text-white/80 placeholder-white/20 outline-none font-mono"
                    />
                    {isLoading && (
                        <div className="w-4 h-4 border-2 border-mymolt-yellow/30 border-t-mymolt-yellow rounded-full animate-spin flex-shrink-0" />
                    )}
                </div>
            </form>

            {/* Right side controls */}
            <div className="flex items-center gap-0.5">
                {isFiltered && (
                    <div className="flex items-center gap-1 px-2 py-1 rounded-lg bg-emerald-500/10 text-emerald-400 mr-1">
                        <Shield size={12} />
                        <span className="text-[9px] font-bold uppercase tracking-widest">Filtered</span>
                    </div>
                )}
                <button
                    onClick={onBookmark}
                    className="p-2 rounded-lg hover:bg-white/[0.06] text-white/30 hover:text-mymolt-yellow transition"
                >
                    <Star size={16} />
                </button>
            </div>
        </div>
    );
};
