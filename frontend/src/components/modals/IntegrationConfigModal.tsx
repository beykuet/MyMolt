import { useState } from 'react';
import { apiClient } from '../../api/client';
import { X, Key, Shield, AlertCircle, CheckCircle2 } from 'lucide-react';

interface Props {
    integration: {
        name: string;
        description: string;
        category: string;
    } | null;
    onClose: () => void;
    onSuccess: () => void;
}

export default function IntegrationConfigModal({ integration, onClose, onSuccess }: Props) {
    const [apiKey, setApiKey] = useState('');
    const [loading, setLoading] = useState(false);
    const [error, setError] = useState<string | null>(null);
    const [success, setSuccess] = useState(false);

    if (!integration) return null;

    const handleSave = async () => {
        if (!apiKey.trim()) {
            setError('API Key/Token is required');
            return;
        }

        setLoading(true);
        setError(null);

        try {
            await apiClient.post(`/api/integrations/${integration.name}/configure`, {
                api_key: apiKey
            });
            setSuccess(true);
            setTimeout(() => {
                onSuccess();
                onClose();
            }, 1500);
        } catch (e: any) {
            setError(e.message || 'Failed to save configuration');
        } finally {
            setLoading(false);
        }
    };

    return (
        <div className="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/60 backdrop-blur-sm">
            <div className="bg-[#0f172a] border border-mymolt-glassBorder rounded-3xl w-full max-w-md shadow-2xl overflow-hidden animate-in fade-in zoom-in duration-200">
                {/* Header */}
                <div className="p-6 border-b border-white/5 flex justify-between items-center bg-gradient-to-r from-mymolt-blue to-mymolt-blue-subtle text-white">
                    <div>
                        <h3 className="text-xl font-bold flex items-center gap-2">
                            <Key className="w-5 h-5 text-mymolt-yellow" />
                            Configure {integration.name}
                        </h3>
                        <p className="text-xs text-white/50 mt-1 font-medium tracking-tight">{integration.category}</p>
                    </div>
                    <button
                        onClick={onClose}
                        className="p-2 hover:bg-white/10 rounded-full transition-colors"
                    >
                        <X className="w-5 h-5" />
                    </button>
                </div>

                {/* Body */}
                <div className="p-8 space-y-6">
                    {!success ? (
                        <>
                            <div className="bg-mymolt-yellow/5 border border-mymolt-yellow/10 rounded-2xl p-5 flex gap-4">
                                <Shield className="w-6 h-6 text-mymolt-yellow shrink-0 mt-0.5" />
                                <div className="space-y-1">
                                    <h4 className="text-sm font-bold text-white tracking-tight">Sovereign Encryption</h4>
                                    <p className="text-xs text-white/50 font-medium leading-relaxed">
                                        Your API keys are encrypted at rest using your local soul key. MyMolt never transmits these keys outside your private infrastructure.
                                    </p>
                                </div>
                            </div>

                            <div className="space-y-2">
                                <label className="text-[10px] font-black text-white/30 uppercase tracking-[0.2em] pl-1 font-mono">
                                    {integration.category.includes('Chat') ? 'Bot Token / App Secret' : 'API Key'}
                                </label>
                                <div className="relative group">
                                    <input
                                        type="password"
                                        value={apiKey}
                                        onChange={(e) => setApiKey(e.target.value)}
                                        placeholder={`Enter ${integration.name} credentials...`}
                                        className="w-full bg-black/20 border border-mymolt-glassBorder rounded-2xl py-4 px-5 text-sm text-white focus:outline-none focus:border-mymolt-yellow/50 focus:ring-1 focus:ring-mymolt-yellow/20 transition-all font-mono"
                                    />
                                    <div className="absolute right-4 top-1/2 -translate-y-1/2 opacity-0 group-hover:opacity-100 transition-opacity">
                                        <Key className="w-4 h-4 text-white/20" />
                                    </div>
                                </div>
                            </div>

                            {error && (
                                <div className="flex items-center gap-2 text-red-400 bg-red-400/10 border border-red-400/20 px-4 py-3 rounded-2xl animate-in slide-in-from-top-1 duration-200">
                                    <AlertCircle className="w-4 h-4 shrink-0" />
                                    <span className="text-xs font-bold tracking-tight">{error}</span>
                                </div>
                            )}

                            <button
                                onClick={handleSave}
                                disabled={loading}
                                className={`w-full py-4 rounded-2xl font-black uppercase tracking-[0.15em] text-xs transition-all flex items-center justify-center gap-2 shadow-lg shadow-mymolt-yellow/5 ${loading
                                        ? 'bg-white/5 text-white/20 cursor-not-allowed'
                                        : 'bg-mymolt-yellow text-black hover:scale-[1.02] active:scale-95 hover:shadow-mymolt-yellow/20'
                                    }`}
                            >
                                {loading ? 'Securing...' : 'Verify & Save Integration'}
                            </button>
                        </>
                    ) : (
                        <div className="py-12 flex flex-col items-center text-center space-y-4 animate-in fade-in transition-all">
                            <div className="w-20 h-20 bg-green-500/10 rounded-full flex items-center justify-center border border-green-500/20 mb-2">
                                <CheckCircle2 className="w-10 h-10 text-green-400" />
                            </div>
                            <h4 className="text-xl font-bold text-white tracking-tight">Configuration Secured</h4>
                            <p className="text-sm text-white/50 max-w-[240px] font-medium leading-relaxed">
                                {integration.name} is now operational within your MyMolt workspace.
                            </p>
                        </div>
                    )}
                </div>

                {/* Footer */}
                {!success && (
                    <div className="px-8 py-6 bg-black/20 border-t border-white/5 flex items-center justify-center">
                        <p className="text-[10px] text-white/20 font-bold uppercase tracking-widest flex items-center gap-2">
                            <Shield className="w-3 h-3" />
                            End-to-End Encrypted Configuration
                        </p>
                    </div>
                )}
            </div>
        </div>
    );
}
