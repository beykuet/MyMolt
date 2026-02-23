import React, { useState } from 'react';
import { ArrowLeft, Users, Shield, Fingerprint, Cpu, Plug2, Activity } from 'lucide-react';
import { FamilyManager } from '../components/admin/FamilyManager';
import { SecurityDashboard } from '../components/admin/SecurityDashboard';
import { IdentityManager } from '../components/admin/IdentityManager';
import { ProviderConfig } from '../components/admin/ProviderConfig';
import { McpManager } from '../components/admin/McpManager';
import { SystemOverview } from '../components/admin/SystemOverview';
import { motion, AnimatePresence } from 'framer-motion';

type AdminSection = 'family' | 'security' | 'identity' | 'providers' | 'mcp' | 'system';

interface AdminPanelProps {
    onBack: () => void;
}

const sections: { id: AdminSection; icon: React.ElementType; label: string }[] = [
    { id: 'family', icon: Users, label: 'Family' },
    { id: 'security', icon: Shield, label: 'Security' },
    { id: 'identity', icon: Fingerprint, label: 'Identity' },
    { id: 'providers', icon: Cpu, label: 'Providers' },
    { id: 'mcp', icon: Plug2, label: 'MCP Servers' },
    { id: 'system', icon: Activity, label: 'System' },
];

export const AdminPanel: React.FC<AdminPanelProps> = ({ onBack }) => {
    const [active, setActive] = useState<AdminSection>('family');

    return (
        <div className="flex h-screen bg-[#1a1f2e] text-white overflow-hidden">
            {/* Admin Sidebar */}
            <aside className="w-64 bg-[#141822] border-r border-white/[0.06] flex flex-col">
                <div className="p-5 border-b border-white/[0.06]">
                    <button
                        onClick={onBack}
                        className="flex items-center gap-2 text-white/40 hover:text-white transition-colors text-sm font-medium"
                    >
                        <ArrowLeft size={16} />
                        Back to Chat
                    </button>
                    <h1 className="text-lg font-bold mt-3 tracking-tight">Admin Panel</h1>
                </div>
                <nav className="flex-1 p-3 space-y-1">
                    {sections.map((s) => (
                        <button
                            key={s.id}
                            onClick={() => setActive(s.id)}
                            className={`w-full flex items-center gap-3 px-4 py-3 rounded-xl text-sm font-medium transition-all ${active === s.id
                                    ? 'bg-mymolt-yellow/10 text-mymolt-yellow'
                                    : 'text-white/40 hover:text-white/70 hover:bg-white/[0.03]'
                                }`}
                        >
                            <s.icon size={18} />
                            {s.label}
                            {active === s.id && (
                                <motion.div
                                    layoutId="adminActive"
                                    className="ml-auto w-1.5 h-1.5 rounded-full bg-mymolt-yellow"
                                />
                            )}
                        </button>
                    ))}
                </nav>
                <div className="p-4 border-t border-white/[0.06] text-[10px] text-white/20 font-medium uppercase tracking-widest">
                    Root Access Only
                </div>
            </aside>

            {/* Content Area */}
            <main className="flex-1 overflow-y-auto">
                <AnimatePresence mode="wait">
                    <motion.div
                        key={active}
                        initial={{ opacity: 0, y: 8 }}
                        animate={{ opacity: 1, y: 0 }}
                        exit={{ opacity: 0, y: -8 }}
                        transition={{ duration: 0.2 }}
                        className="p-8 max-w-5xl mx-auto"
                    >
                        {active === 'family' && <FamilyManager />}
                        {active === 'security' && <SecurityDashboard />}
                        {active === 'identity' && <IdentityManager />}
                        {active === 'providers' && <ProviderConfig />}
                        {active === 'mcp' && <McpManager />}
                        {active === 'system' && <SystemOverview />}
                    </motion.div>
                </AnimatePresence>
            </main>
        </div>
    );
};
