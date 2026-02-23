import { useEffect, useState } from 'react';
import { apiClient } from '../../api/client';
import { Save, RefreshCw } from 'lucide-react';

export const ModelTuningWidget = () => {
    const [config, setConfig] = useState({ system_prompt: '', temperature: 0.7 });
    const [loading, setLoading] = useState(true);
    const [saving, setSaving] = useState(false);

    useEffect(() => {
        loadConfig();
    }, []);

    const loadConfig = async () => {
        try {
            const data = await apiClient.get<any>('/api/config/model');
            setConfig(data);
        } catch (e) {
            console.error(e);
        } finally {
            setLoading(false);
        }
    };

    const handleSave = async () => {
        setSaving(true);
        try {
            await apiClient.post('/api/config/model', config);
            alert('Core tuning updated (runtime only).');
        } catch (e: any) {
            alert('Failed to update: ' + e.message);
        } finally {
            setSaving(false);
        }
    };

    if (loading) return <div className="text-white/50 p-4">Loading core config...</div>;

    return (
        <div className="h-full flex flex-col gap-4 text-white p-1">
            <h3 className="text-xl font-bold flex items-center gap-2">
                Core Tuning
                <button onClick={loadConfig} className="p-1 hover:bg-white/10 rounded-full transition-colors ml-auto mr-0">
                    <RefreshCw size={14} className="text-white/30" />
                </button>
            </h3>

            <div className="flex-1 flex flex-col gap-2 min-h-0">
                <label className="text-[10px] font-black text-mymolt-yellow uppercase tracking-widest">System Prompt (Persona)</label>
                <textarea
                    value={config.system_prompt}
                    onChange={e => setConfig({ ...config, system_prompt: e.target.value })}
                    className="flex-1 bg-black/20 border border-mymolt-glassBorder rounded-2xl p-5 text-sm font-mono leading-relaxed focus:outline-none focus:border-mymolt-yellow/50 resize-none text-white/80 placeholder:opacity-20 transition-all shadow-inner"
                    placeholder="Enter the sovereign persona instructions..."
                />
            </div>

            <div className="flex items-center gap-8 bg-black/20 p-5 rounded-2xl border border-mymolt-glassBorder mt-auto">
                <div className="flex-1">
                    <div className="flex justify-between mb-3">
                        <label className="text-[10px] font-black text-mymolt-yellow uppercase tracking-widest">Temperature (Creativity)</label>
                        <span className="text-xs font-black text-mymolt-yellow font-mono">{config.temperature.toFixed(2)}</span>
                    </div>
                    <input
                        type="range"
                        min="0" max="2" step="0.05"
                        value={config.temperature}
                        onChange={e => setConfig({ ...config, temperature: parseFloat(e.target.value) })}
                        className="w-full h-1.5 bg-white/10 rounded-full appearance-none cursor-pointer accent-mymolt-yellow transition-all hover:bg-white/15"
                    />
                </div>

                <button
                    onClick={handleSave}
                    disabled={saving}
                    className="bg-mymolt-yellow hover:bg-yellow-400 text-black px-8 py-3 rounded-xl font-black text-xs uppercase tracking-wider flex items-center gap-3 transition-all disabled:opacity-50 shadow-lg shadow-yellow-500/10 active:scale-95"
                >
                    <Save size={16} />
                    {saving ? 'REFORMING...' : 'APPLY REFORM'}
                </button>
            </div>
        </div>
    );
};
