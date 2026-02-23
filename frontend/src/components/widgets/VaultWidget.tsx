import { useState, useEffect } from 'react';
import { Lock, FileText, Download, Activity, Clock } from 'lucide-react';
import { apiClient } from '../../api/client';

interface VaultEntry {
    id: string;
    description: string;
    created_at: string;
    tags: string[];
}

export function VaultWidget() {
    const [entries, setEntries] = useState<VaultEntry[]>([]);
    const [loading, setLoading] = useState(true);

    const fetchVault = async () => {
        try {
            const res = await apiClient.fetch('/vault');
            if (res.ok) {
                const data = await res.json();
                setEntries(data.slice(0, 5)); // Only show latest 5
            }
        } catch (e) {
            console.error(e);
        } finally {
            setLoading(false);
        }
    };

    useEffect(() => {
        fetchVault();
    }, []);

    return (
        <div className="glass-panel p-6 h-[500px]">
            <h3 className="font-black text-xl mb-6 flex items-center gap-3">
                <Lock size={20} className="text-mymolt-yellow" /> Secure Vault Audit
            </h3>

            <div className="space-y-3">
                {loading ? (
                    <div className="text-center py-8 opacity-50"><Activity className="animate-spin mx-auto mb-2" /> Decrypting Index...</div>
                ) : entries.length === 0 ? (
                    <div className="text-center py-8 text-mymolt-text-muted text-sm border-2 border-dashed border-white/10 rounded-xl">
                        No encrypted memories found yet.
                    </div>
                ) : (
                    entries.map((entry) => (
                        <div key={entry.id} className="group relative p-5 bg-black/20 rounded-2xl border border-mymolt-glassBorder hover:border-mymolt-yellow/30 transition-all shadow-lg">
                            <div className="flex items-start justify-between">
                                <div className="flex items-center gap-3">
                                    <div className="p-3 bg-mymolt-yellow/10 rounded-xl text-mymolt-yellow border border-mymolt-yellow/20">
                                        <FileText size={20} />
                                    </div>
                                    <div>
                                        <h4 className="font-medium text-sm truncate max-w-[180px]">{entry.description}</h4>
                                        <div className="flex items-center gap-2 mt-1">
                                            <span className="text-[10px] text-mymolt-text-muted/40 uppercase font-black tracking-widest flex items-center gap-1.5">
                                                <Clock size={10} /> {new Date(entry.created_at).toLocaleDateString()}
                                            </span>
                                            {entry.tags.map(t => (
                                                <span key={t} className="text-[10px] bg-white/10 px-1.5 py-0.5 rounded text-purple-200">
                                                    #{t}
                                                </span>
                                            ))}
                                        </div>
                                    </div>
                                </div>

                                <button className="p-2 hover:bg-mymolt-yellow/10 rounded-xl opacity-0 group-hover:opacity-100 transition-all text-mymolt-yellow" title="Download Decrypted">
                                    <Download size={18} />
                                </button>
                            </div>
                        </div>
                    ))
                )}
            </div>

            <div className="mt-4 pt-4 border-t border-white/10 flex justify-between items-center text-xs text-mymolt-text-muted">
                <span>{entries.length} Secure Items</span>
                <button className="hover:text-white transition-colors">View All →</button>
            </div>
        </div>
    );
}
