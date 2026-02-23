import React, { useState, useEffect } from 'react';
import { useAuth } from '../context/AuthContext';
import { Lock, Loader2, ShieldCheck, Sparkles } from 'lucide-react';
import { motion, AnimatePresence } from 'framer-motion';
import { apiClient } from '../api/client';

interface FamilyMember {
    name: string;
    role: string;
    scope: string;
    channels: Record<string, string>;
}

const ROLE_COLORS: Record<string, { bg: string; glow: string; icon: string }> = {
    Root: { bg: 'linear-gradient(135deg, #dc2626, #991b1b)', glow: '0 0 32px rgba(220,38,38,0.3)', icon: '🛡️' },
    Adult: { bg: 'linear-gradient(135deg, #0055d4, #003da0)', glow: '0 0 32px rgba(0,85,212,0.3)', icon: '💼' },
    Child: { bg: 'linear-gradient(135deg, #22c55e, #16a34a)', glow: '0 0 32px rgba(34,197,94,0.3)', icon: '🌟' },
    Senior: { bg: 'linear-gradient(135deg, #f59e0b, #d97706)', glow: '0 0 32px rgba(245,158,11,0.3)', icon: '💛' },
};

const ROLE_DESCRIPTIONS: Record<string, string> = {
    Root: 'System Admin',
    Adult: 'Full Access',
    Child: 'Safe Mode',
    Senior: 'Voice First',
};

export const Lobby: React.FC = () => {
    const { login, logout } = useAuth();
    const [members, setMembers] = useState<FamilyMember[]>([]);
    const [loading, setLoading] = useState(true);
    const [selectedMember, setSelectedMember] = useState<FamilyMember | null>(null);
    const [pairingCode, setPairingCode] = useState('');
    const [isSubmitting, setIsSubmitting] = useState(false);
    const [error, setError] = useState('');
    const [time, setTime] = useState(new Date());

    // Clean slate
    useEffect(() => { logout(); }, [logout]);

    // Clock
    useEffect(() => {
        const t = setInterval(() => setTime(new Date()), 30000);
        return () => clearInterval(t);
    }, []);

    // Fetch family members
    useEffect(() => {
        const fetchMembers = async () => {
            try {
                // Try to fetch from API — if not authenticated, provide defaults from config
                const res = await apiClient.get('/family/members') as { members?: FamilyMember[] };
                setMembers(res.members || []);
            } catch {
                // Fallback: show role-based cards if family isn't configured
                setMembers([
                    { name: 'Owner', role: 'Root', scope: 'user:owner', channels: {} },
                    { name: 'Adult', role: 'Adult', scope: 'user:adult', channels: {} },
                    { name: 'Child', role: 'Child', scope: 'user:child', channels: {} },
                    { name: 'Senior', role: 'Senior', scope: 'user:senior', channels: {} },
                ]);
            }
            setLoading(false);
        };
        fetchMembers();
    }, []);

    const handleMemberSelect = (member: FamilyMember) => {
        if (member.role === 'Root') {
            setSelectedMember(member);
        } else {
            const token = localStorage.getItem('mymolt_pairing_token') || '';
            login(member.role as 'Root' | 'Adult' | 'Child' | 'Senior', token);
        }
    };

    const handlePairingSubmit = async (e: React.FormEvent) => {
        e.preventDefault();
        if (pairingCode.length !== 6) return;

        setIsSubmitting(true);
        setError('');
        const success = await login('Root', pairingCode);
        setIsSubmitting(false);

        if (!success) {
            setError('Invalid code. Check your logs.');
            setPairingCode('');
        }
    };

    const greeting = () => {
        const hour = time.getHours();
        if (hour < 6) return 'Good night';
        if (hour < 12) return 'Good morning';
        if (hour < 18) return 'Good afternoon';
        return 'Good evening';
    };

    const timeStr = time.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
    const dateStr = time.toLocaleDateString([], { weekday: 'long', month: 'long', day: 'numeric' });

    return (
        <div style={{
            minHeight: '100vh',
            background: 'linear-gradient(135deg, #0a0e18 0%, #0d1525 50%, #0a1020 100%)',
            display: 'flex',
            flexDirection: 'column',
            alignItems: 'center',
            justifyContent: 'center',
            padding: '24px',
            color: '#e2e8f0',
            fontFamily: "'Segoe UI', system-ui, -apple-system, sans-serif",
            position: 'relative',
            overflow: 'hidden',
        }}>
            {/* Ambient orbs */}
            <div style={{
                position: 'absolute', top: '-20%', left: '-10%',
                width: 500, height: 500, borderRadius: '50%',
                background: 'radial-gradient(circle, rgba(0,85,212,0.08) 0%, transparent 70%)',
                pointerEvents: 'none',
            }} />
            <div style={{
                position: 'absolute', bottom: '-20%', right: '-10%',
                width: 500, height: 500, borderRadius: '50%',
                background: 'radial-gradient(circle, rgba(255,204,0,0.06) 0%, transparent 70%)',
                pointerEvents: 'none',
            }} />

            <AnimatePresence mode="wait">
                {!selectedMember ? (
                    <motion.div
                        key="select"
                        initial={{ opacity: 0, y: 20 }}
                        animate={{ opacity: 1, y: 0 }}
                        exit={{ opacity: 0, y: -20 }}
                        style={{ maxWidth: 800, width: '100%', textAlign: 'center' }}
                    >
                        {/* Clock + Greeting */}
                        <motion.div
                            initial={{ opacity: 0, y: -10 }}
                            animate={{ opacity: 1, y: 0 }}
                            transition={{ delay: 0.1 }}
                            style={{ marginBottom: 48 }}
                        >
                            <div style={{
                                fontSize: 56, fontWeight: 200, letterSpacing: '-0.02em',
                                color: 'rgba(255,255,255,0.9)', lineHeight: 1,
                            }}>
                                {timeStr}
                            </div>
                            <div style={{
                                fontSize: 16, color: 'rgba(255,255,255,0.3)',
                                marginTop: 8, fontWeight: 400,
                            }}>
                                {dateStr}
                            </div>
                            <div style={{
                                fontSize: 24, fontWeight: 300,
                                marginTop: 24, color: 'rgba(255,255,255,0.5)',
                            }}>
                                {greeting()}. Who's using MyMolt?
                            </div>
                        </motion.div>

                        {/* Person grid */}
                        {loading ? (
                            <div style={{ display: 'flex', justifyContent: 'center', padding: 40 }}>
                                <Loader2 size={32} style={{ animation: 'spin 1s linear infinite', color: 'rgba(255,204,0,0.5)' }} />
                            </div>
                        ) : (
                            <div style={{
                                display: 'grid',
                                gridTemplateColumns: members.length <= 2
                                    ? `repeat(${members.length}, 1fr)`
                                    : members.length <= 4
                                        ? 'repeat(2, 1fr)'
                                        : 'repeat(3, 1fr)',
                                gap: 16,
                                maxWidth: members.length <= 2 ? 400 : 600,
                                margin: '0 auto',
                            }}>
                                {members.map((member, i) => (
                                    <PersonCard
                                        key={member.name}
                                        member={member}
                                        index={i}
                                        onClick={() => handleMemberSelect(member)}
                                    />
                                ))}
                            </div>
                        )}

                        {/* MyMolt branding */}
                        <motion.div
                            initial={{ opacity: 0 }}
                            animate={{ opacity: 1 }}
                            transition={{ delay: 0.5 }}
                            style={{
                                marginTop: 48,
                                display: 'flex', alignItems: 'center', justifyContent: 'center', gap: 8,
                            }}
                        >
                            <div style={{
                                width: 24, height: 24, borderRadius: 8,
                                background: 'linear-gradient(135deg, #ffcc00, #f59e0b)',
                                display: 'flex', alignItems: 'center', justifyContent: 'center',
                                fontSize: 12, fontWeight: 900, color: '#000',
                            }}>M</div>
                            <span style={{
                                fontSize: 11, fontWeight: 700, letterSpacing: '0.15em',
                                textTransform: 'uppercase' as const,
                                color: 'rgba(255,255,255,0.12)',
                            }}>
                                MyMolt · Sovereign Runtime
                            </span>
                        </motion.div>
                    </motion.div>
                ) : (
                    <motion.div
                        key="auth"
                        initial={{ opacity: 0, scale: 0.95 }}
                        animate={{ opacity: 1, scale: 1 }}
                        exit={{ opacity: 0, scale: 0.95 }}
                        style={{ maxWidth: 420, width: '100%' }}
                    >
                        <form onSubmit={handlePairingSubmit} style={{ textAlign: 'center' }}>
                            {/* Back header */}
                            <div style={{ marginBottom: 32 }}>
                                <div style={{
                                    width: 80, height: 80, borderRadius: 24,
                                    background: ROLE_COLORS[selectedMember.role]?.bg || ROLE_COLORS.Adult.bg,
                                    boxShadow: ROLE_COLORS[selectedMember.role]?.glow,
                                    display: 'flex', alignItems: 'center', justifyContent: 'center',
                                    margin: '0 auto 16px',
                                }}>
                                    <Lock color="white" size={32} />
                                </div>
                                <div style={{ fontSize: 24, fontWeight: 600 }}>
                                    Hi, {selectedMember.name}
                                </div>
                                <div style={{ fontSize: 14, color: 'rgba(255,255,255,0.3)', marginTop: 4 }}>
                                    Enter the 6-digit pairing code from your logs
                                </div>
                            </div>

                            {/* Code input */}
                            <div style={{
                                background: 'rgba(255,255,255,0.03)',
                                border: '1px solid rgba(255,255,255,0.06)',
                                borderRadius: 20,
                                padding: 32,
                                backdropFilter: 'blur(20px)',
                            }}>
                                <input
                                    type="text"
                                    maxLength={6}
                                    placeholder="000000"
                                    value={pairingCode}
                                    onChange={(e) => {
                                        setPairingCode(e.target.value.replace(/\D/g, ''));
                                        setError('');
                                    }}
                                    autoFocus
                                    disabled={isSubmitting}
                                    style={{
                                        width: '100%',
                                        background: 'rgba(0,0,0,0.3)',
                                        border: error ? '2px solid rgba(239,68,68,0.5)' : '2px solid rgba(255,255,255,0.08)',
                                        borderRadius: 16,
                                        padding: '16px 24px',
                                        fontSize: 32,
                                        textAlign: 'center' as const,
                                        fontFamily: 'monospace',
                                        letterSpacing: '0.5em',
                                        color: '#e2e8f0',
                                        outline: 'none',
                                        transition: 'border-color 0.3s',
                                    }}
                                />

                                {error && (
                                    <div style={{ color: '#ef4444', fontSize: 12, marginTop: 8 }}>
                                        {error}
                                    </div>
                                )}

                                <div style={{ display: 'flex', gap: 8, marginTop: 16 }}>
                                    <button
                                        type="button"
                                        onClick={() => { setSelectedMember(null); setPairingCode(''); setError(''); }}
                                        style={{
                                            flex: 1,
                                            padding: '12px',
                                            borderRadius: 12,
                                            border: '1px solid rgba(255,255,255,0.08)',
                                            background: 'transparent',
                                            color: 'rgba(255,255,255,0.5)',
                                            fontWeight: 600,
                                            fontSize: 14,
                                            cursor: 'pointer',
                                        }}
                                    >
                                        Back
                                    </button>
                                    <button
                                        type="submit"
                                        disabled={pairingCode.length !== 6 || isSubmitting}
                                        style={{
                                            flex: 2,
                                            padding: '12px',
                                            borderRadius: 12,
                                            border: 'none',
                                            background: pairingCode.length === 6
                                                ? 'linear-gradient(135deg, #ffcc00, #f59e0b)'
                                                : 'rgba(255,255,255,0.04)',
                                            color: pairingCode.length === 6 ? '#000' : 'rgba(255,255,255,0.2)',
                                            fontWeight: 700,
                                            fontSize: 14,
                                            cursor: pairingCode.length === 6 ? 'pointer' : 'default',
                                            transition: 'all 0.3s',
                                        }}
                                    >
                                        {isSubmitting ? (
                                            <span style={{ display: 'flex', alignItems: 'center', justifyContent: 'center', gap: 8 }}>
                                                <Loader2 size={16} style={{ animation: 'spin 1s linear infinite' }} />
                                                Verifying...
                                            </span>
                                        ) : (
                                            <>
                                                <ShieldCheck size={14} style={{ display: 'inline', marginRight: 6, verticalAlign: 'middle' }} />
                                                Authorize
                                            </>
                                        )}
                                    </button>
                                </div>
                            </div>
                        </form>
                    </motion.div>
                )}
            </AnimatePresence>

            <style>{`
                @keyframes spin {
                    from { transform: rotate(0deg); }
                    to { transform: rotate(360deg); }
                }
                input::placeholder { color: rgba(255,255,255,0.1); }
                input:focus { border-color: rgba(255,204,0,0.4) !important; }
                button:hover { filter: brightness(1.05); }
            `}</style>
        </div>
    );
};

// ── Person Card ────────────────────────────────────────────────────

const PersonCard: React.FC<{
    member: FamilyMember;
    index: number;
    onClick: () => void;
}> = ({ member, index, onClick }) => {
    const colors = ROLE_COLORS[member.role] || ROLE_COLORS.Adult;
    const initials = member.name.split(' ').map(w => w[0]).join('').slice(0, 2).toUpperCase();
    const isRoot = member.role === 'Root';

    return (
        <motion.button
            initial={{ opacity: 0, y: 20 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ delay: index * 0.08 }}
            whileHover={{ scale: 1.04, y: -4 }}
            whileTap={{ scale: 0.98 }}
            onClick={onClick}
            style={{
                display: 'flex',
                flexDirection: 'column',
                alignItems: 'center',
                gap: 12,
                padding: '28px 20px',
                borderRadius: 24,
                background: 'rgba(255,255,255,0.02)',
                border: '1px solid rgba(255,255,255,0.05)',
                backdropFilter: 'blur(20px)',
                cursor: 'pointer',
                transition: 'all 0.3s',
                position: 'relative',
                overflow: 'hidden',
            }}
            onMouseEnter={(e) => {
                (e.currentTarget as HTMLElement).style.background = 'rgba(255,255,255,0.05)';
                (e.currentTarget as HTMLElement).style.borderColor = 'rgba(255,255,255,0.1)';
                (e.currentTarget as HTMLElement).style.boxShadow = colors.glow;
            }}
            onMouseLeave={(e) => {
                (e.currentTarget as HTMLElement).style.background = 'rgba(255,255,255,0.02)';
                (e.currentTarget as HTMLElement).style.borderColor = 'rgba(255,255,255,0.05)';
                (e.currentTarget as HTMLElement).style.boxShadow = 'none';
            }}
        >
            {/* Root lock indicator */}
            {isRoot && (
                <div style={{
                    position: 'absolute', top: 8, right: 8,
                    opacity: 0.3,
                }}>
                    <Lock size={14} />
                </div>
            )}

            {/* Avatar */}
            <div style={{
                width: 64, height: 64, borderRadius: 20,
                background: colors.bg,
                display: 'flex', alignItems: 'center', justifyContent: 'center',
                fontSize: 22, fontWeight: 700, color: '#fff',
                boxShadow: `0 4px 20px ${colors.glow.match(/rgba\([^)]+\)/)?.[0] || 'transparent'}`,
                position: 'relative',
            }}>
                {initials}
                {/* Sparkle for children */}
                {member.role === 'Child' && (
                    <Sparkles size={14} color="#ffcc00" style={{
                        position: 'absolute', top: -4, right: -4,
                    }} />
                )}
            </div>

            {/* Name */}
            <div style={{ textAlign: 'center' as const }}>
                <div style={{
                    fontSize: 16, fontWeight: 600,
                    color: 'rgba(255,255,255,0.9)',
                }}>
                    {member.name}
                </div>
                <div style={{
                    fontSize: 10, fontWeight: 700,
                    letterSpacing: '0.12em',
                    textTransform: 'uppercase' as const,
                    color: 'rgba(255,255,255,0.2)',
                    marginTop: 2,
                }}>
                    {ROLE_DESCRIPTIONS[member.role] || member.role}
                </div>
            </div>

            {/* Role emoji */}
            <div style={{
                fontSize: 18, marginTop: -4,
                filter: 'saturate(0.7)',
            }}>
                {colors.icon}
            </div>
        </motion.button>
    );
};
