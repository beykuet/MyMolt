import { useEffect, useState } from 'react';
import { apiClient } from '../api/client';
import type { Integration } from '../types';
import { Plug, CheckCircle2, Circle, Activity, Radio } from 'lucide-react';
import IntegrationConfigModal from '../components/modals/IntegrationConfigModal';

// Mock live metrics generator for Phase 4 display
const useChannelMetrics = (activeChannels: Integration[]) => {
    const [metrics, setMetrics] = useState<Record<string, { volume: number, rateLimit: number }>>({});
    useEffect(() => {
        if (activeChannels.length === 0) return;
        const interval = setInterval(() => {
            setMetrics(prev => {
                const updated = { ...prev };
                activeChannels.forEach(ch => {
                    const current = prev[ch.name] || { volume: 0, rateLimit: Math.floor(Math.random() * 20) };
                    updated[ch.name] = {
                        volume: current.volume + Math.floor(Math.random() * 3),
                        rateLimit: Math.max(0, Math.min(100, current.rateLimit + (Math.random() > 0.5 ? 5 : -5)))
                    };
                });
                return updated;
            });
        }, 3000);
        return () => clearInterval(interval);
    }, [activeChannels]);
    return metrics;
};

export default function Integrations() {
    const [integrations, setIntegrations] = useState<Integration[]>([]);
    const [loading, setLoading] = useState(true);
    const [selectedIntegration, setSelectedIntegration] = useState<Integration | null>(null);

    const fetchIntegrations = async () => {
        try {
            const data = await apiClient.get<Integration[]>('/api/integrations');
            setIntegrations(data);
        } catch (e) {
            console.error(e);
        } finally {
            setLoading(false);
        }
    };

    useEffect(() => {
        fetchIntegrations();
    }, []);

    const categories = Array.from(new Set(integrations.map(i => i.category)));

    if (loading) return <div className="p-6 text-white font-black uppercase tracking-widest animate-pulse">Synchronizing Store...</div>;

    return (
        <div className="p-6 space-y-8 text-white h-full overflow-y-auto">
            <h2 className="text-4xl font-black flex items-center gap-4 mb-12 tracking-tight">
                <Plug className="w-10 h-10 text-mymolt-primary" />
                Integration System
            </h2>

            {/* Communication Channels Manager (Phase 4) */}
            {integrations.filter(i => i.category === 'Chat' && i.status === 'active').length > 0 && (
                <div className="mb-16 bg-[#1a1f2e] border border-white/10 rounded-3xl p-8 shadow-2xl relative overflow-hidden">
                    <div className="absolute top-0 right-0 p-12 opacity-5 pointer-events-none">
                        <Radio size={200} />
                    </div>
                    <div className="flex items-center gap-3 mb-8 relative z-10">
                        <Activity className="text-mymolt-primary" size={24} />
                        <h3 className="text-xl font-bold tracking-tight">Communication Channels Manager</h3>
                    </div>
                    <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6 relative z-10">
                        {integrations.filter(i => i.category === 'Chat' && i.status === 'active').map(ch => {
                            const { volume = 0, rateLimit = 0 } = useChannelMetrics(integrations.filter(i => i.category === 'Chat' && i.status === 'active'))[ch.name] || {};
                            return (
                                <div key={ch.name + '-metric'} className="bg-black/30 border border-white/5 rounded-2xl p-6">
                                    <div className="flex justify-between items-center mb-4">
                                        <h4 className="font-bold text-white text-lg">{ch.name}</h4>
                                        <span className="w-2 h-2 rounded-full bg-emerald-400 shadow-[0_0_8px_rgba(52,211,153,0.5)] animate-pulse" />
                                    </div>
                                    <div className="space-y-4">
                                        <div>
                                            <div className="flex justify-between text-[10px] font-bold uppercase tracking-widest text-white/50 mb-1">
                                                <span>Session Volume</span>
                                                <span className="text-mymolt-primary">{volume} msgs</span>
                                            </div>
                                            <div className="h-1.5 w-full bg-white/5 rounded-full overflow-hidden">
                                                <div className="h-full bg-mymolt-primary transition-all duration-1000" style={{ width: `${Math.min(100, (volume / 100) * 100)}%` }} />
                                            </div>
                                        </div>
                                        <div>
                                            <div className="flex justify-between text-[10px] font-bold uppercase tracking-widest text-white/50 mb-1">
                                                <span>API Rate Limit</span>
                                                <span className={rateLimit > 80 ? 'text-red-400' : 'text-emerald-400'}>{rateLimit}%</span>
                                            </div>
                                            <div className="h-1.5 w-full bg-white/5 rounded-full overflow-hidden">
                                                <div className={`h-full transition-all duration-1000 ${rateLimit > 80 ? 'bg-red-400' : 'bg-emerald-400'}`} style={{ width: `${rateLimit}%` }} />
                                            </div>
                                        </div>
                                    </div>
                                </div>
                            );
                        })}
                    </div>
                    <p className="text-xs text-white/30 tracking-wide mt-6 pt-4 border-t border-white/5">
                        These channels expose your MyMolt Agent to the external world. Traffic is monitored for anomalous patterns automatically.
                    </p>
                </div>
            )}

            {categories.map(cat => (
                <div key={cat} className="space-y-6 mb-16">
                    <h3 className="text-[10px] font-black text-white/30 uppercase tracking-[0.3em] pl-1 font-mono flex items-center gap-4">
                        {cat}
                        <div className="h-px bg-white/5 flex-grow" />
                    </h3>
                    <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-8">
                        {integrations.filter(i => i.category === cat).map(int => (
                            <div
                                key={int.name}
                                onClick={() => int.status !== 'coming_soon' && setSelectedIntegration(int)}
                                className={`group bg-mymolt-glass border border-mymolt-glassBorder backdrop-blur-3xl rounded-[2rem] p-8 hover:border-mymolt-yellow/40 transition-all cursor-pointer relative overflow-hidden shadow-2xl ${int.status === 'coming_soon' ? 'opacity-40 grayscale cursor-not-allowed' : 'hover:scale-[1.02] active:scale-[0.98]'}`}
                            >
                                <div className="flex justify-between items-start mb-6">
                                    <h4 className="font-extrabold text-xl text-white group-hover:text-mymolt-yellow transition-colors tracking-tight">{int.name}</h4>
                                    {int.status === 'active' && <CheckCircle2 className="w-6 h-6 text-green-400" />}
                                    {int.status === 'available' && <Circle className="w-6 h-6 text-white/10 group-hover:text-mymolt-yellow/40" />}
                                </div>
                                <p className="text-sm text-white/40 line-clamp-2 mb-8 h-10 leading-relaxed font-semibold tracking-tight">{int.description}</p>

                                <div className="flex items-center justify-between mt-auto pt-6 border-t border-white/5">
                                    <span className={`text-[9px] px-4 py-1.5 rounded-full font-black uppercase tracking-[0.15em] flex items-center gap-2.5 ${int.status === 'active' ? 'bg-green-500/10 text-green-400 border border-green-500/20' :
                                        int.status === 'available' ? 'bg-mymolt-primary/10 text-mymolt-primary border border-mymolt-primary/20' :
                                            'bg-white/5 text-white/20 border border-white/10'
                                        }`}>
                                        <div className={`w-1.5 h-1.5 rounded-full ${int.status === 'active' ? 'bg-green-400 animate-pulse' : int.status === 'available' ? 'bg-mymolt-primary' : 'bg-white/20'}`} />
                                        {int.status === 'active' ? 'Operational' : int.status === 'available' ? 'Configure' : 'Locked'}
                                    </span>
                                    {int.status === 'available' && (
                                        <span className="text-[10px] font-black uppercase tracking-widest text-white/20 group-hover:text-mymolt-yellow transition-colors">Setup &rarr;</span>
                                    )}
                                </div>

                                {/* Background highlight */}
                                <div className="absolute -right-4 -bottom-4 w-24 h-24 bg-mymolt-yellow/5 rounded-full blur-3xl group-hover:bg-mymolt-yellow/10 transition-colors" />
                            </div>
                        ))}
                    </div>
                </div>
            ))}

            <IntegrationConfigModal
                integration={selectedIntegration}
                onClose={() => setSelectedIntegration(null)}
                onSuccess={fetchIntegrations}
            />
        </div>
    );
}
