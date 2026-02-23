import React, { useState, useEffect } from 'react';
import { Users, Plus, Trash2, Check } from 'lucide-react';
import { apiClient } from '../../api/client';
import type { UserRole } from '../../types';

interface FamilyMember {
    name: string;
    role: string;
    channels: Record<string, string>;
}

interface FamilyData {
    max_members: number;
    members: FamilyMember[];
}

const ROLES: { value: UserRole; label: string; color: string }[] = [
    { value: 'Root', label: 'Root', color: 'bg-red-500/20 text-red-400' },
    { value: 'Adult', label: 'Adult', color: 'bg-blue-500/20 text-blue-400' },
    { value: 'Senior', label: 'Senior', color: 'bg-white/10 text-white/70' },
    { value: 'Child', label: 'Child', color: 'bg-green-500/20 text-green-400' },
];

const CHANNEL_TYPES = ['telegram', 'whatsapp', 'discord', 'imessage', 'slack', 'matrix', 'email', 'websocket'];

export const FamilyManager: React.FC = () => {
    const [family, setFamily] = useState<FamilyData>({ max_members: 8, members: [] });
    const [isAdding, setIsAdding] = useState(false);
    const [newName, setNewName] = useState('');
    const [newRole, setNewRole] = useState<string>('adult');
    const [newChannel, setNewChannel] = useState('telegram');
    const [newChannelId, setNewChannelId] = useState('');

    useEffect(() => {
        apiClient.fetch('/family/members')
            .then(r => r.ok ? r.json() : null)
            .then(data => { if (data) setFamily(data); })
            .catch(() => { });
    }, []);

    const handleAdd = async () => {
        if (!newName.trim() || !newChannelId.trim()) return;
        const member: FamilyMember = {
            name: newName.trim(),
            role: newRole,
            channels: { [newChannel]: newChannelId.trim() },
        };
        try {
            const res = await apiClient.fetch('/family/members', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify(member),
            });
            if (res.ok) {
                setFamily(prev => ({
                    ...prev,
                    members: [...prev.members, member],
                }));
                setNewName('');
                setNewChannelId('');
                setIsAdding(false);
            }
        } catch { }
    };

    const handleRemove = async (name: string) => {
        try {
            const res = await apiClient.fetch(`/family/members/${encodeURIComponent(name)}`, { method: 'DELETE' });
            if (res.ok) {
                setFamily(prev => ({
                    ...prev,
                    members: prev.members.filter(m => m.name !== name),
                }));
            }
        } catch { }
    };

    const roleInfo = (role: string) => ROLES.find(r => r.value.toLowerCase() === role.toLowerCase()) || ROLES[1];

    return (
        <div>
            <div className="flex items-center justify-between mb-8">
                <div>
                    <h2 className="text-2xl font-bold tracking-tight">Family Members</h2>
                    <p className="text-white/40 text-sm mt-1">
                        {family.members.length}/{family.max_members} slots used
                    </p>
                </div>
                <button
                    onClick={() => setIsAdding(true)}
                    disabled={family.members.length >= family.max_members}
                    className="flex items-center gap-2 px-4 py-2 bg-mymolt-yellow/10 text-mymolt-yellow rounded-xl text-sm font-medium hover:bg-mymolt-yellow/20 transition disabled:opacity-30 disabled:cursor-not-allowed"
                >
                    <Plus size={16} />
                    Add Member
                </button>
            </div>

            <div className="space-y-3">
                {family.members.map((member) => {
                    const ri = roleInfo(member.role);
                    return (
                        <div
                            key={member.name}
                            className="flex items-center justify-between p-5 bg-[#1e2435] rounded-2xl border border-white/[0.06] hover:border-white/[0.1] transition group"
                        >
                            <div className="flex items-center gap-4">
                                <div className="w-10 h-10 rounded-full bg-white/[0.06] flex items-center justify-center text-lg">
                                    {member.role.toLowerCase() === 'child' ? '🧒' :
                                        member.role.toLowerCase() === 'senior' ? '👵' :
                                            member.role.toLowerCase() === 'root' ? '🛡️' : '👤'}
                                </div>
                                <div>
                                    <p className="font-semibold text-white">{member.name}</p>
                                    <div className="flex items-center gap-2 mt-1">
                                        <span className={`text-[10px] font-bold uppercase tracking-widest px-2 py-0.5 rounded-full ${ri.color}`}>
                                            {member.role}
                                        </span>
                                        {Object.entries(member.channels).map(([ch, id]) => (
                                            <span key={ch} className="text-[10px] text-white/30 font-mono">
                                                {ch}:{id.length > 12 ? id.slice(0, 12) + '…' : id}
                                            </span>
                                        ))}
                                    </div>
                                </div>
                            </div>
                            <button
                                onClick={() => handleRemove(member.name)}
                                className="opacity-0 group-hover:opacity-100 p-2 hover:bg-red-500/10 rounded-lg text-white/30 hover:text-red-400 transition"
                            >
                                <Trash2 size={16} />
                            </button>
                        </div>
                    );
                })}

                {family.members.length === 0 && !isAdding && (
                    <div className="text-center py-16 text-white/20">
                        <Users size={48} className="mx-auto mb-4 opacity-50" />
                        <p className="font-medium">No family members registered</p>
                        <p className="text-sm mt-1">Single-user mode is active</p>
                    </div>
                )}

                {isAdding && (
                    <div className="p-5 bg-[#1e2435] rounded-2xl border border-mymolt-yellow/20 space-y-4">
                        <div className="grid grid-cols-2 gap-4">
                            <input
                                value={newName}
                                onChange={(e) => setNewName(e.target.value)}
                                placeholder="Name"
                                autoFocus
                                className="bg-black/20 border border-white/10 rounded-xl px-4 py-2.5 text-sm focus:border-mymolt-yellow/50 outline-none transition"
                            />
                            <select
                                value={newRole}
                                onChange={(e) => setNewRole(e.target.value)}
                                className="bg-black/20 border border-white/10 rounded-xl px-4 py-2.5 text-sm focus:border-mymolt-yellow/50 outline-none appearance-none"
                            >
                                {ROLES.map(r => (
                                    <option key={r.value} value={r.value.toLowerCase()}>{r.label}</option>
                                ))}
                            </select>
                        </div>
                        <div className="grid grid-cols-3 gap-4">
                            <select
                                value={newChannel}
                                onChange={(e) => setNewChannel(e.target.value)}
                                className="bg-black/20 border border-white/10 rounded-xl px-4 py-2.5 text-sm focus:border-mymolt-yellow/50 outline-none appearance-none"
                            >
                                {CHANNEL_TYPES.map(ch => (
                                    <option key={ch} value={ch}>{ch}</option>
                                ))}
                            </select>
                            <input
                                value={newChannelId}
                                onChange={(e) => setNewChannelId(e.target.value)}
                                placeholder="Channel ID"
                                className="col-span-2 bg-black/20 border border-white/10 rounded-xl px-4 py-2.5 text-sm focus:border-mymolt-yellow/50 outline-none transition"
                            />
                        </div>
                        <div className="flex justify-end gap-2">
                            <button
                                onClick={() => setIsAdding(false)}
                                className="px-4 py-2 text-sm text-white/40 hover:text-white transition"
                            >
                                Cancel
                            </button>
                            <button
                                onClick={handleAdd}
                                disabled={!newName.trim() || !newChannelId.trim()}
                                className="flex items-center gap-2 px-4 py-2 bg-mymolt-yellow text-black rounded-xl text-sm font-bold hover:bg-yellow-400 disabled:opacity-30 transition"
                            >
                                <Check size={14} />
                                Add
                            </button>
                        </div>
                    </div>
                )}
            </div>
        </div>
    );
};
