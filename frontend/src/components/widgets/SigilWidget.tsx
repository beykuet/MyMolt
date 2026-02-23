import React, { useState, useEffect } from 'react';
import { Eye, EyeOff, ShieldCheck, Lock, ArrowRight, Activity, Filter } from 'lucide-react';
import { motion, AnimatePresence } from 'framer-motion';
import { apiClient } from '../../api/client';

interface SigilLog {
    timestamp: string;
    event_id?: string;
    id?: string;
    event_type?: string;
    action?: any;
    resource?: string;
    status?: boolean;
}

export const SigilWidget: React.FC = () => {
    const [logs, setLogs] = useState<SigilLog[]>([]);
    const [revealed, setRevealed] = useState<Record<string, boolean>>({});
    const [loading, setLoading] = useState(true);

    useEffect(() => {
        const fetchLogs = async () => {
            try {
                const data = await apiClient.get<SigilLog[]>('/api/security/sigil');
                setLogs(data);
            } catch (err) {
                console.error('Failed to fetch Sigil logs', err);
            } finally {
                setLoading(false);
            }
        };
        fetchLogs();
    }, []);

    const toggleReveal = (id: string) => {
        setRevealed(prev => ({ ...prev, [id]: !prev[id] }));
    };

    return (
        <div className="flex flex-col h-full bg-mymolt-glass backdrop-blur-xl border border-mymolt-glassBorder rounded-3xl overflow-hidden shadow-2xl text-white font-sans">
            {/* Header */}
            <div className="p-8 border-b border-mymolt-glassBorder bg-gradient-to-r from-emerald-600/10 to-mymolt-blue/10 flex items-center justify-between">
                <div>
                    <h2 className="text-xl font-bold flex items-center gap-2">
                        <ShieldCheck className="text-emerald-400" />
                        Security Audit Log
                    </h2>
                    <p className="text-xs text-white/50 mt-1 uppercase tracking-widest font-medium">
                        Sigil Interceptions & Human Approvals
                    </p>
                </div>
                <div className="flex gap-4">
                    <div className="text-right">
                        <div className="text-2xl font-mono font-bold text-emerald-400">{logs.length}</div>
                        <div className="text-[10px] text-white/30 uppercase font-bold">Total Events</div>
                    </div>
                </div>
            </div>

            {/* Grid Header */}
            <div className="px-6 py-3 bg-white/5 border-b border-white/5 grid grid-cols-12 text-[10px] uppercase tracking-wider font-bold text-white/30">
                <div className="col-span-3">Timestamp</div>
                <div className="col-span-4">Operation / Reason</div>
                <div className="col-span-5 text-right">Protection Status</div>
            </div>

            {/* Log List */}
            <div className="flex-1 overflow-y-auto p-4 space-y-3">
                {loading ? (
                    <div className="h-full flex flex-col items-center justify-center opacity-20">
                        <Activity className="animate-pulse mb-2" size={40} />
                        <p>Establishing Secure Stream...</p>
                    </div>
                ) : logs.length === 0 ? (
                    <div className="h-full flex flex-col items-center justify-center opacity-20 border-2 border-dashed border-white/10 rounded-2xl">
                        <Filter size={48} className="mb-4" />
                        <p className="text-xl font-light">No data intercepted yet</p>
                        <p className="text-sm mt-2">All data flows are currently safe</p>
                    </div>
                ) : (
                    logs.map((log) => {
                        const internalId = log.id || log.event_id || Math.random().toString();
                        const isApproval = log.event_type === 'SecurityEvent';

                        return (
                            <motion.div
                                initial={{ opacity: 0, y: 10 }}
                                animate={{ opacity: 1, y: 0 }}
                                key={internalId}
                                className={`bg-white/5 border rounded-2xl p-4 transition-all group ${isApproval ? 'border-mymolt-warning/20 hover:bg-mymolt-warning/5' : 'border-white/10 hover:bg-white/10'}`}
                            >
                                <div className="grid grid-cols-12 items-center gap-4">
                                    <div className="col-span-3 text-xs font-mono text-white/50">
                                        {new Date(log.timestamp).toLocaleString().replace(/Invalid Date/, 'Just now')}
                                    </div>
                                    <div className="col-span-4 flex items-center gap-2">
                                        <div className={`p-2 rounded-lg ${isApproval ? 'bg-mymolt-warning/20 text-mymolt-warning' : 'bg-emerald-500/20 text-emerald-400'}`}>
                                            {isApproval ? <Activity size={14} /> : <Lock size={14} />}
                                        </div>
                                        <span className="text-sm font-medium">
                                            {isApproval ? 'Human-in-the-Loop Auth' : (typeof log.action === 'string' ? log.action : log.action?.command || 'Sensitive Content')}
                                        </span>
                                    </div>
                                    <div className="col-span-5 flex items-center justify-end gap-3">
                                        <div className={`flex items-center gap-1.5 px-3 py-1 rounded-full border text-[10px] font-bold uppercase ${isApproval ? (log.status ? 'bg-mymolt-success/10 border-mymolt-success/20 text-mymolt-success' : 'bg-red-500/10 border-red-500/20 text-red-500') : 'bg-emerald-500/10 border-emerald-500/20 text-emerald-400'
                                            }`}>
                                            <ShieldCheck size={12} />
                                            {isApproval ? (log.status ? 'Approved' : 'Denied') : 'Anonymized'}
                                        </div>
                                        {!isApproval && (
                                            <button
                                                onClick={() => toggleReveal(internalId)}
                                                className="p-2 hover:bg-white/10 rounded-xl transition-colors text-white/50 hover:text-white"
                                            >
                                                {revealed[internalId] ? <EyeOff size={18} /> : <Eye size={18} />}
                                            </button>
                                        )}
                                    </div>
                                </div>

                                <AnimatePresence>
                                    {revealed[internalId] && !isApproval && (
                                        <motion.div
                                            initial={{ height: 0, opacity: 0 }}
                                            animate={{ height: 'auto', opacity: 1 }}
                                            exit={{ height: 0, opacity: 0 }}
                                            className="mt-4 pt-4 border-t border-white/5 overflow-hidden"
                                        >
                                            <div className="grid grid-cols-2 gap-4">
                                                <div className="space-y-2">
                                                    <label className="text-[10px] uppercase font-bold text-red-400/50">Original (Risky)</label>
                                                    <div className="p-3 bg-red-500/5 border border-red-500/20 rounded-xl text-sm font-mono text-red-200 break-all">
                                                        sk-proj-4fG8... (Redacted Sensitive Key)
                                                    </div>
                                                </div>
                                                <div className="space-y-2 flex flex-col items-center justify-center">
                                                    <ArrowRight className="text-white/20" />
                                                </div>
                                                <div className="space-y-2 absolute right-8 w-[40%]">
                                                    <label className="text-[10px] uppercase font-bold text-emerald-400/50">Dummy (Safe)</label>
                                                    <div className="p-3 bg-emerald-500/5 border border-emerald-500/20 rounded-xl text-sm font-mono text-emerald-200">
                                                        [VAULT: API Key]
                                                    </div>
                                                </div>
                                            </div>
                                        </motion.div>
                                    )}
                                </AnimatePresence>
                            </motion.div>
                        );
                    })
                )}
            </div>

            {/* Footer info */}
            <div className="p-4 bg-white/5 text-[10px] text-center text-white/30 uppercase tracking-[0.2em]">
                Data protection is enforced at the Memory Layer by Sovereign Runtime v1.0
            </div>
        </div>
    );
};
