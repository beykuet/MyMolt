import React, { useState, useEffect } from 'react';
import { motion } from 'framer-motion';
import { Shield, Activity, Lock, AlertTriangle, CheckCircle, ExternalLink, Clock } from 'lucide-react';
import { apiClient } from '../api/client';

interface SecurityConfig {
    enabled_skills: string[];
    disabled_skills: string[];
    confirmation_required: Record<string, string>;
}

const Security: React.FC = () => {
    const [config, setConfig] = useState<SecurityConfig | null>(null);
    const [loading, setLoading] = useState(true);
    const [error, setError] = useState<string | null>(null);

    useEffect(() => {
        fetchConfig();
    }, []);

    const fetchConfig = async () => {
        try {
            const res = await apiClient.fetch('/security/policy');
            if (!res.ok) throw new Error('Failed to fetch security policy');
            setConfig(await res.json());
        } catch (err: any) {
            setError(err.message);
        } finally {
            setLoading(false);
        }
    };

    const updateConfig = async (newConfig: SecurityConfig) => {
        try {
            const updated = await apiClient.post<SecurityConfig>('/security/policy', newConfig);
            setConfig(updated);
        } catch (err: any) {
            setError(err.message);
        }
    };

    const toggleSkill = (skill: string, enabled: boolean) => {
        if (!config) return;
        const newEnabled = enabled
            ? [...config.enabled_skills, skill]
            : config.enabled_skills.filter(s => s !== skill);

        const newDisabled = enabled
            ? config.disabled_skills.filter(s => s !== skill)
            : [...config.disabled_skills, skill];

        updateConfig({
            ...config,
            enabled_skills: newEnabled,
            disabled_skills: newDisabled
        });
    };

    const setConfirmation = (action: string, level: string) => {
        if (!config) return;
        updateConfig({
            ...config,
            confirmation_required: { ...config.confirmation_required, [action]: level }
        });
    };

    if (loading) return <div className="p-8 text-white/50">Loading security policy...</div>;
    if (error) return <div className="p-8 text-red-400">Error: {error}</div>;
    if (!config) return null;

    const allSkills = Array.from(new Set([...config.enabled_skills, ...config.disabled_skills]));

    return (
        <div className="h-full overflow-y-auto pr-2 pb-20">
            <div className="max-w-4xl mx-auto space-y-8">

                {/* Header */}
                <div className="flex items-center gap-4 mb-8">
                    <div className="w-12 h-12 rounded-2xl bg-mymolt-yellow flex items-center justify-center shadow-lg shadow-yellow-500/20">
                        <Shield className="text-black" size={24} />
                    </div>
                    <div>
                        <h1 className="text-3xl font-black text-white tracking-tight">Security Center</h1>
                        <p className="text-white/50 font-medium">Manage agent permissions and autonomous behavior</p>
                    </div>
                </div>

                {/* Regular Audits Disclaimer */}
                <motion.div
                    initial={{ opacity: 0, y: 10 }}
                    animate={{ opacity: 1, y: 0 }}
                    className="p-6 rounded-3xl bg-red-500/10 border border-red-500/20 relative overflow-hidden"
                >
                    <div className="absolute top-0 right-0 p-32 bg-red-500/5 blur-[100px] rounded-full" />
                    <div className="relative z-10">
                        <div className="flex items-center gap-3 mb-4">
                            <AlertTriangle className="text-red-400" size={24} />
                            <h2 className="text-xl font-bold text-red-400">Mandatory Security Audits</h2>
                        </div>
                        <p className="text-white/80 mb-6 leading-relaxed">
                            To maintain a secure environment, you must perform regular audits of your agent's activity.
                            Failure to do so significantly increases the risk of successful prompt injection or agent hijacking.
                        </p>

                        <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                            <div className="p-4 rounded-2xl bg-black/20 border border-white/5">
                                <div className="flex items-center gap-2 mb-2">
                                    <Clock className="text-mymolt-yellow" size={16} />
                                    <h3 className="font-bold text-white text-sm">Weekly Checklist</h3>
                                </div>
                                <ul className="space-y-2 text-sm text-white/60">
                                    <li className="flex items-start gap-2">
                                        <div className="w-1 h-1 rounded-full bg-white/30 mt-2" />
                                        Review all actions in Audit Log
                                    </li>
                                    <li className="flex items-start gap-2">
                                        <div className="w-1 h-1 rounded-full bg-white/30 mt-2" />
                                        Check for unexpected file access
                                    </li>
                                    <li className="flex items-start gap-2">
                                        <div className="w-1 h-1 rounded-full bg-white/30 mt-2" />
                                        Review new integrations/skills
                                    </li>
                                </ul>
                            </div>

                            <div className="p-4 rounded-2xl bg-black/20 border border-white/5">
                                <div className="flex items-center gap-2 mb-2">
                                    <Activity className="text-blue-400" size={16} />
                                    <h3 className="font-bold text-white text-sm">Skill Review</h3>
                                </div>
                                <ul className="space-y-2 text-sm text-white/60">
                                    <li className="flex items-start gap-2">
                                        <div className="w-1 h-1 rounded-full bg-white/30 mt-2" />
                                        Verify skill source repositories
                                    </li>
                                    <li className="flex items-start gap-2">
                                        <div className="w-1 h-1 rounded-full bg-white/30 mt-2" />
                                        Check for upstream updates
                                    </li>
                                    <li className="flex items-start gap-2">
                                        <div className="w-1 h-1 rounded-full bg-white/30 mt-2" />
                                        Disable unused skills
                                    </li>
                                </ul>
                            </div>
                        </div>
                    </div>
                </motion.div>

                {/* Skill Permissions */}
                <div className="p-8 rounded-3xl bg-mymolt-glass border border-mymolt-glassBorder shadow-2xl backdrop-blur-xl">
                    <h2 className="text-xl font-bold text-white mb-6 flex items-center gap-3">
                        <Lock className="text-blue-400" size={20} />
                        Skill Permissions
                    </h2>

                    <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                        {allSkills.map(skill => {
                            const isEnabled = config.enabled_skills.includes(skill);
                            return (
                                <div key={skill} className={`
                                    p-4 rounded-2xl border transition-all flex items-center justify-between
                                    ${isEnabled
                                        ? 'bg-blue-500/10 border-blue-500/20'
                                        : 'bg-white/5 border-white/5 opacity-60 hover:opacity-100'}
                                `}>
                                    <div>
                                        <div className="font-bold text-white">{skill}</div>
                                        <div className="text-xs text-white/40 mt-1 flex items-center gap-1">
                                            Last reviewed: <span className="text-white/60">Never</span>
                                            <a href="#" className="ml-2 hover:text-blue-400 transition-colors">
                                                <ExternalLink size={10} />
                                            </a>
                                        </div>
                                    </div>
                                    <button
                                        onClick={() => toggleSkill(skill, !isEnabled)}
                                        className={`
                                            w-12 h-6 rounded-full relative transition-colors duration-300
                                            ${isEnabled ? 'bg-blue-500' : 'bg-white/10'}
                                        `}
                                    >
                                        <div className={`
                                            absolute top-1 left-1 w-4 h-4 rounded-full bg-white transition-transform duration-300 shadow-md
                                            ${isEnabled ? 'translate-x-6' : 'translate-x-0'}
                                        `} />
                                    </button>
                                </div>
                            );
                        })}
                    </div>
                </div>

                {/* Confirmation Policy */}
                <div className="p-8 rounded-3xl bg-mymolt-glass border border-mymolt-glassBorder shadow-2xl backdrop-blur-xl">
                    <h2 className="text-xl font-bold text-white mb-6 flex items-center gap-3">
                        <CheckCircle className="text-green-400" size={20} />
                        Confirmation Policy
                    </h2>

                    <div className="space-y-4">
                        {Object.entries(config.confirmation_required).map(([action, level]) => (
                            <div key={action} className="flex items-center justify-between p-4 rounded-2xl bg-white/5 border border-white/5">
                                <div>
                                    <div className="font-bold text-white font-mono text-sm">{action}</div>
                                    <div className="text-xs text-white/40 mt-1">Requiring user approval</div>
                                </div>
                                <select
                                    value={level}
                                    onChange={(e) => setConfirmation(action, e.target.value)}
                                    className="bg-black/40 text-white text-sm border border-white/10 rounded-xl px-3 py-2 outline-none focus:border-blue-500/50"
                                >
                                    <option value="always">Always Ask</option>
                                    <option value="risky_only">Risky Only</option>
                                    <option value="never">Never (Dangerous)</option>
                                </select>
                            </div>
                        ))}
                    </div>
                </div>

            </div>
        </div>
    );
};

export default Security;
