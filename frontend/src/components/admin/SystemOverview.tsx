import React, { useState, useEffect } from 'react';
import { Activity, Database, HardDrive, FileText, Radio, Container, Download, RotateCcw, Brain, BookOpen, Users2, Calendar } from 'lucide-react';
import { apiClient } from '../../api/client';

interface SystemData {
    version: string;
    uptime: string;
    cpu_percent: number;
    memory_mb: number;
    disk_mb: number;
    memory_entries: number;
    memory_categories: Record<string, number>;
    embedding_percent: number;
    embedding_cache_hits: number;
    pim_contacts: number;
    pim_events: number;
    pim_notes: number;
    pim_storage_mb: number;
    audit_events: number;
    audit_file_mb: number;
    audit_rotated: number;
    channels: { name: string; status: string }[];
    docker_ready: boolean;
}

const StatCard: React.FC<{ label: string; value: string | number; sub?: string; icon: React.ElementType; accent?: string }> = ({ label, value, sub, icon: Icon, accent = 'text-mymolt-yellow' }) => (
    <div className="bg-[#1e2435] rounded-2xl border border-white/[0.06] p-5">
        <div className="flex items-center gap-2 mb-3">
            <Icon size={14} className={accent} />
            <span className="text-[10px] font-bold uppercase tracking-widest text-white/40">{label}</span>
        </div>
        <p className="text-2xl font-bold">{value}</p>
        {sub && <p className="text-[11px] text-white/30 mt-1">{sub}</p>}
    </div>
);

export const SystemOverview: React.FC = () => {
    const [sys, setSys] = useState<SystemData | null>(null);

    useEffect(() => {
        const fetchStatus = () => {
            apiClient.fetch('/system/status')
                .then(r => r.ok ? r.json() : null)
                .then(data => { if (data) setSys(data); })
                .catch(() => { });
        };
        fetchStatus();
        const interval = setInterval(fetchStatus, 15000);
        return () => clearInterval(interval);
    }, []);

    const s = sys || {
        version: 'v1.0',
        uptime: '—',
        cpu_percent: 0,
        memory_mb: 0,
        disk_mb: 0,
        memory_entries: 0,
        memory_categories: {},
        embedding_percent: 0,
        embedding_cache_hits: 0,
        pim_contacts: 0,
        pim_events: 0,
        pim_notes: 0,
        pim_storage_mb: 0,
        audit_events: 0,
        audit_file_mb: 0,
        audit_rotated: 0,
        channels: [],
        docker_ready: false,
    };

    return (
        <div>
            <h2 className="text-2xl font-bold tracking-tight mb-2">System Overview</h2>
            <p className="text-white/40 text-sm mb-8">Version {s.version} · Uptime {s.uptime}</p>

            {/* Top stats */}
            <div className="grid grid-cols-4 gap-4 mb-6">
                <StatCard label="CPU" value={`${s.cpu_percent}%`} icon={Activity} />
                <StatCard label="Memory" value={`${s.memory_mb} MB`} icon={Database} />
                <StatCard label="Disk" value={s.disk_mb < 1024 ? `${s.disk_mb} MB` : `${(s.disk_mb / 1024).toFixed(1)} GB`} icon={HardDrive} />
                <StatCard label="Version" value={s.version} icon={Activity} accent="text-blue-400" />
            </div>

            {/* Memory Brain */}
            <div className="bg-[#1e2435] rounded-2xl border border-white/[0.06] p-6 mb-6">
                <div className="flex items-center justify-between mb-5">
                    <div className="flex items-center gap-2">
                        <Brain size={16} className="text-mymolt-yellow" />
                        <h3 className="text-sm font-bold uppercase tracking-wider text-white/60">Memory Brain</h3>
                    </div>
                    <div className="flex gap-2">
                        <button className="text-xs font-medium px-3 py-1.5 rounded-lg bg-white/[0.04] hover:bg-white/[0.08] border border-white/[0.06] transition">
                            Reindex
                        </button>
                        <button className="text-xs font-medium px-3 py-1.5 rounded-lg bg-white/[0.04] hover:bg-white/[0.08] border border-white/[0.06] transition">
                            Hygiene
                        </button>
                        <button className="text-xs font-medium px-3 py-1.5 rounded-lg bg-white/[0.04] hover:bg-white/[0.08] border border-white/[0.06] transition flex items-center gap-1">
                            <Download size={12} /> Export
                        </button>
                    </div>
                </div>
                <div className="grid grid-cols-3 gap-6 mb-4">
                    <div>
                        <p className="text-3xl font-bold">{s.memory_entries}</p>
                        <p className="text-[10px] font-bold uppercase tracking-widest text-white/40 mt-1">Entries</p>
                    </div>
                    <div>
                        <p className="text-3xl font-bold">{s.embedding_percent}%</p>
                        <p className="text-[10px] font-bold uppercase tracking-widest text-white/40 mt-1">Embeddings</p>
                    </div>
                    <div>
                        <p className="text-3xl font-bold">{s.embedding_cache_hits.toLocaleString()}</p>
                        <p className="text-[10px] font-bold uppercase tracking-widest text-white/40 mt-1">Cache Hits</p>
                    </div>
                </div>
                <div className="text-xs text-white/30">
                    Backend: SQLite (hybrid vector + FTS5) · Categories: {Object.entries(s.memory_categories).map(([k, v]) => `${k}(${v})`).join(', ') || '—'}
                </div>
            </div>

            {/* PIM Data */}
            <div className="bg-[#1e2435] rounded-2xl border border-white/[0.06] p-6 mb-6">
                <div className="flex items-center gap-2 mb-5">
                    <BookOpen size={16} className="text-mymolt-yellow" />
                    <h3 className="text-sm font-bold uppercase tracking-wider text-white/60">PIM Data</h3>
                </div>
                <div className="grid grid-cols-4 gap-4">
                    <div className="text-center p-4 rounded-xl bg-black/20">
                        <Users2 size={20} className="mx-auto mb-2 text-blue-400" />
                        <p className="text-xl font-bold">{s.pim_contacts}</p>
                        <p className="text-[10px] text-white/40 uppercase tracking-widest font-bold mt-1">Contacts</p>
                    </div>
                    <div className="text-center p-4 rounded-xl bg-black/20">
                        <Calendar size={20} className="mx-auto mb-2 text-emerald-400" />
                        <p className="text-xl font-bold">{s.pim_events}</p>
                        <p className="text-[10px] text-white/40 uppercase tracking-widest font-bold mt-1">Events</p>
                    </div>
                    <div className="text-center p-4 rounded-xl bg-black/20">
                        <FileText size={20} className="mx-auto mb-2 text-amber-400" />
                        <p className="text-xl font-bold">{s.pim_notes}</p>
                        <p className="text-[10px] text-white/40 uppercase tracking-widest font-bold mt-1">Notes</p>
                    </div>
                    <div className="text-center p-4 rounded-xl bg-black/20">
                        <HardDrive size={20} className="mx-auto mb-2 text-white/40" />
                        <p className="text-xl font-bold">{s.pim_storage_mb}</p>
                        <p className="text-[10px] text-white/40 uppercase tracking-widest font-bold mt-1">MB Enc.</p>
                    </div>
                </div>
                <p className="text-xs text-white/20 mt-3">Encryption: ChaCha20-Poly1305 · All PIM data encrypted at rest</p>
            </div>

            {/* Audit & Channels */}
            <div className="grid grid-cols-2 gap-4">
                {/* Audit */}
                <div className="bg-[#1e2435] rounded-2xl border border-white/[0.06] p-6">
                    <div className="flex items-center justify-between mb-5">
                        <div className="flex items-center gap-2">
                            <FileText size={16} className="text-mymolt-yellow" />
                            <h3 className="text-sm font-bold uppercase tracking-wider text-white/60">Audit Log</h3>
                        </div>
                        <div className="flex gap-2">
                            <button className="text-xs font-medium px-3 py-1.5 rounded-lg bg-white/[0.04] hover:bg-white/[0.08] border border-white/[0.06] transition flex items-center gap-1">
                                <Download size={12} /> Download
                            </button>
                            <button className="text-xs font-medium px-3 py-1.5 rounded-lg bg-white/[0.04] hover:bg-white/[0.08] border border-white/[0.06] transition flex items-center gap-1">
                                <RotateCcw size={12} /> Rotate
                            </button>
                        </div>
                    </div>
                    <div className="space-y-3">
                        <div className="flex justify-between text-sm">
                            <span className="text-white/40">Total events</span>
                            <span className="font-medium">{s.audit_events.toLocaleString()}</span>
                        </div>
                        <div className="flex justify-between text-sm">
                            <span className="text-white/40">Current file</span>
                            <span className="font-medium">{s.audit_file_mb} MB</span>
                        </div>
                        <div className="flex justify-between text-sm">
                            <span className="text-white/40">Archived</span>
                            <span className="font-medium">{s.audit_rotated} files</span>
                        </div>
                    </div>
                </div>

                {/* Channels */}
                <div className="bg-[#1e2435] rounded-2xl border border-white/[0.06] p-6">
                    <div className="flex items-center gap-2 mb-5">
                        <Radio size={16} className="text-mymolt-yellow" />
                        <h3 className="text-sm font-bold uppercase tracking-wider text-white/60">Channels</h3>
                    </div>
                    <div className="space-y-2">
                        {(s.channels.length > 0 ? s.channels : [
                            { name: 'WebSocket', status: 'active' },
                            { name: 'CLI', status: 'active' },
                        ]).map(ch => (
                            <div key={ch.name} className="flex items-center justify-between py-2 border-b border-white/[0.04] last:border-0">
                                <span className="text-sm">{ch.name}</span>
                                <span className={`w-2 h-2 rounded-full ${ch.status === 'active' ? 'bg-emerald-400' : 'bg-white/20'}`} />
                            </div>
                        ))}
                    </div>
                    <button className="mt-4 w-full text-xs font-medium py-2 rounded-lg bg-white/[0.04] hover:bg-white/[0.08] border border-white/[0.06] transition">
                        Configure Channels
                    </button>
                </div>
            </div>

            {/* Docker */}
            <div className="mt-4 bg-[#1e2435] rounded-2xl border border-white/[0.06] p-6">
                <div className="flex items-center gap-2 mb-3">
                    <Container size={16} className="text-mymolt-yellow" />
                    <h3 className="text-sm font-bold uppercase tracking-wider text-white/60">Containerization</h3>
                </div>
                <div className="flex items-center gap-4 text-sm">
                    <span className={`flex items-center gap-1.5 ${s.docker_ready ? 'text-emerald-400' : 'text-white/30'}`}>
                        {s.docker_ready ? '✅' : '⬚'} Dockerfile
                    </span>
                    <span className="text-white/30">|</span>
                    <span className="text-white/50">docker-compose: MyMolt + Ollama + Hoodik</span>
                </div>
            </div>
        </div>
    );
};
