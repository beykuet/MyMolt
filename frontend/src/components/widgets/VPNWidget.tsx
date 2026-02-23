import { useState, useEffect } from 'react';
import { Shield, Plus, QrCode, X, Activity } from 'lucide-react';
import QRCode from 'react-qr-code';
import { motion, AnimatePresence } from 'framer-motion';
import { apiClient } from '../../api/client';

interface VpnPeer {
    id: string;
    name: string;
    allowed_ips: string;
    created_at: string;
}

export function VPNWidget() {
    const [peers, setPeers] = useState<VpnPeer[]>([]);
    const [loading, setLoading] = useState(false);
    const [showModal, setShowModal] = useState(false);
    const [newPeerConfig, setNewPeerConfig] = useState<string | null>(null);

    const fetchPeers = async () => {
        try {
            const res = await apiClient.fetch('/vpn/peers');
            if (res.ok) setPeers(await res.json());
        } catch (e) {
            console.error(e);
        }
    };

    useEffect(() => {
        fetchPeers();
    }, []);

    const handleCreatePeer = async () => {
        setLoading(true);
        try {
            const name = `Device-${Date.now().toString().slice(-4)}`;
            const res = await apiClient.fetch('/vpn/peers', {
                method: 'POST',
                body: JSON.stringify({ name })
            });

            if (res.ok) {
                const data = await res.json();
                setNewPeerConfig(data.config_file);
                setShowModal(true);
                fetchPeers();
            }
        } catch (e) {
            console.error(e);
        } finally {
            setLoading(false);
        }
    };

    return (
        <div className="glass-panel p-6 h-[500px] flex flex-col relative overflow-hidden">
            <div className="absolute top-0 right-0 w-32 h-32 bg-mymolt-red/20 blur-3xl rounded-full translate-x-10 -translate-y-10" />

            <div className="flex justify-between items-start mb-6 z-10">
                <div>
                    <h3 className="font-bold text-lg flex items-center gap-2">
                        <Shield size={20} className="text-mymolt-red" /> VPN Mesh
                    </h3>
                    <p className="text-sm text-mymolt-text-muted mt-1">
                        {peers.length} Secure Tunnels Active
                    </p>
                </div>
                <button
                    onClick={handleCreatePeer}
                    disabled={loading}
                    className="bg-mymolt-yellow hover:bg-yellow-400 text-black px-4 py-2 rounded-lg font-bold flex items-center gap-2 text-sm transition-all shadow-lg hover:shadow-yellow-500/20 active:scale-95 disabled:opacity-50"
                >
                    {loading ? <Activity className="animate-spin" size={16} /> : <Plus size={16} />}
                    Add Device
                </button>
            </div>

            <div className="flex-1 overflow-y-auto space-y-3 custom-scrollbar pr-2 z-10">
                {peers.map((peer) => (
                    <div key={peer.id} className="p-4 bg-black/20 border border-mymolt-glassBorder rounded-2xl flex items-center justify-between group hover:border-mymolt-red/30 transition-all">
                        <div className="flex items-center gap-3">
                            <div className="w-2 h-2 rounded-full bg-green-500 animate-pulse" />
                            <div>
                                <div className="font-medium text-sm">{peer.name}</div>
                                <div className="text-xs text-mymolt-text-muted font-mono">{peer.allowed_ips}</div>
                            </div>
                        </div>
                        <button className="text-xs text-red-400 hover:text-red-300 opacity-0 group-hover:opacity-100 transition-opacity">
                            Revoke
                        </button>
                    </div>
                ))}
                {peers.length === 0 && (
                    <div className="text-center py-12 border-2 border-dashed border-white/10 rounded-xl text-mymolt-text-muted text-sm">
                        <QrCode className="mx-auto mb-3 opacity-50" size={32} />
                        No devices connected.<br />Add your phone to start.
                    </div>
                )}
            </div>

            {/* QR Code Modal via Framer Motion */}
            <AnimatePresence>
                {showModal && newPeerConfig && (
                    <motion.div
                        initial={{ opacity: 0 }}
                        animate={{ opacity: 1 }}
                        exit={{ opacity: 0 }}
                        className="absolute inset-0 z-50 bg-mymolt-blue/90 backdrop-blur-xl flex items-center justify-center p-6"
                    >
                        <motion.div
                            initial={{ scale: 0.9, y: 20 }}
                            animate={{ scale: 1, y: 0 }}
                            className="bg-white text-black p-8 rounded-[2rem] max-w-sm w-full shadow-[0_0_50px_rgba(0,0,0,0.5)] relative border border-white/20"
                        >
                            <button
                                onClick={() => setShowModal(false)}
                                className="absolute top-4 right-4 text-gray-500 hover:text-black"
                            >
                                <X size={24} />
                            </button>

                            <h3 className="font-black text-2xl mb-2 tracking-tight">Sovereign Link</h3>
                            <p className="text-sm text-gray-500 mb-8 leading-relaxed">Open the WireGuard app on your device and scan this secure pairing code.</p>

                            <div className="bg-gray-50 p-6 rounded-3xl shadow-inner border border-gray-100 aspect-square flex items-center justify-center mb-8">
                                <div className="h-full w-full">
                                    <QRCode value={newPeerConfig} size={256} style={{ height: "auto", maxWidth: "100%", width: "100%" }} />
                                </div>
                            </div>

                            <div className="flex flex-col gap-3">
                                <button className="w-full py-4 text-sm font-black uppercase tracking-widest text-white bg-mymolt-blue rounded-2xl hover:bg-mymolt-blue-subtle transition-all shadow-lg shadow-blue-500/20 active:scale-95">
                                    Download Config
                                </button>
                                <button onClick={() => setShowModal(false)} className="w-full py-3 text-sm font-bold text-gray-400 hover:text-gray-600 transition-colors">
                                    Dismiss
                                </button>
                            </div>
                        </motion.div>
                    </motion.div>
                )}
            </AnimatePresence>
        </div>
    );
}
