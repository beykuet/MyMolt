import React, { useState, useEffect, useRef } from 'react';
import {
    Fingerprint, UserCheck, ShieldCheck, RefreshCw, Key, ChevronRight,
    Gavel, Lock, Upload, Globe, Wallet, ExternalLink, CheckCircle2, AlertCircle, Loader2
} from 'lucide-react';
import { motion, AnimatePresence } from 'framer-motion';
import { apiClient } from '../../api/client';
import type { IdentityStatus, IdentityProvider } from '../../types';

type ModalView = null | 'google-oidc' | 'eidas-upload' | 'ssi-wallet' | 'oidc-provider';

export const IdentityWidget: React.FC = () => {
    const [identities, setIdentities] = useState<IdentityStatus[]>([]);
    const [providers, setProviders] = useState<IdentityProvider[]>([]);
    const [loading, setLoading] = useState(true);
    const [modalView, setModalView] = useState<ModalView>(null);
    const [modalStatus, setModalStatus] = useState<'idle' | 'loading' | 'success' | 'error'>('idle');
    const [modalMessage, setModalMessage] = useState('');

    const [vpInput, setVpInput] = useState('');
    const fileInputRef = useRef<HTMLInputElement>(null);

    const fetchData = async () => {
        setLoading(true);
        try {
            const [idData, providerData] = await Promise.all([
                apiClient.get<IdentityStatus[]>('/api/identity'),
                apiClient.get<IdentityProvider[]>('/api/auth/providers')
            ]);
            setIdentities(idData);
            setProviders(providerData);
        } catch (err) {
            console.error(err);
        } finally {
            setLoading(false);
        }
    };

    useEffect(() => {
        fetchData();
    }, []);

    // Reset modal status when view changes
    useEffect(() => {
        setModalStatus('idle');
        setModalMessage('');
        setVpInput('');
    }, [modalView]);

    // ── eIDAS Certificate Upload Handler ─────────────────────────────
    const handleEidasUpload = async (e: React.ChangeEvent<HTMLInputElement>) => {
        const file = e.target.files?.[0];
        if (!file) return;

        setModalStatus('loading');
        setModalMessage('Verifying eIDAS certificate…');

        try {
            const formData = new FormData();
            formData.append('certificate', file);

            const res = await apiClient.fetch('/api/identity/eidas/verify', {
                method: 'POST',
                body: formData,
            });
            const data = await res.json();

            if (data.success) {
                setModalStatus('success');
                setModalMessage(`eIDAS identity verified: ${data.id} (Level: ${data.level})`);
                fetchData(); // Refresh bindings
            } else {
                setModalStatus('error');
                setModalMessage(data.error || 'Verification failed');
            }
        } catch (err) {
            setModalStatus('error');
            setModalMessage('Network error during verification');
        }
    };

    // ── SSI Wallet VP Verification Handler ────────────────────────────
    const handleSSIVerify = async () => {
        if (!vpInput.trim()) return;

        setModalStatus('loading');
        setModalMessage('Verifying Verifiable Presentation…');

        try {
            const data = await apiClient.post<{ success: boolean; result?: any; error?: string }>(
                '/api/identity/verify-vp',
                { vp: vpInput }
            );

            if (data.success) {
                setModalStatus('success');
                const holder = data.result?.holder_did || 'Unknown DID';
                setModalMessage(`VP verified! Holder: ${holder}`);
                fetchData();
            } else {
                setModalStatus('error');
                setModalMessage(data.error || 'VP verification failed');
            }
        } catch (err) {
            setModalStatus('error');
            setModalMessage('Network error during VP verification');
        }
    };

    // ── Google OIDC Login Handler ─────────────────────────────────────
    const handleGoogleOIDC = () => {
        // Simulate linking for demo (in production, redirect to /api/auth/login/google)
        setModalStatus('loading');
        setModalMessage('Connecting to Google OIDC…');

        apiClient.post<{ success: boolean; message?: string; error?: string }>(
            '/api/identity/simulate',
            { provider: 'Google OIDC', id: `google-${Date.now().toString(36)}` }
        ).then(data => {
            if (data.success) {
                setModalStatus('success');
                setModalMessage('Google identity linked successfully');
                fetchData();
            } else {
                setModalStatus('error');
                setModalMessage(data.error || 'Failed to link Google identity');
            }
        }).catch(() => {
            setModalStatus('error');
            setModalMessage('Connection failed');
        });
    };

    // ── Generic OIDC Login Handler ────────────────────────────────────
    const handleOIDCLogin = (provider: IdentityProvider) => {
        // Redirect to OIDC login endpoint
        window.location.href = `/api/auth/login/${provider.id}`;
    };

    const getProviderIcon = (providerName: string) => {
        const name = providerName.toLowerCase();
        if (name.includes('google')) return <Globe size={20} className="text-blue-400" />;
        if (name.includes('eidas') || name.includes('austria') || name.includes('spid')) return <ShieldCheck size={20} className="text-mymolt-yellow" />;
        if (name.includes('ssi') || name.includes('wallet') || name.includes('did')) return <Wallet size={20} className="text-purple-400" />;
        if (name.includes('local')) return <Key size={20} className="text-green-400" />;
        return <UserCheck size={20} className="text-mymolt-text-muted" />;
    };

    const trustLevelColor = (level: number) => {
        if (level >= 3) return 'bg-green-500/10 border-green-500/20 text-green-400';
        if (level >= 2) return 'bg-mymolt-yellow-dim border-mymolt-yellow/20 text-mymolt-yellow';
        return 'bg-blue-500/10 border-blue-500/20 text-blue-400';
    };

    return (
        <div className="flex flex-col h-full bg-mymolt-glass backdrop-blur-xl border border-mymolt-glassBorder rounded-3xl overflow-hidden shadow-2xl transition-all">
            {/* Header */}
            <div className="p-8 border-b border-mymolt-glassBorder bg-gradient-to-r from-mymolt-blue/60 to-mymolt-blue-subtle/40">
                <div className="flex items-center justify-between mb-2">
                    <div className="flex items-center gap-3">
                        <div className="w-12 h-12 rounded-2xl bg-mymolt-blue flex items-center justify-center shadow-lg shadow-mymolt-blue/50">
                            <Fingerprint className="text-white" size={24} />
                        </div>
                        <div>
                            <h2 className="text-2xl font-bold text-white tracking-tight">Soul Identity</h2>
                            <p className="text-xs text-mymolt-text-muted/60 uppercase tracking-[0.2em] font-bold">eIDAS · OIDC · SSI</p>
                        </div>
                    </div>
                    <button
                        onClick={fetchData}
                        className="p-3 hover:bg-mymolt-glass rounded-2xl transition-colors text-mymolt-text-muted/50 hover:text-white"
                    >
                        <RefreshCw size={20} className={loading ? 'animate-spin' : ''} />
                    </button>
                </div>
            </div>

            <div className="flex-1 p-6 space-y-6 overflow-y-auto">
                {/* Trust Level Overview */}
                <div className="p-4 bg-mymolt-glass border border-mymolt-glassBorder rounded-2xl flex items-center justify-between">
                    <div className="flex items-center gap-3">
                        <Gavel className="text-mymolt-yellow" size={20} />
                        <span className="text-sm font-medium text-white">Consensual Trust Level</span>
                    </div>
                    <div className="flex gap-1">
                        {[1, 2, 3].map((lvl) => {
                            const maxLevel = Math.max(...identities.map(i => i.trust_level), 0);
                            return (
                                <div
                                    key={lvl}
                                    className={`h-2 w-8 rounded-full transition-colors ${lvl <= maxLevel ? 'bg-mymolt-blue' : 'bg-white/10'}`}
                                />
                            );
                        })}
                    </div>
                </div>

                {/* Current Bindings */}
                <div className="space-y-4">
                    <label className="text-[10px] uppercase font-bold text-mymolt-text-muted/40 tracking-widest block ml-2">Verified Connections</label>
                    <AnimatePresence>
                        {identities.map((id) => (
                            <motion.div
                                initial={{ opacity: 0, x: -10 }}
                                animate={{ opacity: 1, x: 0 }}
                                key={`${id.provider}-${id.id}`}
                                className="group bg-mymolt-glass border border-mymolt-glassBorder p-5 rounded-2xl hover:bg-white/10 transition-all flex items-center justify-between"
                            >
                                <div className="flex items-center gap-4">
                                    <div className="w-10 h-10 rounded-xl bg-mymolt-glass flex items-center justify-center border border-mymolt-glassBorder group-hover:scale-110 transition-transform">
                                        {getProviderIcon(id.provider)}
                                    </div>
                                    <div>
                                        <div className="flex items-center gap-2">
                                            <div className="text-sm font-bold text-white mb-0.5">{id.provider}</div>
                                            {id.provider.toLowerCase().includes('eidas') && (
                                                <div className="relative flex items-center justify-center" title="EU Qualified Trust Service Verified">
                                                    <div className="absolute inset-0 bg-yellow-400/30 rounded-full blur-[6px] animate-pulse" />
                                                    <CheckCircle2 size={14} className="text-mymolt-yellow drop-shadow-[0_0_8px_rgba(250,204,21,0.8)] relative z-10" />
                                                </div>
                                            )}
                                        </div>
                                        <div className="text-[10px] font-mono text-mymolt-text-muted/40 truncate max-w-[200px]">{id.id}</div>
                                    </div>
                                </div>
                                <div className="flex items-center gap-3">
                                    <div className={`flex items-center gap-1.5 px-3 py-1 border rounded-full text-[10px] font-bold uppercase ${trustLevelColor(id.trust_level)}`}>
                                        <ShieldCheck size={12} />
                                        Level {id.trust_level}
                                    </div>
                                    <ChevronRight size={16} className="text-white/20 group-hover:text-white transition-colors" />
                                </div>
                            </motion.div>
                        ))}
                    </AnimatePresence>
                </div>

                {/* ── Identity Bridge Cards ─────────────────────────────── */}
                <div className="mt-8">
                    <label className="text-[10px] uppercase font-bold text-mymolt-text-muted/40 tracking-widest block ml-2 mb-4">Link Identity</label>
                    <div className="grid grid-cols-2 lg:grid-cols-4 gap-3">
                        {/* Google OIDC */}
                        <button
                            onClick={() => setModalView('google-oidc')}
                            className="p-5 bg-mymolt-glass border border-mymolt-glassBorder rounded-2xl hover:bg-mymolt-blue/10 hover:border-mymolt-blue/30 transition-all flex flex-col items-center gap-3 group"
                        >
                            <div className="w-12 h-12 rounded-xl bg-white/5 flex items-center justify-center border border-mymolt-glassBorder">
                                <Globe size={22} className="text-blue-400 group-hover:text-white transition-colors" />
                            </div>
                            <div className="text-center">
                                <span className="text-xs font-bold text-white/70 group-hover:text-white block">Google OIDC</span>
                                <span className="text-[9px] text-mymolt-text-muted/30 uppercase tracking-wider">Level 1</span>
                            </div>
                        </button>

                        {/* eIDAS */}
                        <button
                            onClick={() => setModalView('eidas-upload')}
                            className="p-5 bg-mymolt-glass border border-mymolt-glassBorder rounded-2xl hover:bg-mymolt-yellow-dim hover:border-mymolt-yellow/30 transition-all flex flex-col items-center gap-3 group"
                        >
                            <div className="w-12 h-12 rounded-xl bg-white/5 flex items-center justify-center border border-mymolt-glassBorder">
                                <ShieldCheck size={22} className="text-mymolt-yellow group-hover:text-white transition-colors" />
                            </div>
                            <div className="text-center">
                                <span className="text-xs font-bold text-white/70 group-hover:text-white block">eIDAS</span>
                                <span className="text-[9px] text-mymolt-text-muted/30 uppercase tracking-wider">Level 3 (High)</span>
                            </div>
                        </button>

                        {/* SSI Wallet */}
                        <button
                            onClick={() => setModalView('ssi-wallet')}
                            className="p-5 bg-mymolt-glass border border-mymolt-glassBorder rounded-2xl hover:bg-mymolt-yellow/10 hover:border-mymolt-yellow/30 transition-all flex flex-col items-center gap-3 group"
                        >
                            <div className="w-12 h-12 rounded-xl bg-white/5 flex items-center justify-center border border-mymolt-glassBorder">
                                <Wallet size={22} className="text-mymolt-yellow group-hover:text-white transition-colors" />
                            </div>
                            <div className="text-center">
                                <span className="text-xs font-bold text-white/70 group-hover:text-white block">SSI Wallet</span>
                                <span className="text-[9px] text-mymolt-text-muted/30 uppercase tracking-wider">DID / VP</span>
                            </div>
                        </button>

                        {/* Dynamic OIDC providers */}
                        {providers.map((p) => (
                            <button
                                key={p.id}
                                onClick={() => handleOIDCLogin(p)}
                                className="p-5 bg-mymolt-glass border border-mymolt-glassBorder rounded-2xl hover:bg-mymolt-blue/10 hover:border-mymolt-blue/30 transition-all flex flex-col items-center gap-3 group"
                            >
                                <div className="w-12 h-12 rounded-xl bg-white/5 flex items-center justify-center border border-mymolt-glassBorder overflow-hidden">
                                    {p.icon_url
                                        ? <img src={p.icon_url} alt={p.name} className="w-6 h-6" />
                                        : <ExternalLink size={22} className="text-mymolt-text-muted/50 group-hover:text-white transition-colors" />
                                    }
                                </div>
                                <div className="text-center">
                                    <span className="text-xs font-bold text-white/70 group-hover:text-white block">{p.name}</span>
                                    <span className="text-[9px] text-mymolt-text-muted/30 uppercase tracking-wider">Level {p.trust_level}</span>
                                </div>
                            </button>
                        ))}
                    </div>
                </div>
            </div>

            {/* Footer */}
            <div className="p-6 bg-mymolt-glass/50 border-t border-mymolt-glassBorder">
                <div className="flex items-center gap-2 text-xs text-mymolt-text-muted/40">
                    <Lock size={12} />
                    Soul bindings are immutable and eIDAS 2.0 compliant.
                </div>
            </div>

            {/* ── Modal Overlay ──────────────────────────────────────── */}
            <AnimatePresence>
                {modalView && (
                    <motion.div
                        initial={{ opacity: 0 }}
                        animate={{ opacity: 1 }}
                        exit={{ opacity: 0 }}
                        className="fixed inset-0 bg-black/60 backdrop-blur-sm z-[100] flex items-center justify-center p-4"
                        onClick={() => setModalView(null)}
                    >
                        <motion.div
                            initial={{ opacity: 0, scale: 0.95, y: 20 }}
                            animate={{ opacity: 1, scale: 1, y: 0 }}
                            exit={{ opacity: 0, scale: 0.95, y: 20 }}
                            onClick={(e) => e.stopPropagation()}
                            className="w-full max-w-md bg-slate-900 border border-mymolt-glassBorder rounded-3xl shadow-2xl overflow-hidden"
                        >
                            {/* ── Google OIDC Modal ── */}
                            {modalView === 'google-oidc' && (
                                <div className="p-8 space-y-6">
                                    <div className="flex items-center gap-4">
                                        <div className="w-14 h-14 rounded-2xl bg-blue-500/10 border border-blue-500/20 flex items-center justify-center">
                                            <Globe size={28} className="text-blue-400" />
                                        </div>
                                        <div>
                                            <h3 className="text-xl font-bold text-white">Google OIDC</h3>
                                            <p className="text-xs text-mymolt-text-muted/50">OpenID Connect · Trust Level 1</p>
                                        </div>
                                    </div>

                                    <p className="text-sm text-mymolt-text-muted/70 leading-relaxed">
                                        Link your Google account to verify your email identity. This creates a low-trust binding
                                        that can be upgraded with eIDAS or SSI verification.
                                    </p>

                                    <StatusBanner status={modalStatus} message={modalMessage} />

                                    <div className="flex gap-3">
                                        <button
                                            onClick={() => setModalView(null)}
                                            className="flex-1 px-4 py-3 rounded-xl border border-mymolt-glassBorder text-mymolt-text-muted/70 hover:bg-mymolt-glass transition-colors text-sm font-medium"
                                        >
                                            Cancel
                                        </button>
                                        <button
                                            onClick={handleGoogleOIDC}
                                            disabled={modalStatus === 'loading' || modalStatus === 'success'}
                                            className="flex-[2] px-4 py-3 rounded-xl bg-mymolt-blue hover:bg-blue-500 disabled:bg-mymolt-glass disabled:text-mymolt-text-muted/30 text-white font-bold text-sm transition-all flex items-center justify-center gap-2 shadow-lg shadow-mymolt-blue/20"
                                        >
                                            {modalStatus === 'loading' ? <Loader2 size={16} className="animate-spin" /> : <Globe size={16} />}
                                            {modalStatus === 'success' ? 'Linked!' : 'Sign in with Google'}
                                        </button>
                                    </div>
                                </div>
                            )}

                            {/* ── eIDAS Upload Modal ── */}
                            {modalView === 'eidas-upload' && (
                                <div className="p-8 space-y-6">
                                    <div className="flex items-center gap-4">
                                        <div className="w-14 h-14 rounded-2xl bg-mymolt-yellow-dim border border-mymolt-yellow/20 flex items-center justify-center">
                                            <ShieldCheck size={28} className="text-mymolt-yellow" />
                                        </div>
                                        <div>
                                            <h3 className="text-xl font-bold text-white">eIDAS Verification</h3>
                                            <p className="text-xs text-mymolt-text-muted/50">EU Digital Identity · Trust Level 3 (High)</p>
                                        </div>
                                    </div>

                                    <p className="text-sm text-mymolt-text-muted/70 leading-relaxed">
                                        Upload your eIDAS-compliant certificate (PEM/CRT) issued by a qualified trust service provider.
                                        This creates a high-trust government-verified binding.
                                    </p>

                                    <StatusBanner status={modalStatus} message={modalMessage} />

                                    <input
                                        ref={fileInputRef}
                                        type="file"
                                        accept=".pem,.crt,.cer,.der"
                                        onChange={handleEidasUpload}
                                        className="hidden"
                                    />

                                    <div className="flex gap-3">
                                        <button
                                            onClick={() => setModalView(null)}
                                            className="flex-1 px-4 py-3 rounded-xl border border-mymolt-glassBorder text-mymolt-text-muted/70 hover:bg-mymolt-glass transition-colors text-sm font-medium"
                                        >
                                            Cancel
                                        </button>
                                        <button
                                            onClick={() => fileInputRef.current?.click()}
                                            disabled={modalStatus === 'loading' || modalStatus === 'success'}
                                            className="flex-[2] px-4 py-3 rounded-xl bg-mymolt-yellow hover:bg-yellow-400 disabled:bg-mymolt-glass disabled:text-mymolt-text-muted/30 text-black font-bold text-sm transition-all flex items-center justify-center gap-2 shadow-lg shadow-mymolt-yellow/20"
                                        >
                                            {modalStatus === 'loading' ? <Loader2 size={16} className="animate-spin" /> : <Upload size={16} />}
                                            {modalStatus === 'success' ? 'Verified!' : 'Upload Certificate'}
                                        </button>
                                    </div>
                                </div>
                            )}

                            {/* ── SSI Wallet Modal ── */}
                            {modalView === 'ssi-wallet' && (
                                <div className="p-8 space-y-6">
                                    <div className="flex items-center gap-4">
                                        <div className="w-14 h-14 rounded-2xl bg-purple-500/10 border border-purple-500/20 flex items-center justify-center">
                                            <Wallet size={28} className="text-purple-400" />
                                        </div>
                                        <div>
                                            <h3 className="text-xl font-bold text-white">SSI Wallet</h3>
                                            <p className="text-xs text-mymolt-text-muted/50">Verifiable Presentation · DID-based</p>
                                        </div>
                                    </div>

                                    <p className="text-sm text-mymolt-text-muted/70 leading-relaxed">
                                        Paste your Verifiable Presentation (VP) in JSON-LD format from your SSI wallet
                                        (e.g., Sphereon, walt.id, or EU Digital Identity Wallet).
                                    </p>

                                    <textarea
                                        value={vpInput}
                                        onChange={(e) => setVpInput(e.target.value)}
                                        placeholder='{ "@context": [...], "type": "VerifiablePresentation", ... }'
                                        className="w-full h-32 bg-black/20 border border-mymolt-glassBorder rounded-xl p-4 text-sm font-mono text-white placeholder-mymolt-text-muted/20 focus:outline-none focus:ring-2 focus:ring-mymolt-yellow/40 focus:border-mymolt-yellow/50 resize-none transition-all"
                                    />

                                    <StatusBanner status={modalStatus} message={modalMessage} />

                                    <div className="flex gap-3">
                                        <button
                                            onClick={() => setModalView(null)}
                                            className="flex-1 px-4 py-3 rounded-xl border border-mymolt-glassBorder text-mymolt-text-muted/70 hover:bg-mymolt-glass transition-colors text-sm font-medium"
                                        >
                                            Cancel
                                        </button>
                                        <button
                                            onClick={handleSSIVerify}
                                            disabled={!vpInput.trim() || modalStatus === 'loading' || modalStatus === 'success'}
                                            className="flex-[2] px-4 py-3 rounded-xl bg-mymolt-yellow hover:bg-yellow-400 disabled:bg-mymolt-glass disabled:text-mymolt-text-muted/30 text-black font-bold text-sm transition-all flex items-center justify-center gap-2 shadow-lg shadow-yellow-500/20"
                                        >
                                            {modalStatus === 'loading' ? <Loader2 size={16} className="animate-spin" /> : <Wallet size={16} />}
                                            {modalStatus === 'success' ? 'Verified!' : 'Verify VP'}
                                        </button>
                                    </div>
                                </div>
                            )}
                        </motion.div>
                    </motion.div>
                )}
            </AnimatePresence>
        </div>
    );
};

// ── Status Banner Component ──────────────────────────────────────────
const StatusBanner: React.FC<{ status: string; message: string }> = ({ status, message }) => {
    if (status === 'idle' || !message) return null;

    const styles = {
        loading: 'bg-mymolt-blue/10 border-mymolt-blue/20 text-blue-300',
        success: 'bg-green-500/10 border-green-500/20 text-green-300',
        error: 'bg-mymolt-red/10 border-mymolt-red/20 text-red-300',
    };

    const icons = {
        loading: <Loader2 size={14} className="animate-spin" />,
        success: <CheckCircle2 size={14} />,
        error: <AlertCircle size={14} />,
    };

    const style = styles[status as keyof typeof styles] || styles.loading;
    const icon = icons[status as keyof typeof icons] || icons.loading;

    return (
        <motion.div
            initial={{ opacity: 0, height: 0 }}
            animate={{ opacity: 1, height: 'auto' }}
            className={`flex items-center gap-2 px-4 py-3 rounded-xl border text-sm font-medium ${style}`}
        >
            {icon}
            {message}
        </motion.div>
    );
};
