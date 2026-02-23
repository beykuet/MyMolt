import { useEffect, useState } from 'react';
import { apiClient } from '../api/client';
import type { Skill } from '../types';
import { Trash2, Download, Box } from 'lucide-react';

export default function Skills() {
    const [skills, setSkills] = useState<Skill[]>([]);
    const [url, setUrl] = useState('');
    const [loading, setLoading] = useState(true);
    const [installing, setInstalling] = useState(false);

    useEffect(() => {
        loadSkills();
    }, []);

    const loadSkills = async () => {
        try {
            const data = await apiClient.get<Skill[]>('/api/skills');
            setSkills(data);
        } catch (err) {
            console.error(err);
        } finally {
            setLoading(false);
        }
    };

    const handleInstall = async () => {
        if (!url) return;
        setInstalling(true);
        try {
            await apiClient.post('/api/skills', { url });
            setUrl('');
            await loadSkills();
        } catch (err: any) {
            alert('Install failed: ' + (err.message || err));
        } finally {
            setInstalling(false);
        }
    };

    const handleRemove = async (name: string) => {
        if (!confirm(`Delete skill ${name}?`)) return;
        try {
            await apiClient.delete(`/api/skills/${name}`);
            await loadSkills();
        } catch (err: any) {
            alert('Remove failed: ' + (err.message || err));
        }
    };

    if (loading) return <div className="p-6 text-white">Loading skills...</div>;

    return (
        <div className="space-y-6 text-white p-6 h-full overflow-y-auto">
            <div className="flex justify-between items-center mb-6">
                <h2 className="text-2xl font-bold flex items-center gap-2">
                    <Box className="w-6 h-6 text-blue-400" />
                    SkillForge
                </h2>
                <div className="flex gap-2">
                    <input
                        type="text"
                        value={url}
                        onChange={e => setUrl(e.target.value)}
                        placeholder="https://github.com/user/repo"
                        className="bg-black/20 border border-mymolt-glassBorder rounded-xl px-4 py-2 text-sm focus:outline-none focus:border-mymolt-yellow w-72 text-white placeholder-white/20 transition-all font-mono"
                    />
                    <button
                        onClick={handleInstall}
                        disabled={installing || !url}
                        className="bg-mymolt-yellow hover:bg-yellow-400 text-black disabled:opacity-50 px-6 py-2 rounded-xl flex items-center gap-2 text-sm transition-all font-bold shadow-lg shadow-yellow-500/10"
                    >
                        {installing ? 'Installing...' : <><Download size={16} /> Install Skill</>}
                    </button>
                </div>
            </div>

            <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6 pb-20">
                {skills.map(skill => (
                    <div key={skill.name} className="bg-mymolt-glass border border-mymolt-glassBorder backdrop-blur-xl rounded-2xl p-6 hover:border-mymolt-yellow/40 transition-all group shadow-xl">
                        <div className="flex justify-between items-start mb-3">
                            <div>
                                <h3 className="font-semibold text-lg text-white group-hover:text-blue-200 transition-colors">{skill.name}</h3>
                                <span className="text-xs text-gray-500 font-mono bg-gray-900 px-1.5 py-0.5 rounded">v{skill.version}</span>
                            </div>
                            <button
                                onClick={() => handleRemove(skill.name)}
                                className="text-gray-500 hover:text-red-400 p-1.5 rounded-lg hover:bg-red-900/20 transition-colors"
                                title="Remove Skill"
                            >
                                <Trash2 size={18} />
                            </button>
                        </div>
                        <p className="text-gray-400 text-sm mb-4 line-clamp-3 h-10">{skill.description}</p>

                        <div className="pt-4 border-t border-white/10">
                            <p className="text-[10px] text-white/30 uppercase font-black tracking-widest mb-3">Tools Provided</p>
                            <div className="flex flex-wrap gap-2">
                                {skill.tools.length > 0 ? skill.tools.map(t => (
                                    <span key={t} className="bg-black/30 text-mymolt-yellow px-2.5 py-1 rounded-lg text-xs border border-mymolt-yellow/20 flex items-center gap-1.5 font-bold">
                                        <Box size={10} /> {t}
                                    </span>
                                )) : <span className="text-white/20 text-xs italic">No tools declared</span>}
                            </div>
                        </div>
                    </div>
                ))}
                {skills.length === 0 && (
                    <div className="col-span-full flex flex-col items-center justify-center p-16 text-gray-500 border-2 border-dashed border-gray-800 rounded-xl bg-gray-900/20">
                        <Box size={48} className="mb-4 opacity-50" />
                        <h3 className="text-lg font-medium text-gray-400">No skills installed</h3>
                        <p className="text-sm">Extend your agent's capabilities by installing community skills.</p>
                    </div>
                )}
            </div>
        </div>
    );
}
