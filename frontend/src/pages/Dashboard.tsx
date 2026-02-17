import { Sidebar } from '../components/layout/Sidebar';
import { Header } from '../components/layout/Header';
import { StatusCard } from '../components/ui/StatusCard';
import { VoiceButton } from '../components/ui/VoiceButton';
import { Mascot } from '../components/ui/Mascot';
import { Activity, Shield, Cpu, Lock, Plus } from 'lucide-react';
import { useState, useEffect } from 'react';
import type { IdentityStatus, SystemStatus, IdentityProvider } from '../types';

import { useSocket } from '../hooks/useSocket';
import { useAudio } from '../hooks/useAudio';

interface DashboardProps {
    onLogout: () => void;
}

export function Dashboard({ onLogout }: DashboardProps) {
    const { isRecording, volume, startRecording, stopRecording, playAudio } = useAudio();
    const { sendMessage, messages } = useSocket();

    // Handle incoming audio
    useEffect(() => {
        const lastMsg = messages[messages.length - 1];
        if (lastMsg?.type === 'audio') {
            playAudio(lastMsg.payload.data);
        }
    }, [messages, playAudio]);

    const [identities, setIdentities] = useState<IdentityStatus[]>([]);
    const [providers, setProviders] = useState<IdentityProvider[]>([]);
    const [systemStatus, setSystemStatus] = useState<SystemStatus | null>(null);
    const [loadingIds, setLoadingIds] = useState(false);

    const fetchSystemStatus = async () => {
        try {
            const token = localStorage.getItem('mymolt_token');
            const headers: HeadersInit = {};
            if (token) headers['Authorization'] = `Bearer ${token}`;

            const res = await fetch('/api/system/status', { headers });
            if (res.ok) {
                setSystemStatus(await res.json());
            }
        } catch (e) {
            console.error(e);
        }
    };

    const fetchIdentities = async () => {
        try {
            const token = localStorage.getItem('mymolt_token');
            const headers: HeadersInit = {};
            if (token) headers['Authorization'] = `Bearer ${token}`;

            const res = await fetch('/api/identity', { headers });
            if (res.ok) {
                const data = await res.json();
                setIdentities(data);
            }
        } catch (e) {
            console.error(e);
        }
    };

    const fetchProviders = async () => {
        try {
            const token = localStorage.getItem('mymolt_token');
            const headers: HeadersInit = {};
            if (token) headers['Authorization'] = `Bearer ${token}`;

            const res = await fetch('/api/auth/providers', { headers });
            if (res.ok) {
                const data = await res.json();
                setProviders(data);
            }
        } catch (e) {
            console.error(e);
        }
    };

    useEffect(() => {
        fetchSystemStatus();
        fetchIdentities();
        fetchProviders();
        const interval = setInterval(fetchSystemStatus, 5000); // Poll status
        return () => clearInterval(interval);
    }, []);

    const togglePairing = async () => {
        if (!systemStatus) return;
        const newState = !systemStatus.pairing_enabled;
        try {
            const token = localStorage.getItem('mymolt_token');
            await fetch('/api/config/pairing', {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json',
                    ...(token ? { 'Authorization': `Bearer ${token}` } : {})
                },
                body: JSON.stringify({ enabled: newState })
            });
            fetchSystemStatus();
        } catch (e) {
            console.error(e);
        }
    };

    const toggleVoiceEcho = async () => {
        if (!systemStatus) return;
        const newState = !systemStatus.voice_echo_enabled;
        try {
            const token = localStorage.getItem('mymolt_token');
            await fetch('/api/config/voice_echo', {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json',
                    ...(token ? { 'Authorization': `Bearer ${token}` } : {})
                },
                body: JSON.stringify({ enabled: newState })
            });
            fetchSystemStatus();
        } catch (e) {
            console.error(e);
        }
    };

    const simulateLink = async (provider: string) => {
        setLoadingIds(true);
        try {
            const token = localStorage.getItem('mymolt_token');
            await fetch('/api/identity/simulate', {
                method: 'POST',
                headers: {
                    'Authorization': `Bearer ${token}`,
                    'Content-Type': 'application/json'
                },
                body: JSON.stringify({ provider, id: `simulated-${Date.now()}` })
            });
            await fetchIdentities();
        } finally {
            setLoadingIds(false);
        }
    };

    return (
        <div className="min-h-screen bg-zinc-50 dark:bg-zinc-950 text-zinc-900 dark:text-zinc-100 font-sans selection:bg-blue-500/30">
            <Sidebar />
            <Header status="online" onLogout={onLogout} />

            <main className="pl-0 md:pl-28 pt-24 px-6 pb-12 max-w-7xl mx-auto">
                {/* Welcome Section */}
                <div className="flex flex-col md:flex-row items-center justify-between mb-12">
                    <div className="text-center md:text-left mb-8 md:mb-0">
                        <h1 className="text-4xl md:text-5xl font-bold bg-clip-text text-transparent bg-gradient-to-r from-blue-600 to-purple-600 dark:from-blue-400 dark:to-purple-400 mb-2">
                            Good Evening, Ben.
                        </h1>
                        <p className="text-zinc-500 dark:text-zinc-400 text-lg">
                            System is optimal. EU Compliance active.
                        </p>
                    </div>
                    <Mascot />
                </div>

                {/* Action Grid */}
                <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6 mb-12">
                    <StatusCard title="System Health" icon={<Activity className="text-green-500" />}>
                        98% Optimal
                        <div className="h-1 w-full bg-zinc-200 dark:bg-zinc-800 rounded-full mt-2 overflow-hidden">
                            <div className="h-full w-[98%] bg-green-500 rounded-full" />
                        </div>
                    </StatusCard>

                    <StatusCard title="Security" icon={<Shield className={systemStatus?.pairing_enabled ? "text-green-500" : "text-yellow-500"} />}>
                        <div className="flex items-center justify-between mb-2">
                            <span className="text-sm font-medium">{systemStatus?.pairing_enabled ? "Pairing: Active" : "Pairing: Off"}</span>
                            <button
                                onClick={togglePairing}
                                className={`text-[10px] px-2 py-1 rounded font-bold uppercase tracking-wider ${systemStatus?.pairing_enabled
                                    ? "bg-red-100 text-red-600 hover:bg-red-200 dark:bg-red-900/30 dark:text-red-400"
                                    : "bg-green-100 text-green-600 hover:bg-green-200 dark:bg-green-900/30 dark:text-green-400"
                                    }`}
                            >
                                {systemStatus?.pairing_enabled ? "Disable" : "Enable"}
                            </button>
                        </div>
                        <div className="flex items-center justify-between">
                            <span className="text-sm font-medium">{systemStatus?.voice_echo_enabled ? "Echo: Loopback" : "Echo: Off"}</span>
                            <button
                                onClick={toggleVoiceEcho}
                                className={`text-[10px] px-2 py-1 rounded font-bold uppercase tracking-wider ${systemStatus?.voice_echo_enabled
                                    ? "bg-orange-100 text-orange-600 hover:bg-orange-200 dark:bg-orange-900/30 dark:text-orange-400"
                                    : "bg-zinc-100 text-zinc-600 hover:bg-zinc-200 dark:bg-zinc-800 dark:text-zinc-400"
                                    }`}
                            >
                                {systemStatus?.voice_echo_enabled ? "Disable" : "Enable"}
                            </button>
                        </div>
                        <div className="mt-4 pt-4 border-t border-zinc-100 dark:border-zinc-800">
                            <button
                                onClick={() => sendMessage({ type: 'control', payload: { event: 'voice_test' } })}
                                className="w-full py-2 rounded bg-blue-500 hover:bg-blue-600 text-white text-xs font-bold uppercase tracking-wider transition-colors"
                            >
                                Test Bot Voice
                            </button>
                        </div>
                        <div className="flex gap-1 mt-2 items-center">
                            <div className={`h-2 w-2 rounded-full ${systemStatus?.pairing_enabled ? "bg-green-500 animate-pulse" : "bg-yellow-500"}`} />
                            <div className="text-xs text-zinc-400">
                                {systemStatus?.pairing_enabled ? "Gateway protected" : "Open access mode"}
                            </div>
                        </div>
                    </StatusCard>

                    <StatusCard title="Load" icon={<Cpu className="text-purple-500" />}>
                        12% CPU / 0.4GB RAM
                    </StatusCard>

                    <StatusCard title="Identity" icon={<Lock className="text-yellow-500" />}>
                        <div className="flex flex-wrap items-center gap-2">
                            {identities.map(id => (
                                <span key={id.provider} className={`px-2 py-0.5 rounded text-xs font-bold border capitalize ${id.trust_level >= 3
                                    ? 'bg-blue-100 dark:bg-blue-900/30 text-blue-700 dark:text-blue-300 border-blue-200 dark:border-blue-800'
                                    : 'bg-zinc-100 dark:bg-zinc-800 text-zinc-600 dark:text-zinc-400 border-zinc-200 dark:border-zinc-700'
                                    }`}>
                                    {id.provider}
                                </span>
                            ))}

                            <input
                                type="file"
                                id="eidas-upload"
                                className="hidden"
                                accept=".json,.pem,.cer"
                                onChange={async (e) => {
                                    const file = e.target.files?.[0];
                                    if (!file) return;

                                    setLoadingIds(true);
                                    try {
                                        const formData = new FormData();
                                        formData.append('certificate', file);

                                        const token = localStorage.getItem('mymolt_token');
                                        await fetch('/api/identity/eidas/verify', {
                                            method: 'POST',
                                            headers: {
                                                'Authorization': `Bearer ${token}`
                                            },
                                            body: formData
                                        });
                                        await fetchIdentities();
                                    } finally {
                                        setLoadingIds(false);
                                    }
                                }}
                            />

                            {!identities.some(i => i.provider === 'eIDAS') && (
                                <button
                                    onClick={() => document.getElementById('eidas-upload')?.click()}
                                    disabled={loadingIds}
                                    className="px-2 py-0.5 rounded bg-transparent hover:bg-zinc-100 dark:hover:bg-zinc-800 text-zinc-400 hover:text-blue-500 border border-dashed border-zinc-300 dark:border-zinc-700 text-xs flex items-center gap-1 transition-colors"
                                >
                                    <Shield size={12} /> Verify eID
                                </button>
                            )}

                            {!identities.some(i => i.provider === 'google') && (
                                <button
                                    onClick={() => simulateLink('google')}
                                    disabled={loadingIds}
                                    className="px-2 py-0.5 rounded bg-transparent hover:bg-zinc-100 dark:hover:bg-zinc-800 text-zinc-400 hover:text-blue-500 border border-dashed border-zinc-300 dark:border-zinc-700 text-xs flex items-center gap-1 transition-colors"
                                >
                                    <Plus size={12} /> Google
                                </button>
                            )}

                            {providers.map(p => (
                                !identities.some(i => i.provider === p.name) && (
                                    <button
                                        key={p.id}
                                        onClick={() => window.location.href = `/api/auth/login/${p.id}`}
                                        className="px-2 py-0.5 rounded bg-transparent hover:bg-zinc-100 dark:hover:bg-zinc-800 text-zinc-400 hover:text-blue-500 border border-dashed border-zinc-300 dark:border-zinc-700 text-xs flex items-center gap-1 transition-colors"
                                    >
                                        {p.icon_url ? <img src={p.icon_url} className="w-3 h-3" /> : <Plus size={12} />}
                                        {p.name}
                                    </button>
                                )
                            ))}
                        </div>
                    </StatusCard>
                </div>

                {/* Voice Interaction Area */}
                <div className="fixed bottom-8 left-0 right-0 flex justify-center z-50 pointer-events-none">
                    <div className="pointer-events-auto">
                        <VoiceButton
                            isListening={isRecording}
                            volume={volume}
                            onClick={() => {
                                if (isRecording) {
                                    stopRecording();
                                } else {
                                    startRecording((base64) => {
                                        sendMessage({
                                            type: 'audio',
                                            payload: {
                                                data: base64,
                                                format: 'webm' // MediaRecorder default
                                            }
                                        });
                                    });
                                }
                            }}
                        />
                    </div>
                </div>
            </main >
        </div >
    );
}
