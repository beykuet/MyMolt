import { useState, useEffect } from 'react';
import { Book, Send } from 'lucide-react';
import { apiClient } from '../../api/client';

interface DiaryEntry {
    timestamp: string;
    content: string;
}

export function DiaryWidget() {
    const [entries, setEntries] = useState<DiaryEntry[]>([]);
    const [input, setInput] = useState('');
    const [loading, setLoading] = useState(false);

    const fetchEntries = async () => {
        try {
            const res = await apiClient.fetch('/soul/diary');
            if (res.ok) setEntries(await res.json());
        } catch (e) {
            console.error(e);
        }
    };

    useEffect(() => {
        fetchEntries();
        // Poll for updates (e.g. from Voice Mode)
        const interval = setInterval(fetchEntries, 10000);
        return () => clearInterval(interval);
    }, []);

    const handleSubmit = async (e: React.FormEvent) => {
        e.preventDefault();
        if (!input.trim()) return;
        setLoading(true);

        try {
            await apiClient.fetch('/soul/diary', {
                method: 'POST',
                body: JSON.stringify({ content: input })
            });
            setInput('');
            fetchEntries();
        } finally {
            setLoading(false);
        }
    };

    return (
        <div className="glass-panel p-6 row-span-2 flex flex-col h-[500px]">
            <h3 className="font-black text-xl mb-6 flex items-center gap-3">
                <Book size={20} className="text-mymolt-yellow" /> My Soul Diary
            </h3>

            <div className="flex-1 overflow-y-auto space-y-4 pr-2 custom-scrollbar">
                {entries.length === 0 ? (
                    <div className="text-mymolt-text-muted text-sm italic py-4">
                        Your diary is empty. Start writing...
                    </div>
                ) : (
                    entries.map((entry, i) => (
                        <div key={i} className="p-4 bg-black/20 rounded-2xl border border-mymolt-glassBorder animate-fade-in-up shadow-lg" style={{ animationDelay: `${i * 0.1}s` }}>
                            <span className="text-[10px] font-black text-white/30 uppercase tracking-widest">{entry.timestamp}</span>
                            <p className="text-sm mt-2 leading-relaxed">{entry.content}</p>
                        </div>
                    ))
                )}
            </div>

            <form onSubmit={handleSubmit} className="mt-4 relative">
                <input
                    type="text"
                    value={input}
                    onChange={(e) => setInput(e.target.value)}
                    placeholder="Capture a heart-memory..."
                    className="w-full bg-black/20 border border-mymolt-glassBorder rounded-2xl pl-6 pr-12 py-4 text-sm focus:outline-none focus:border-mymolt-yellow transition-all placeholder:text-white/10 shadow-inner"
                    disabled={loading}
                />
                <button
                    type="submit"
                    disabled={loading}
                    className="absolute right-2 top-1/2 -translate-y-1/2 p-1.5 text-mymolt-yellow hover:bg-white/10 rounded-full transition"
                >
                    <Send size={16} />
                </button>
            </form>
        </div>
    );
}
