import { useState, useEffect } from 'react';
import QRCode from 'react-qr-code';
import { Shield, Plus, Trash2, Smartphone, Download, Server } from 'lucide-react';
import { motion, AnimatePresence } from 'framer-motion';

interface Peer {
    id: string;
    name: string;
    public_key: string;
    allowed_ips: string;
    created_at: string;
}

export const VPNManager = () => {
    const [peers, setPeers] = useState<Peer[]>([]);
    const [loading, setLoading] = useState(false);
    const [serverInit, setServerInit] = useState(false);
    const [newPeerConfig, setNewPeerConfig] = useState<string | null>(null);
    const [newPeerName, setNewPeerName] = useState('');
    const [showAddModal, setShowAddModal] = useState(false);

    const fetchPeers = async () => {
        try {
            const token = localStorage.getItem('mymolt_token');
            const res = await fetch('/api/vpn/peers', {
                headers: token ? { 'Authorization': `Bearer ${token}` } : {}
            });
            if (res.ok) {
                const data = await res.json();
                setPeers(data);
                setServerInit(true);
            } else {
                setServerInit(false); // Maybe 404/500 if config missing
            }
        } catch (e) {
            console.error("Failed to fetch VPN peers", e);
            setServerInit(false);
        }
    };

    useEffect(() => {
        fetchPeers();
    }, []);

    const initServer = async () => {
        setLoading(true);
        try {
            const token = localStorage.getItem('mymolt_token');
            const res = await fetch('/api/vpn/setup', {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json',
                    ...(token ? { 'Authorization': `Bearer ${token}` } : {})
                },
                body: JSON.stringify({
                    interface: 'wg0',
                    port: 51820,
                    cidr: '10.100.0.1/24'
                })
            });
            if (res.ok) {
                await fetchPeers();
                setServerInit(true);
            }
        } finally {
            setLoading(false);
        }
    };

    const addPeer = async () => {
        if (!newPeerName) return;
        setLoading(true);
        try {
            const token = localStorage.getItem('mymolt_token');
            const res = await fetch('/api/vpn/peers', {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json',
                    ...(token ? { 'Authorization': `Bearer ${token}` } : {})
                },
                body: JSON.stringify({ name: newPeerName })
            });

            if (res.ok) {
                const data = await res.json();
                setNewPeerConfig(data.client_config);
                await fetchPeers();
                setNewPeerName('');
            }
        } finally {
            setLoading(false);
        }
    };

    const deletePeer = async (id: string) => {
        if (!confirm('Are you sure you want to revoke this device?')) return;

        try {
            const token = localStorage.getItem('mymolt_token');
            await fetch(`/api/vpn/peers/${id}`, {
                method: 'DELETE',
                headers: token ? { 'Authorization': `Bearer ${token}` } : {}
            });
            await fetchPeers();
        } catch (e) {
            console.error(e);
        }
    };

    const downloadConfig = (config: string) => {
        const element = document.createElement("a");
        const file = new Blob([config], { type: 'text/plain' });
        element.href = URL.createObjectURL(file);
        element.download = "mymolt-vpn.conf";
        document.body.appendChild(element);
        element.click();
    };

    if (!serverInit && !loading && peers.length === 0) {
        return (
            <div className="p-6 bg-white dark:bg-zinc-900 border border-zinc-200 dark:border-zinc-800 rounded-xl">
                <div className="flex flex-col items-center text-center space-y-4 py-8">
                    <div className="p-4 bg-blue-100 dark:bg-blue-900/30 rounded-full text-blue-600 dark:text-blue-400">
                        <Server size={32} />
                    </div>
                    <h3 className="text-xl font-bold">Configure Private VPN</h3>
                    <p className="text-zinc-500 max-w-md">
                        Create a secure WireGuard network to connect your devices directly to MyMolt, anywhere in the world.
                    </p>
                    <button
                        onClick={initServer}
                        disabled={loading}
                        className="px-6 py-2 bg-blue-600 hover:bg-blue-700 text-white rounded-lg font-medium transition-colors flex items-center gap-2"
                    >
                        {loading ? 'Initializing...' : 'Create VPN Network'}
                    </button>
                </div>
            </div>
        );
    }

    return (
        <div className="p-6 bg-white dark:bg-zinc-900 border border-zinc-200 dark:border-zinc-800 rounded-xl relative overflow-hidden">

            <div className="flex items-center justify-between mb-6">
                <div className="flex items-center gap-3">
                    <div className="p-2 bg-green-100 dark:bg-green-900/30 rounded-lg text-green-600 dark:text-green-400">
                        <Shield size={20} />
                    </div>
                    <div>
                        <h3 className="font-bold text-lg">Secure VPN</h3>
                        <p className="text-xs text-zinc-500">10.100.0.1/24 • Port 51820</p>
                    </div>
                </div>
                <button
                    onClick={() => setShowAddModal(true)}
                    className="flex items-center gap-2 px-3 py-1.5 bg-zinc-100 hover:bg-zinc-200 dark:bg-zinc-800 dark:hover:bg-zinc-700 rounded-lg text-sm font-medium transition-colors"
                >
                    <Plus size={16} /> Add Device
                </button>
            </div>

            <div className="space-y-3">
                {peers.map(peer => (
                    <div key={peer.id} className="flex items-center justify-between p-3 bg-zinc-50 dark:bg-zinc-950/50 rounded-lg border border-zinc-100 dark:border-zinc-800">
                        <div className="flex items-center gap-3">
                            <div className="p-2 bg-zinc-200 dark:bg-zinc-800 rounded-full">
                                <Smartphone size={16} className="text-zinc-500" />
                            </div>
                            <div>
                                <div className="font-medium text-sm">{peer.name}</div>
                                <div className="text-xs text-zinc-500 font-mono">{peer.allowed_ips}</div>
                            </div>
                        </div>
                        <button
                            onClick={() => deletePeer(peer.id)}
                            className="p-2 hover:bg-red-100 dark:hover:bg-red-900/30 text-zinc-400 hover:text-red-500 rounded-lg transition-colors"
                        >
                            <Trash2 size={16} />
                        </button>
                    </div>
                ))}

                {peers.length === 0 && (
                    <div className="text-center py-8 text-zinc-500 text-sm">
                        No devices connected. Add one to get started.
                    </div>
                )}
            </div>

            {/* Config Modal */}
            <AnimatePresence>
                {(showAddModal || newPeerConfig) && (
                    <motion.div
                        initial={{ opacity: 0 }}
                        animate={{ opacity: 1 }}
                        exit={{ opacity: 0 }}
                        className="fixed inset-0 z-50 flex items-center justify-center bg-black/50 backdrop-blur-sm p-4"
                    >
                        <motion.div
                            initial={{ scale: 0.95, opacity: 0 }}
                            animate={{ scale: 1, opacity: 1 }}
                            exit={{ scale: 0.95, opacity: 0 }}
                            className="bg-white dark:bg-zinc-900 border border-zinc-200 dark:border-zinc-800 rounded-xl shadow-2xl w-full max-w-md p-6"
                        >
                            {newPeerConfig ? (
                                <div className="flex flex-col items-center space-y-6">
                                    <div className="text-center">
                                        <h3 className="text-xl font-bold mb-2">Device Configuration</h3>
                                        <p className="text-sm text-zinc-500">Scan this QR code with the WireGuard app.</p>
                                    </div>

                                    <div className="p-4 bg-white rounded-xl shadow-inner border border-zinc-200">
                                        <QRCode value={newPeerConfig} size={200} />
                                    </div>

                                    <div className="flex w-full gap-3">
                                        <button
                                            onClick={() => downloadConfig(newPeerConfig)}
                                            className="flex-1 px-4 py-2 bg-zinc-100 hover:bg-zinc-200 dark:bg-zinc-800 dark:hover:bg-zinc-700 rounded-lg font-medium text-sm flex items-center justify-center gap-2"
                                        >
                                            <Download size={16} /> Download
                                        </button>
                                        <button
                                            onClick={() => {
                                                setNewPeerConfig(null);
                                                setShowAddModal(false);
                                            }}
                                            className="flex-1 px-4 py-2 bg-blue-600 hover:bg-blue-700 text-white rounded-lg font-medium text-sm"
                                        >
                                            Done
                                        </button>
                                    </div>
                                </div>
                            ) : (
                                <div className="space-y-4">
                                    <div className="flex justify-between items-center">
                                        <h3 className="text-lg font-bold">Add New Device</h3>
                                        <button onClick={() => setShowAddModal(false)} className="text-zinc-500 hover:text-zinc-700">✕</button>
                                    </div>
                                    <div>
                                        <label className="block text-sm font-medium mb-1 text-zinc-700 dark:text-zinc-300">Device Name</label>
                                        <input
                                            type="text"
                                            value={newPeerName}
                                            onChange={(e) => setNewPeerName(e.target.value)}
                                            placeholder="e.g. Ben's iPhone"
                                            className="w-full px-3 py-2 bg-zinc-50 dark:bg-zinc-950 border border-zinc-200 dark:border-zinc-800 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
                                            autoFocus
                                        />
                                    </div>
                                    <div className="flex justify-end gap-2 pt-2">
                                        <button
                                            onClick={() => setShowAddModal(false)}
                                            className="px-4 py-2 text-sm text-zinc-500 hover:text-zinc-700 dark:hover:text-zinc-300"
                                        >
                                            Cancel
                                        </button>
                                        <button
                                            onClick={addPeer}
                                            disabled={loading || !newPeerName}
                                            className="px-4 py-2 bg-blue-600 hover:bg-blue-700 disabled:opacity-50 text-white rounded-lg text-sm font-medium"
                                        >
                                            {loading ? 'Generating...' : 'Generate Config'}
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
