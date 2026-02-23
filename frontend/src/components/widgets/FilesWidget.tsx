import { useState, useEffect, useRef } from 'react';
import { HardDrive, RefreshCw, ExternalLink, ShieldCheck, AlertCircle } from 'lucide-react';

type LoadState = 'loading' | 'ready' | 'error';

export function FilesWidget() {
    const [loadState, setLoadState] = useState<LoadState>('loading');
    const iframeRef = useRef<HTMLIFrameElement>(null);

    // Check if Hoodik is reachable
    useEffect(() => {
        const checkHoodik = async () => {
            try {
                await fetch('/system/', { method: 'HEAD', mode: 'no-cors' });
                // no-cors will succeed even if opaque — we just need the request to not throw
                setLoadState('ready');
            } catch {
                setLoadState('error');
            }
        };

        const timer = setTimeout(checkHoodik, 1000); // Give Hoodik 1s to initialize
        return () => clearTimeout(timer);
    }, []);

    const handleRetry = () => {
        setLoadState('loading');
        setTimeout(() => {
            if (iframeRef.current) {
                iframeRef.current.src = '/system/';
            }
            setLoadState('ready');
        }, 500);
    };

    return (
        <div className="h-full flex flex-col">
            {/* Header */}
            <div className="flex items-center justify-between mb-6">
                <div className="flex items-center gap-4">
                    <div className="p-3 bg-mymolt-yellow/10 rounded-2xl border border-mymolt-yellow/20">
                        <HardDrive size={24} className="text-mymolt-yellow" />
                    </div>
                    <div>
                        <h2 className="text-2xl font-black tracking-tight bg-clip-text text-transparent bg-gradient-to-r from-white to-white/60">
                            Sovereign Files
                        </h2>
                        <div className="flex items-center gap-2 mt-1">
                            <ShieldCheck size={12} className="text-green-400" />
                            <span className="text-[10px] font-black text-green-400/80 uppercase tracking-widest">
                                End-to-End Encrypted
                            </span>
                        </div>
                    </div>
                </div>

                <div className="flex items-center gap-3">
                    <button
                        onClick={handleRetry}
                        className="p-2.5 hover:bg-white/5 rounded-xl transition-all text-white/40 hover:text-white group"
                        title="Refresh"
                    >
                        <RefreshCw size={18} className="group-hover:rotate-180 transition-transform duration-500" />
                    </button>
                    <a
                        href="/system/"
                        target="_blank"
                        rel="noopener noreferrer"
                        className="flex items-center gap-2 px-4 py-2 bg-white/5 border border-white/10 rounded-xl hover:bg-white/10 transition-all text-sm text-white/60 hover:text-white"
                    >
                        <ExternalLink size={14} />
                        <span className="font-bold text-xs">Open Full</span>
                    </a>
                </div>
            </div>

            {/* Content */}
            <div className="flex-1 relative rounded-2xl overflow-hidden border border-mymolt-glassBorder bg-black/20 shadow-2xl">
                {loadState === 'loading' && (
                    <div className="absolute inset-0 flex flex-col items-center justify-center bg-mymolt-glass backdrop-blur-xl z-10">
                        <div className="relative">
                            <div className="w-16 h-16 rounded-full border-2 border-mymolt-yellow/20 border-t-mymolt-yellow animate-spin" />
                            <HardDrive size={24} className="absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 text-mymolt-yellow" />
                        </div>
                        <p className="mt-6 text-sm font-bold text-white/40">Initializing Secure Storage...</p>
                        <p className="mt-1 text-[10px] font-black text-white/20 uppercase tracking-widest">Connecting to Hoodik Engine</p>
                    </div>
                )}

                {loadState === 'error' && (
                    <div className="absolute inset-0 flex flex-col items-center justify-center bg-mymolt-glass backdrop-blur-xl z-10">
                        <div className="p-5 bg-red-500/10 rounded-2xl border border-red-500/20 mb-4">
                            <AlertCircle size={32} className="text-red-400" />
                        </div>
                        <p className="text-sm font-bold text-white/60">Storage Engine Offline</p>
                        <p className="mt-1 text-[10px] text-white/30 max-w-xs text-center">
                            The Hoodik secure storage is initializing. This may take a few seconds on first boot.
                        </p>
                        <button
                            onClick={handleRetry}
                            className="mt-6 px-6 py-2.5 bg-mymolt-yellow/10 border border-mymolt-yellow/20 rounded-xl text-mymolt-yellow font-bold text-sm hover:bg-mymolt-yellow/20 transition-all"
                        >
                            Retry Connection
                        </button>
                    </div>
                )}

                {(loadState === 'ready' || loadState === 'loading') && (
                    <iframe
                        ref={iframeRef}
                        src="/system/"
                        className="w-full h-full border-0 min-h-[600px]"
                        title="Sovereign Files — Hoodik E2EE Storage"
                        onLoad={() => setLoadState('ready')}
                        onError={() => setLoadState('error')}
                        sandbox="allow-same-origin allow-scripts allow-forms allow-popups allow-modals"
                    />
                )}
            </div>
        </div>
    );
}
