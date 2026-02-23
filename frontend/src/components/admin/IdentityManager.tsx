import React, { useState, useEffect } from 'react';
import { Fingerprint, ShieldCheck, KeyRound, FileText, Link2, ExternalLink, CheckCircle2 } from 'lucide-react';
import { apiClient } from '../../api/client';
import EidasUploadModal from '../modals/EidasUploadModal';

interface IdentityBinding {
    provider: string;
    id: string;
    trust_level: number;
    verified: boolean;
}

interface AuditEntry {
    timestamp: string;
    event_type: string;
    action: string;
    result: string;
}

const TrustMeter: React.FC<{ level: string }> = ({ level }) => {
    const levels: Record<string, { width: string; color: string }> = {
        Low: { width: 'w-1/3', color: 'bg-red-400' },
        Medium: { width: 'w-2/3', color: 'bg-amber-400' },
        High: { width: 'w-full', color: 'bg-emerald-400' },
    };
    const l = levels[level] || levels.Low;
    return (
        <div className="w-full h-1.5 bg-white/[0.06] rounded-full overflow-hidden">
            <div className={`h-full ${l.width} ${l.color} rounded-full transition-all duration-500`} />
        </div>
    );
};

export const IdentityManager: React.FC = () => {
    const [bindings, setBindings] = useState<IdentityBinding[]>([]);
    const [auditLog, setAuditLog] = useState<AuditEntry[]>([]);
    const [trustLevel, _setTrustLevel] = useState('High');
    const [isEidasModalOpen, setIsEidasModalOpen] = useState(false);

    const refreshBindings = () => {
        apiClient.fetch('/identity')
            .then(r => r.ok ? r.json() : [])
            .then(data => { if (Array.isArray(data)) setBindings(data); })
            .catch(() => { });
    };

    useEffect(() => {
        refreshBindings();
        apiClient.fetch('/security/sigil')
            .then(r => r.ok ? r.json() : [])
            .then(data => { if (Array.isArray(data)) setAuditLog(data.slice(0, 10)); })
            .catch(() => { });
    }, []);

    const identityProviders = [
        { type: 'eidas', label: 'eIDAS Certificate', icon: ShieldCheck, description: 'EU electronic identity' },
        { type: 'oidc', label: 'OIDC (OAuth)', icon: KeyRound, description: 'Auth0, Google, Azure AD' },
        { type: 'ssi', label: 'SSI / Verifiable Presentation', icon: FileText, description: 'Self-sovereign identity' },
        { type: 'did', label: 'DID (Decentralized ID)', icon: Link2, description: 'W3C Decentralized Identifier' },
    ];

    return (
        <div>
            <h2 className="text-2xl font-bold tracking-tight mb-2">Identity & SIGIL</h2>
            <p className="text-white/40 text-sm mb-8">SOUL.md identity, trust calibration, and Sigil audit</p>

            {/* Trust Level */}
            <div className="bg-[#1e2435] rounded-2xl border border-white/[0.06] p-6 mb-6">
                <div className="flex items-center justify-between mb-4">
                    <div className="flex items-center gap-2">
                        <Fingerprint size={16} className="text-mymolt-yellow" />
                        <h3 className="text-sm font-bold uppercase tracking-wider text-white/60">Trust Level</h3>
                    </div>
                    <span className="text-2xl font-bold">{trustLevel}</span>
                </div>
                <TrustMeter level={trustLevel} />
                <p className="text-xs text-white/30 mt-3">
                    Trust is computed from verified identity bindings. Higher trust unlocks more tools.
                </p>
            </div>

            {/* Identity Bindings */}
            <div className="bg-[#1e2435] rounded-2xl border border-white/[0.06] p-6 mb-6">
                <div className="flex items-center gap-2 mb-5">
                    <ShieldCheck size={16} className="text-mymolt-yellow" />
                    <h3 className="text-sm font-bold uppercase tracking-wider text-white/60">Identity Bindings</h3>
                </div>
                <div className="space-y-3">
                    {identityProviders.map((provider) => {
                        const binding = bindings.find(b => b.provider.toLowerCase().includes(provider.type));
                        const Icon = provider.icon;
                        return (
                            <div
                                key={provider.type}
                                className="flex items-center justify-between p-4 rounded-xl bg-black/20 border border-white/[0.04] hover:border-white/[0.08] transition"
                            >
                                <div className="flex items-center gap-3">
                                    <Icon size={18} className={binding?.verified ? 'text-emerald-400' : 'text-white/20'} />
                                    <div>
                                        <p className="text-sm font-medium">{provider.label}</p>
                                        <p className="text-[11px] text-white/30">{provider.description}</p>
                                    </div>
                                </div>
                                {binding?.verified ? (
                                    <div className="flex items-center gap-2">
                                        <CheckCircle2 size={14} className="text-emerald-400" />
                                        <span className="text-[10px] font-bold uppercase tracking-widest text-emerald-400">Verified</span>
                                        <button className="ml-2 text-xs text-white/30 hover:text-white transition">Manage</button>
                                    </div>
                                ) : (
                                    <button
                                        onClick={() => provider.type === 'eidas' ? setIsEidasModalOpen(true) : null}
                                        className="text-xs font-medium text-mymolt-yellow hover:text-yellow-300 transition flex items-center gap-1"
                                    >
                                        Configure <ExternalLink size={12} />
                                    </button>
                                )}
                            </div>
                        );
                    })}
                </div>
            </div>

            {/* Sigil Audit Log */}
            <div className="bg-[#1e2435] rounded-2xl border border-white/[0.06] p-6">
                <div className="flex items-center justify-between mb-5">
                    <div className="flex items-center gap-2">
                        <FileText size={16} className="text-mymolt-yellow" />
                        <h3 className="text-sm font-bold uppercase tracking-wider text-white/60">SIGIL Audit</h3>
                    </div>
                    <button className="text-xs font-medium text-mymolt-yellow hover:text-yellow-300 transition">
                        Export →
                    </button>
                </div>
                {auditLog.length > 0 ? (
                    <div className="space-y-1">
                        {auditLog.map((entry, i) => (
                            <div key={i} className="flex items-center gap-3 py-2 text-xs font-mono border-b border-white/[0.03] last:border-0">
                                <span className="text-white/20 w-14 flex-shrink-0">{entry.timestamp}</span>
                                <span className={entry.result === 'allowed' ? 'text-emerald-400' : 'text-red-400'}>
                                    {entry.result === 'allowed' ? '✅' : '🔒'}
                                </span>
                                <span className="text-white/50">{entry.event_type}</span>
                                <span className="text-white/70 truncate">{entry.action}</span>
                            </div>
                        ))}
                    </div>
                ) : (
                    <p className="text-center text-white/20 py-8 text-sm">
                        No audit events yet. Activity will appear here.
                    </p>
                )}
            </div>

            <EidasUploadModal
                isOpen={isEidasModalOpen}
                onClose={() => setIsEidasModalOpen(false)}
                onSuccess={refreshBindings}
            />
        </div>
    );
};
