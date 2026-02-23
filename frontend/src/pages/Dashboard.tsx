import React, { useState, useEffect } from 'react';
import {
    Activity,
    Shield,
    LogOut,
    Menu,
    X,
    MessageSquare,
    Eye,
    Fingerprint,
    Network,
    HardDrive,
    Lock,
    Brain,
    ChevronDown,
    Box,
    Plug,
    Settings,
    Globe
} from 'lucide-react';
import Security from './Security';
import Skills from './Skills';
import Integrations from './Integrations';
import { useAuth } from '../context/AuthContext';
import { apiClient } from '../api/client';
import type { SystemStatus, UserRole } from '../types';
import { DiaryWidget } from '../components/widgets/DiaryWidget';
import { VPNWidget } from '../components/widgets/VPNWidget';
import { VaultWidget } from '../components/widgets/VaultWidget';
import { ChatWidget } from '../components/widgets/ChatWidget';
import { SigilWidget } from '../components/widgets/SigilWidget';
import { IdentityWidget } from '../components/widgets/IdentityWidget';
import { ModelTuningWidget } from '../components/widgets/ModelTuningWidget';
import { SchedulerWidget } from '../components/widgets/SchedulerWidget';
import { AdBlockWidget } from '../components/widgets/AdBlockWidget';
import { FilesWidget } from '../components/widgets/FilesWidget';
import { BrowserWidget } from '../components/widgets/BrowserWidget';
import { VoiceButton } from '../components/ui/VoiceButton';
import { useAudio } from '../hooks/useAudio';
import { useSocket } from '../hooks/useSocket';
import { motion, AnimatePresence } from 'framer-motion';
import { AdminPanel } from './AdminPanel';

type TabId = 'chat' | 'browser' | 'sigil' | 'adblock' | 'soul' | 'vpn' | 'vault' | 'files' | 'diary' | 'system' | 'skills' | 'integrations' | 'security';

interface DashboardProps {
    role: UserRole;
}

export const Dashboard: React.FC<DashboardProps> = ({ role }) => {
    const { user, logout } = useAuth();
    const { isRecording, volume, startRecording, stopRecording, playAudio } = useAudio();
    const { sendMessage, messages } = useSocket();
    const [status, setStatus] = useState<SystemStatus | null>(null);
    const [isSidebarOpen, setSidebarOpen] = useState(role === 'Root');
    const [activeTab, setActiveTab] = useState<TabId>('chat');
    const [model, setModel] = useState('Llama-3 (Local)');
    const [isModelMenuOpen, setModelMenuOpen] = useState(false);
    const [showAdmin, setShowAdmin] = useState(false);
    const [pendingApprovals, setPendingApprovals] = useState<any[]>([]);
    const [isApprovalsModalOpen, setApprovalsModalOpen] = useState(false);

    const isRoot = role === 'Root';
    const isAdult = role === 'Root' || role === 'Adult';

    // Handle incoming audio
    useEffect(() => {
        const lastMsg = messages[messages.length - 1];
        if (lastMsg?.type === 'audio' && lastMsg.payload.data) {
            playAudio(lastMsg.payload.data);
        }
    }, [messages, playAudio]);

    useEffect(() => {
        const fetchStatus = async () => {
            try {
                const res = await apiClient.fetch('/system/status');
                if (res.ok) setStatus(await res.json());
            } catch (err) {
                console.error('Failed to fetch system status', err);
            }
        };

        fetchStatus();
        const interval = setInterval(fetchStatus, 30000);
        return () => clearInterval(interval);
    }, []);

    useEffect(() => {
        if (!isAdult) return;
        const fetchPending = async () => {
            try {
                const res = await apiClient.fetch('/security/confirm/pending');
                if (res.ok) {
                    const data = await res.json();
                    setPendingApprovals(data.pending || []);
                }
            } catch (err) { }
        };
        fetchPending();
        const interval = setInterval(fetchPending, 3000);
        return () => clearInterval(interval);
    }, [isAdult]);

    const handleApproval = async (id: string, approved: boolean) => {
        try {
            await apiClient.fetch('/security/confirm', {
                method: 'POST',
                body: JSON.stringify({ id, approved })
            });
            setPendingApprovals(prev => prev.filter(a => a.id !== id));
            if (pendingApprovals.length <= 1) setApprovalsModalOpen(false);
        } catch (err) { }
    };

    const sidebarItems = [
        { id: 'chat' as const, icon: MessageSquare, label: 'Sovereign Chat', roles: ['Root', 'Adult', 'Child', 'Senior'] },
        { id: 'browser' as const, icon: Globe, label: 'Sovereign Browser', roles: ['Root', 'Adult', 'Child', 'Senior'] },
        { id: 'soul' as const, icon: Fingerprint, label: 'Soul Identity', roles: ['Root', 'Adult'] },
        { id: 'sigil' as const, icon: Eye, label: 'Sigil Transparency', roles: ['Root'] },
        { id: 'adblock' as const, icon: Shield, label: 'DNS Shield', roles: ['Root', 'Adult'] },
        { id: 'vpn' as const, icon: Network, label: 'VPN Connect', roles: ['Root', 'Adult'] },
        { id: 'vault' as const, icon: Lock, label: 'Secure Vault', roles: ['Root'] },
        { id: 'files' as const, icon: HardDrive, label: 'Sovereign Files', roles: ['Root', 'Adult'] },
        { id: 'diary' as const, icon: Brain, label: 'Cognitive Diary', roles: ['Root', 'Adult', 'Senior'] },
        { id: 'skills' as const, icon: Box, label: 'SkillForge', roles: ['Root'] },
        { id: 'integrations' as const, icon: Plug, label: 'Integration Store', roles: ['Root'] },
        { id: 'system' as const, icon: Activity, label: 'System Health', roles: ['Root', 'Adult'] },
        { id: 'security' as const, icon: Shield, label: 'Security', roles: ['Root'] },
    ];

    const filteredSidebar = sidebarItems.filter(item =>
        item.roles.includes(role)
    );

    // Show Admin Panel full-screen
    if (showAdmin && isRoot) {
        return <AdminPanel onBack={() => setShowAdmin(false)} />;
    }

    return (
        <div className="flex h-screen bg-mymolt-bg bg-gradient text-slate-100 overflow-hidden font-sans">
            <motion.aside
                initial={false}
                animate={{ width: isSidebarOpen ? 280 : 80 }}
                className="relative bg-mymolt-glass backdrop-blur-3xl border-r border-mymolt-glassBorder flex flex-col z-50 shadow-2xl"
            >
                <div className="p-6 flex items-center justify-between">
                    <AnimatePresence>
                        {isSidebarOpen && (
                            <motion.div
                                initial={{ opacity: 0, x: -20 }}
                                animate={{ opacity: 1, x: 0 }}
                                exit={{ opacity: 0, x: -20 }}
                                className="flex items-center gap-3"
                            >
                                <div className="w-10 h-10 rounded-2xl bg-mymolt-primary flex items-center justify-center shadow-lg shadow-cyan-500/30">
                                    <svg className="w-6 h-6" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 48 48" fill="none">
                                        <path d="M 12,36 L 16,14 C 18,10 24,14 24,20 C 24,14 30,10 32,14 L 36,36"
                                            stroke="#ffffff" strokeWidth="6" strokeLinecap="round" strokeLinejoin="round" fill="none" />
                                    </svg>
                                </div>
                                <span className="text-xl font-black tracking-tighter bg-clip-text text-transparent bg-gradient-to-r from-white to-white/50">MyMolt</span>
                            </motion.div>
                        )}
                    </AnimatePresence>
                    <button
                        onClick={() => setSidebarOpen(!isSidebarOpen)}
                        className="p-2 hover:bg-white/5 rounded-xl transition-colors text-white/50"
                    >
                        {isSidebarOpen ? <X size={20} /> : <Menu size={20} />}
                    </button>
                </div>

                <nav className="flex-1 p-4 space-y-2 mt-4">
                    {filteredSidebar.map((item) => (
                        <button
                            key={item.id}
                            onClick={() => setActiveTab(item.id)}
                            className={`
                                w-full flex items-center gap-4 p-4 rounded-2xl transition-all relative group
                                ${activeTab === item.id
                                    ? 'bg-mymolt-primary/10 text-mymolt-primary border border-mymolt-primary/20'
                                    : 'text-white/40 hover:bg-white/5 hover:text-white'}
                            `}
                        >
                            <item.icon size={20} className={activeTab === item.id ? 'text-mymolt-primary' : 'group-hover:text-white transition-colors'} />
                            {isSidebarOpen && (
                                <motion.span
                                    initial={{ opacity: 0 }}
                                    animate={{ opacity: 1 }}
                                    className="font-bold text-sm tracking-tight"
                                >
                                    {item.label}
                                </motion.span>
                            )}
                            {activeTab === item.id && (
                                <motion.div
                                    layoutId="activeTab"
                                    className="absolute left-0 w-1 h-8 bg-mymolt-primary rounded-r-full"
                                />
                            )}
                        </button>
                    ))}
                </nav>

                <div className="p-4 border-t border-white/5 space-y-1">
                    {isRoot && (
                        <button
                            onClick={() => setShowAdmin(true)}
                            className="w-full flex items-center gap-4 p-4 rounded-2xl hover:bg-mymolt-primary/10 text-white/40 hover:text-mymolt-primary transition-all group"
                        >
                            <Settings size={20} />
                            {isSidebarOpen && <span className="font-bold text-sm">Admin Panel</span>}
                        </button>
                    )}
                    <button
                        onClick={logout}
                        className="w-full flex items-center gap-4 p-4 rounded-2xl hover:bg-red-500/10 text-white/40 hover:text-red-400 transition-all group"
                    >
                        <LogOut size={20} />
                        {isSidebarOpen && <span className="font-bold text-sm">Terminate Session</span>}
                    </button>
                </div>
            </motion.aside>

            <main className="flex-1 flex flex-col relative overflow-hidden">
                <header className="h-20 border-b border-mymolt-glassBorder flex items-center justify-between px-8 bg-mymolt-glass backdrop-blur-xl z-40">
                    <div className="flex items-center gap-4">
                        <div className="flex items-center gap-2 px-4 py-2 bg-white/5 rounded-full border border-white/10">
                            <span className={`w-2 h-2 rounded-full ${status ? 'bg-green-500 shadow-[0_0_8px_rgba(34,197,94,0.5)]' : 'bg-red-500'}`} />
                            <span className="text-[10px] font-black uppercase tracking-widest text-white/50">Core {status?.version || 'v1.0'}</span>
                        </div>
                        {isAdult && pendingApprovals.length > 0 && (
                            <button
                                onClick={() => setApprovalsModalOpen(true)}
                                className="flex items-center gap-2 px-4 py-2 bg-mymolt-warning/20 text-mymolt-warning rounded-full border border-mymolt-warning/50 animate-pulse transition-all hover:bg-mymolt-warning/30"
                            >
                                <Lock size={16} />
                                <span className="text-xs font-bold">{pendingApprovals.length} Pending Actions</span>
                            </button>
                        )}
                    </div>

                    <div className="flex items-center gap-6">
                        <div className="relative">
                            <button
                                onClick={() => setModelMenuOpen(!isModelMenuOpen)}
                                className="flex items-center gap-3 px-4 py-2 bg-white/5 border border-white/10 rounded-xl hover:bg-white/10 transition-all"
                            >
                                <Brain size={16} className="text-mymolt-primary" />
                                <span className="text-xs font-bold text-white/80">{model}</span>
                                <ChevronDown size={14} className={`text-white/30 transition-transform ${isModelMenuOpen ? 'rotate-180' : ''}`} />
                            </button>
                            <AnimatePresence>
                                {isModelMenuOpen && (
                                    <motion.div
                                        initial={{ opacity: 0, y: 10 }}
                                        animate={{ opacity: 1, y: 0 }}
                                        exit={{ opacity: 0, y: 10 }}
                                        className="absolute right-0 mt-2 w-56 bg-slate-900 border border-white/10 rounded-2xl shadow-2xl p-2 z-50 overflow-hidden"
                                    >
                                        {[
                                            { id: 'llama3', name: 'Llama-3 (Local)', provider: 'MyMolt Runtime v1' },
                                            { id: 'gpt4', name: 'GPT-4o (Cloud Bridge)', provider: 'OpenAI Gateway' },
                                            { id: 'claude', name: 'Claude 3.5 (Cloud Bridge)', provider: 'Anthropic Gateway' }
                                        ].map(m => (
                                            <button
                                                key={m.id}
                                                onClick={() => { setModel(m.name); setModelMenuOpen(false); }}
                                                className="w-full text-left p-4 rounded-xl hover:bg-white/5 transition-all group"
                                            >
                                                <div className="text-sm font-bold text-white group-hover:text-mymolt-primary">{m.name}</div>
                                                <div className="text-[10px] text-white/30 font-medium uppercase tracking-wider">{m.provider}</div>
                                            </button>
                                        ))}
                                    </motion.div>
                                )}
                            </AnimatePresence>
                        </div>

                        <div className="flex items-center gap-4 pl-6 border-l border-white/10">
                            <div className="text-right hidden sm:block">
                                <p className="text-sm font-black tracking-tight text-white">{user?.username || 'Sovereign'}</p>
                                <p className="text-[10px] font-bold text-mymolt-primary uppercase tracking-widest">{role}</p>
                            </div>
                            <div className="w-10 h-10 rounded-full bg-slate-800 border-2 border-white/10 flex items-center justify-center text-white/50">
                                <Fingerprint size={20} />
                            </div>
                        </div>
                    </div>
                </header>

                <div className={`flex-1 overflow-hidden ${activeTab === 'chat' || activeTab === 'browser' ? 'p-0' : 'p-8 pb-32'}`}>
                    <AnimatePresence mode="wait">
                        <motion.div
                            key={activeTab}
                            initial={{ opacity: 0, scale: 0.98, filter: 'blur(10px)' }}
                            animate={{ opacity: 1, scale: 1, filter: 'blur(0px)' }}
                            exit={{ opacity: 0, scale: 0.98, filter: 'blur(10px)' }}
                            transition={{ duration: 0.4, ease: [0.23, 1, 0.32, 1] }}
                            className="h-full w-full"
                        >
                            {activeTab === 'chat' && <ChatWidget />}
                            {activeTab === 'browser' && <BrowserWidget />}
                            {activeTab === 'sigil' && isRoot && <SigilWidget />}
                            {activeTab === 'adblock' && isAdult && <AdBlockWidget />}
                            {activeTab === 'soul' && isAdult && <IdentityWidget />}
                            {activeTab === 'vpn' && isAdult && <VPNWidget />}
                            {activeTab === 'vault' && isRoot && <VaultWidget />}
                            {activeTab === 'files' && isAdult && <FilesWidget />}

                            {activeTab === 'diary' && (role === 'Root' || role === 'Adult' || role === 'Senior') && <DiaryWidget />}
                            {activeTab === 'skills' && isRoot && <Skills />}
                            {activeTab === 'integrations' && isRoot && <Integrations />}
                            {activeTab === 'security' && isRoot && <Security />}
                            {activeTab === 'system' && (
                                <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
                                    <div className="bg-mymolt-glass backdrop-blur-xl p-8 rounded-[2rem] border border-mymolt-glassBorder shadow-2xl">
                                        <label className="text-[10px] font-black text-mymolt-primary uppercase tracking-widest">Core Status</label>
                                        <h3 className="text-3xl font-black mt-2 text-white">Active</h3>
                                        <div className="mt-4 h-1.5 w-full bg-white/5 rounded-full overflow-hidden">
                                            <div className="h-full w-[98%] bg-mymolt-primary shadow-[0_0_15px_rgba(8,145,178,0.5)]" />
                                        </div>
                                    </div>
                                    <div className="bg-mymolt-glass backdrop-blur-xl p-8 rounded-[2rem] border border-mymolt-glassBorder shadow-2xl col-span-1 md:col-span-1 min-h-[450px]">
                                        <ModelTuningWidget />
                                    </div>
                                    <SchedulerWidget />
                                </div>
                            )}
                        </motion.div>
                    </AnimatePresence>
                </div>

                {/* Voice Toggle — only on chat tab */}
                {activeTab === 'chat' && (
                    <div className="fixed bottom-12 left-1/2 -translate-x-1/2 z-50 pointer-events-none">
                        <div className="pointer-events-auto">
                            <VoiceButton
                                isListening={isRecording}
                                volume={volume}
                                onClick={async () => {
                                    if (isRecording) {
                                        const base64 = await stopRecording();
                                        if (base64) {
                                            sendMessage({
                                                type: 'audio',
                                                payload: {
                                                    data: base64,
                                                    format: 'webm'
                                                }
                                            });
                                        }
                                    } else {
                                        startRecording();
                                    }
                                }}
                            />
                        </div>
                    </div>
                )}

                <AnimatePresence>
                    {isApprovalsModalOpen && (
                        <div className="fixed inset-0 bg-black/80 backdrop-blur-sm z-50 flex items-center justify-center p-4">
                            <motion.div
                                initial={{ opacity: 0, scale: 0.95 }}
                                animate={{ opacity: 1, scale: 1 }}
                                exit={{ opacity: 0, scale: 0.95 }}
                                className="glass-panel max-w-2xl w-full p-8"
                            >
                                <div className="flex justify-between items-center mb-6">
                                    <h3 className="text-2xl font-black text-white flex items-center gap-3">
                                        <Lock className="text-mymolt-warning" />
                                        Pending Human-in-the-Loop Approvals
                                    </h3>
                                    <button onClick={() => setApprovalsModalOpen(false)} className="text-white/50 hover:text-white p-2">
                                        <X size={24} />
                                    </button>
                                </div>

                                <div className="space-y-4 max-h-[60vh] overflow-y-auto pr-2">
                                    {pendingApprovals.length === 0 ? (
                                        <p className="text-white/50">No pending approvals.</p>
                                    ) : pendingApprovals.map((req) => (
                                        <div key={req.id} className="bg-black/40 border border-mymolt-warning/30 rounded-2xl p-6 relative overflow-hidden">
                                            <div className="absolute top-0 left-0 w-1 h-full bg-mymolt-warning" />
                                            <div className="flex justify-between items-start mb-4 pl-2">
                                                <div>
                                                    <span className="text-[10px] font-black text-mymolt-warning uppercase tracking-widest">{req.tool_name}</span>
                                                    <h4 className="text-lg font-bold text-white mt-1">High-Risk Action Requested</h4>
                                                </div>
                                                <span className="text-xs font-mono text-white/40">{new Date(req.requested_at).toLocaleTimeString().replace(/Invalid Date/, 'Just now')}</span>
                                            </div>
                                            <div className="bg-black/50 p-4 rounded-xl border border-white/5 font-mono text-sm text-mymolt-primary mb-6 break-all ml-2">
                                                {req.description}
                                            </div>
                                            <div className="flex gap-4 pl-2">
                                                <button
                                                    onClick={() => handleApproval(req.id, true)}
                                                    className="flex-1 bg-mymolt-success text-white py-3 rounded-xl font-bold hover:brightness-110 transition-all shadow-[0_0_20px_rgba(22,163,74,0.3)] hover:shadow-[0_0_30px_rgba(22,163,74,0.5)] cursor-pointer"
                                                >
                                                    Authorize
                                                </button>
                                                <button
                                                    onClick={() => handleApproval(req.id, false)}
                                                    className="flex-1 bg-red-500/20 text-red-500 border border-red-500/50 py-3 rounded-xl font-bold hover:bg-red-500/30 transition-all cursor-pointer"
                                                >
                                                    Deny
                                                </button>
                                            </div>
                                        </div>
                                    ))}
                                </div>
                            </motion.div>
                        </div>
                    )}
                </AnimatePresence>
            </main>
        </div>
    );
};
