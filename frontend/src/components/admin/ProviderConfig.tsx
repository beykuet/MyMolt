import React, { useState, useEffect } from 'react';
import { Cpu, CheckCircle2, XCircle, Plus, Trash2, Zap, Route } from 'lucide-react';
import { apiClient } from '../../api/client';

interface Provider {
    name: string;
    status: 'connected' | 'disconnected' | 'unconfigured';
    model?: string;
    endpoint?: string;
    api_key_set: boolean;
}

interface ModelRoute {
    hint: string;
    provider: string;
    model: string;
}

export const ProviderConfig: React.FC = () => {
    const [providers, setProviders] = useState<Provider[]>([
        { name: 'OpenRouter', status: 'unconfigured', api_key_set: false },
        { name: 'Ollama', status: 'unconfigured', endpoint: 'http://localhost:11434', api_key_set: false },
        { name: 'Anthropic', status: 'unconfigured', api_key_set: false },
        { name: 'OpenAI', status: 'unconfigured', api_key_set: false },
        { name: 'Gemini', status: 'unconfigured', api_key_set: false },
    ]);
    const [routes, setRoutes] = useState<ModelRoute[]>([]);
    const [activeProvider, setActiveProvider] = useState('');
    const [activeModel, setActiveModel] = useState('');

    useEffect(() => {
        apiClient.fetch('/config/models')
            .then(r => r.ok ? r.json() : null)
            .then(data => {
                if (data) {
                    if (data.active_provider) setActiveProvider(data.active_provider);
                    if (data.active_model) setActiveModel(data.active_model);
                    if (data.providers) setProviders(data.providers);
                    if (data.routes) setRoutes(data.routes);
                }
            })
            .catch(() => { });
    }, []);

    const testProvider = async (_name: string) => {
        // Placeholder — would call /api/providers/{name}/test
    };

    const statusIcon = (status: string) => {
        switch (status) {
            case 'connected': return <CheckCircle2 size={14} className="text-emerald-400" />;
            case 'disconnected': return <XCircle size={14} className="text-red-400" />;
            default: return <div className="w-3.5 h-3.5 rounded-full border-2 border-white/20" />;
        }
    };

    return (
        <div>
            <h2 className="text-2xl font-bold tracking-tight mb-2">Providers & Models</h2>
            <p className="text-white/40 text-sm mb-8">LLM providers, model routes, and STT configuration</p>

            {/* Active provider */}
            <div className="bg-[#1e2435] rounded-2xl border border-mymolt-yellow/20 p-6 mb-6">
                <div className="flex items-center gap-2 mb-3">
                    <Zap size={16} className="text-mymolt-yellow" />
                    <h3 className="text-sm font-bold uppercase tracking-wider text-white/60">Active Provider</h3>
                </div>
                <p className="text-lg font-bold">{activeProvider || 'Not configured'}</p>
                <p className="text-sm text-white/40 mt-1">{activeModel || '—'}</p>
            </div>

            {/* Provider list */}
            <div className="bg-[#1e2435] rounded-2xl border border-white/[0.06] p-6 mb-6">
                <div className="flex items-center gap-2 mb-5">
                    <Cpu size={16} className="text-mymolt-yellow" />
                    <h3 className="text-sm font-bold uppercase tracking-wider text-white/60">Providers</h3>
                </div>
                <div className="space-y-2">
                    {providers.map((p) => (
                        <div
                            key={p.name}
                            className="flex items-center justify-between p-4 rounded-xl bg-black/20 border border-white/[0.04] hover:border-white/[0.08] transition"
                        >
                            <div className="flex items-center gap-3">
                                {statusIcon(p.status)}
                                <div>
                                    <p className="text-sm font-medium">{p.name}</p>
                                    <p className="text-[11px] text-white/30">
                                        {p.endpoint || (p.api_key_set ? 'API Key: ****' : 'Not configured')}
                                    </p>
                                </div>
                            </div>
                            <div className="flex items-center gap-2">
                                {p.status !== 'unconfigured' && (
                                    <button
                                        onClick={() => testProvider(p.name)}
                                        className="text-xs font-medium px-3 py-1.5 rounded-lg bg-white/[0.04] hover:bg-white/[0.08] border border-white/[0.06] transition"
                                    >
                                        Test
                                    </button>
                                )}
                                <button className="text-xs font-medium text-mymolt-yellow hover:text-yellow-300 transition">
                                    {p.status === 'unconfigured' ? 'Setup' : 'Edit'}
                                </button>
                            </div>
                        </div>
                    ))}
                </div>
            </div>

            {/* Model Routes */}
            <div className="bg-[#1e2435] rounded-2xl border border-white/[0.06] p-6">
                <div className="flex items-center justify-between mb-5">
                    <div className="flex items-center gap-2">
                        <Route size={16} className="text-mymolt-yellow" />
                        <h3 className="text-sm font-bold uppercase tracking-wider text-white/60">Model Routes</h3>
                    </div>
                    <button className="flex items-center gap-1 text-xs font-medium text-mymolt-yellow hover:text-yellow-300 transition">
                        <Plus size={12} /> Add Route
                    </button>
                </div>
                {routes.length > 0 ? (
                    <div className="space-y-2">
                        {routes.map((route, i) => (
                            <div
                                key={i}
                                className="flex items-center justify-between p-3 rounded-lg bg-black/20 border border-white/[0.04]"
                            >
                                <div className="flex items-center gap-3 text-sm font-mono">
                                    <span className="text-mymolt-yellow">hint:{route.hint}</span>
                                    <span className="text-white/20">→</span>
                                    <span className="text-white/60">{route.provider}/{route.model}</span>
                                </div>
                                <button className="text-white/20 hover:text-red-400 transition">
                                    <Trash2 size={14} />
                                </button>
                            </div>
                        ))}
                    </div>
                ) : (
                    <p className="text-center text-white/20 py-6 text-sm">
                        No model routes configured. Use <span className="text-white/40 font-mono">hint:name</span> in chat to route.
                    </p>
                )}
            </div>
        </div>
    );
};
