import React, { useState, useEffect } from 'react';
import { Shield, Wifi, Globe, Zap, Settings, Database, CheckCircle2 } from 'lucide-react';
import { motion } from 'framer-motion';
import { apiClient } from '../../api/client';

interface AdBlockState {
    enabled: boolean;
    count: number;
}

export const AdBlockWidget: React.FC = () => {
    const [status, setStatus] = useState<AdBlockState>({ enabled: false, count: 0 });

    const fetchStatus = async () => {
        try {
            const data = await apiClient.get<AdBlockState>('/api/config/adblock');
            setStatus(data);
        } catch (err) {
            console.error(err);
        }
    };

    useEffect(() => {
        fetchStatus();
        const interval = setInterval(fetchStatus, 5000);
        return () => clearInterval(interval);
    }, []);

    const handleToggle = async () => {
        try {
            const next = !status.enabled;
            await apiClient.post('/api/config/adblock/toggle', { enabled: next });
            setStatus(prev => ({ ...prev, enabled: next }));
        } catch (err) {
            console.error(err);
        }
    };

    return (
        <div className="flex flex-col h-full bg-mymolt-glass border border-mymolt-glassBorder backdrop-blur-xl rounded-3xl overflow-hidden shadow-2xl transition-all duration-500 font-sans">
            {/* Dynamic Background Effect */}
            <div className={`absolute inset-0 transition-opacity duration-1000 ${status.enabled ? 'opacity-20' : 'opacity-0'}`}>
                <div className="absolute inset-0 bg-mymolt-blue blur-[100px] -translate-y-1/2 rounded-full" />
            </div>

            <div className="p-8 relative z-10 flex flex-col h-full">
                <div className="flex items-center justify-between mb-8">
                    <div className="flex items-center gap-4">
                        <div className={`p-4 rounded-2xl transition-all duration-500 shadow-lg ${status.enabled ? 'bg-mymolt-blue shadow-mymolt-blue/50' : 'bg-black/40 border border-white/10'}`}>
                            <Shield className="text-white" size={32} />
                        </div>
                        <div>
                            <h2 className="text-2xl font-bold text-white tracking-tight">DNS Shield</h2>
                            <p className="text-[10px] text-mymolt-text-muted/40 uppercase tracking-[0.2em] font-black">Network-wide Protection</p>
                        </div>
                    </div>

                    <button
                        onClick={handleToggle}
                        className={`
              relative w-20 h-10 rounded-full transition-all duration-500 p-1
              ${status.enabled ? 'bg-mymolt-blue' : 'bg-black/40 border border-white/10'}
            `}
                    >
                        <motion.div
                            animate={{ x: status.enabled ? 40 : 0 }}
                            className="w-8 h-8 bg-white rounded-full shadow-lg flex items-center justify-center font-bold"
                        >
                            <Zap size={14} className={status.enabled ? 'text-mymolt-blue' : 'text-white/20'} />
                        </motion.div>
                    </button>
                </div>

                {/* Stats Grid */}
                <div className="grid grid-cols-2 gap-4 mb-8">
                    <div className="bg-white/5 border border-white/10 p-5 rounded-2xl transition-transform hover:scale-[1.02]">
                        <div className="flex items-center gap-3 mb-2 text-white/50">
                            <Database size={16} />
                            <span className="text-[10px] uppercase font-bold tracking-wider">Blocked Assets</span>
                        </div>
                        <div className="text-3xl font-mono font-black text-mymolt-yellow tracking-tighter shadow-mymolt-yellow/10">
                            {status.count.toLocaleString()}
                        </div>
                    </div>
                    <div className="bg-white/5 border border-white/10 p-5 rounded-2xl transition-transform hover:scale-[1.02]">
                        <div className="flex items-center gap-3 mb-2 text-white/50">
                            <Globe size={16} />
                            <span className="text-[10px] uppercase font-bold tracking-wider">Network Mode</span>
                        </div>
                        <div className="text-xl font-bold text-white">
                            {status.enabled ? 'Stealth' : 'Bypass'}
                        </div>
                    </div>
                </div>

                {/* List of active list */}
                <div className="flex-1 space-y-3 overflow-y-auto pr-2">
                    <label className="text-[10px] uppercase font-bold text-white/30 tracking-widest block mb-4">Active Context Blockers</label>
                    {[
                        { name: 'OISD Full', status: 'active', count: '45.2k' },
                        { name: 'StevenBlack Unified', status: 'active', count: '12.1k' },
                        { name: 'Gambling & Fraud', status: 'monitoring', count: '8.4k' },
                        { name: 'Contextual AI trackers', status: 'active', count: '1.2k' }
                    ].map((list) => (
                        <div key={list.name} className="flex items-center justify-between p-4 bg-white/5 rounded-2xl border border-white/5 group hover:border-white/20 transition-all">
                            <div className="flex items-center gap-3">
                                <div className={`p-2 rounded-xl transition-colors ${list.status === 'active' ? 'bg-mymolt-blue/20 text-mymolt-blue border border-mymolt-blue/20' : 'bg-black/40 text-white/10 border border-white/5'}`}>
                                    <CheckCircle2 size={16} />
                                </div>
                                <div>
                                    <div className="text-sm font-bold text-white group-hover:text-mymolt-blue transition-colors">{list.name}</div>
                                    <div className="text-[10px] text-mymolt-text-muted/30 font-mono italic font-medium uppercase tracking-wider">{list.count} definitions</div>
                                </div>
                            </div>
                            <div className="opacity-0 group-hover:opacity-100 transition-opacity">
                                <Settings size={14} className="text-white/30 hover:text-white cursor-pointer" />
                            </div>
                        </div>
                    ))}
                </div>

                {/* Footer */}
                <div className="mt-8 pt-6 border-t border-white/10 flex items-center justify-between">
                    <div className="flex items-center gap-2 text-white/30 text-[10px] font-bold uppercase tracking-wider">
                        <Wifi size={12} className={status.enabled ? 'text-green-500' : ''} />
                        Resolver: 10.100.0.1
                    </div>
                    <div className="flex items-center gap-2 text-white/30 text-[10px] font-bold uppercase tracking-wider">
                        <Zap size={12} className="text-amber-500" />
                        Latency: 1.2ms
                    </div>
                </div>
            </div>
        </div>
    );
};
