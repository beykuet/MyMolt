import React, { useState, useEffect } from 'react';
import { Shield, Network, ShieldAlert, Eye, Lock, Globe, AlertTriangle } from 'lucide-react';
import { apiClient } from '../../api/client';

interface SecurityOverview {
    trust_level: string;
    sandbox_backend: string;
    autonomy_level: string;
    confirmation_gate: boolean;
    pending_confirmations: number;
    audit_events_24h: number;
    blocked_actions_24h: number;
    vpn_status: string;
    vpn_provider: string;
    vpn_exit: string;
    vpn_uptime: string;
    adblock_enabled: boolean;
    adblock_count: number;
    tunnel_type: string;
    tunnel_status: string;
    tls_active: boolean;
    sensitivity_patterns: number;
    sensitivity_detections_24h: number;
}

const StatusBadge: React.FC<{ status: 'ok' | 'warn' | 'error'; label: string }> = ({ status, label }) => {
    const colors = {
        ok: 'bg-emerald-500/10 text-emerald-400 border-emerald-500/20',
        warn: 'bg-amber-500/10 text-amber-400 border-amber-500/20',
        error: 'bg-red-500/10 text-red-400 border-red-500/20',
    };
    return (
        <span className={`text-[10px] font-bold uppercase tracking-widest px-2.5 py-1 rounded-full border ${colors[status]}`}>
            {label}
        </span>
    );
};

const Card: React.FC<{ title: string; icon: React.ElementType; children: React.ReactNode }> = ({ title, icon: Icon, children }) => (
    <div className="bg-[#1e2435] rounded-2xl border border-white/[0.06] p-6">
        <div className="flex items-center gap-2 mb-5">
            <Icon size={16} className="text-mymolt-yellow" />
            <h3 className="text-sm font-bold uppercase tracking-wider text-white/60">{title}</h3>
        </div>
        {children}
    </div>
);

const StatRow: React.FC<{ label: string; value: string | React.ReactNode }> = ({ label, value }) => (
    <div className="flex items-center justify-between py-2.5 border-b border-white/[0.04] last:border-0">
        <span className="text-sm text-white/40">{label}</span>
        <span className="text-sm font-medium text-white">{value}</span>
    </div>
);

export const SecurityDashboard: React.FC = () => {
    const [overview, setOverview] = useState<SecurityOverview | null>(null);

    useEffect(() => {
        apiClient.fetch('/security/overview')
            .then(r => r.ok ? r.json() : null)
            .then(data => setOverview(data))
            .catch(() => { });
    }, []);

    // Fallback values for display
    const sec = overview || {
        trust_level: 'High',
        sandbox_backend: 'Docker',
        autonomy_level: 'Supervised',
        confirmation_gate: true,
        pending_confirmations: 0,
        audit_events_24h: 0,
        blocked_actions_24h: 0,
        vpn_status: 'disconnected',
        vpn_provider: 'WireGuard',
        vpn_exit: '—',
        vpn_uptime: '—',
        adblock_enabled: false,
        adblock_count: 0,
        tunnel_type: 'none',
        tunnel_status: 'inactive',
        tls_active: false,
        sensitivity_patterns: 7,
        sensitivity_detections_24h: 0,
    };

    return (
        <div>
            <h2 className="text-2xl font-bold tracking-tight mb-2">Security Dashboard</h2>
            <p className="text-white/40 text-sm mb-8">Real-time security posture</p>

            {/* Top Indicators */}
            <div className="grid grid-cols-3 gap-4 mb-6">
                <div className="bg-[#1e2435] rounded-2xl border border-white/[0.06] p-5 text-center">
                    <Shield size={24} className="mx-auto mb-2 text-mymolt-yellow" />
                    <p className="text-[10px] font-bold uppercase tracking-widest text-white/40 mb-1">Trust Level</p>
                    <p className="text-xl font-bold">{sec.trust_level}</p>
                </div>
                <div className="bg-[#1e2435] rounded-2xl border border-white/[0.06] p-5 text-center">
                    <Lock size={24} className="mx-auto mb-2 text-blue-400" />
                    <p className="text-[10px] font-bold uppercase tracking-widest text-white/40 mb-1">Sandbox</p>
                    <p className="text-xl font-bold">{sec.sandbox_backend}</p>
                </div>
                <div className="bg-[#1e2435] rounded-2xl border border-white/[0.06] p-5 text-center">
                    <AlertTriangle size={24} className="mx-auto mb-2 text-amber-400" />
                    <p className="text-[10px] font-bold uppercase tracking-widest text-white/40 mb-1">Autonomy</p>
                    <p className="text-xl font-bold">{sec.autonomy_level}</p>
                </div>
            </div>

            <div className="grid grid-cols-2 gap-4">
                {/* VPN */}
                <Card title="VPN" icon={Network}>
                    <StatRow label="Status" value={
                        sec.vpn_status === 'connected'
                            ? <StatusBadge status="ok" label="Connected" />
                            : <StatusBadge status="error" label="Disconnected" />
                    } />
                    <StatRow label="Provider" value={sec.vpn_provider} />
                    <StatRow label="Exit Node" value={sec.vpn_exit} />
                    <StatRow label="Uptime" value={sec.vpn_uptime} />
                    <div className="mt-4 flex gap-2">
                        <button className="flex-1 text-xs font-medium py-2 rounded-lg bg-white/[0.04] hover:bg-white/[0.08] transition border border-white/[0.06]">
                            {sec.vpn_status === 'connected' ? 'Disconnect' : 'Connect'}
                        </button>
                        <button className="flex-1 text-xs font-medium py-2 rounded-lg bg-white/[0.04] hover:bg-white/[0.08] transition border border-white/[0.06]">
                            Settings
                        </button>
                    </div>
                </Card>

                {/* DNS Shield */}
                <Card title="DNS Shield (AdBlock)" icon={ShieldAlert}>
                    <StatRow label="Status" value={
                        sec.adblock_enabled
                            ? <StatusBadge status="ok" label="Active" />
                            : <StatusBadge status="warn" label="Disabled" />
                    } />
                    <StatRow label="Blocked Today" value={sec.adblock_count.toLocaleString()} />
                    <StatRow label="Lists" value="EasyList, OISD" />
                    <div className="mt-4 flex gap-2">
                        <button className="flex-1 text-xs font-medium py-2 rounded-lg bg-white/[0.04] hover:bg-white/[0.08] transition border border-white/[0.06]">
                            {sec.adblock_enabled ? 'Disable' : 'Enable'}
                        </button>
                        <button className="flex-1 text-xs font-medium py-2 rounded-lg bg-white/[0.04] hover:bg-white/[0.08] transition border border-white/[0.06]">
                            Whitelist
                        </button>
                    </div>
                </Card>

                {/* Sensitivity Scanner */}
                <Card title="Sensitivity Scanner" icon={Eye}>
                    <StatRow label="Patterns" value={`${sec.sensitivity_patterns} active`} />
                    <StatRow label="Detections (24h)" value={
                        sec.sensitivity_detections_24h === 0
                            ? <StatusBadge status="ok" label="0 — Clean" />
                            : <StatusBadge status="warn" label={`${sec.sensitivity_detections_24h} detected`} />
                    } />
                    <StatRow label="Scans" value="API keys, IBANs, Credit cards, PINs" />
                    <div className="mt-4 flex gap-2">
                        <button className="flex-1 text-xs font-medium py-2 rounded-lg bg-white/[0.04] hover:bg-white/[0.08] transition border border-white/[0.06]">
                            Test Scanner
                        </button>
                        <button className="flex-1 text-xs font-medium py-2 rounded-lg bg-white/[0.04] hover:bg-white/[0.08] transition border border-white/[0.06]">
                            View Log
                        </button>
                    </div>
                </Card>

                {/* Cloud / Remote Security */}
                <Card title="Cloud Security" icon={Globe}>
                    <StatRow label="TLS" value={
                        sec.tls_active
                            ? <StatusBadge status="ok" label="Active" />
                            : <StatusBadge status="warn" label="Not configured" />
                    } />
                    <StatRow label="Tunnel" value={sec.tunnel_type === 'none' ? '—' : sec.tunnel_type} />
                    <StatRow label="Tunnel Status" value={
                        sec.tunnel_status === 'active'
                            ? <StatusBadge status="ok" label="Connected" />
                            : <StatusBadge status="error" label={sec.tunnel_status} />
                    } />
                    <div className="mt-4">
                        <StatRow label="Confirmation Gate" value={
                            sec.confirmation_gate
                                ? <StatusBadge status="ok" label="Enabled" />
                                : <StatusBadge status="warn" label="Disabled" />
                        } />
                        <StatRow label="Pending" value={sec.pending_confirmations.toString()} />
                    </div>
                </Card>

                {/* Audit Summary */}
                <div className="col-span-2">
                    <Card title="Audit Summary (24h)" icon={Shield}>
                        <div className="grid grid-cols-3 gap-6">
                            <div className="text-center">
                                <p className="text-3xl font-bold text-white">{sec.audit_events_24h}</p>
                                <p className="text-[10px] font-bold uppercase tracking-widest text-white/40 mt-1">Total Events</p>
                            </div>
                            <div className="text-center">
                                <p className="text-3xl font-bold text-emerald-400">{sec.audit_events_24h - sec.blocked_actions_24h}</p>
                                <p className="text-[10px] font-bold uppercase tracking-widest text-white/40 mt-1">Allowed</p>
                            </div>
                            <div className="text-center">
                                <p className="text-3xl font-bold text-red-400">{sec.blocked_actions_24h}</p>
                                <p className="text-[10px] font-bold uppercase tracking-widest text-white/40 mt-1">Blocked</p>
                            </div>
                        </div>
                        <div className="mt-4 flex justify-end">
                            <button className="text-xs font-medium text-mymolt-yellow hover:text-yellow-300 transition">
                                View Full Audit Log →
                            </button>
                        </div>
                    </Card>
                </div>
            </div>
        </div>
    );
};
