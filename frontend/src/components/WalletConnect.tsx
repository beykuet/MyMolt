import React, { useState } from 'react';
import { QrCode, ShieldCheck, RefreshCw } from 'lucide-react';
import { motion, AnimatePresence } from 'framer-motion';

interface WalletConnectProps {
    onConnected: () => void;
}

export const WalletConnect: React.FC<WalletConnectProps> = ({ onConnected }) => {
    const [step, setStep] = useState<'idle' | 'scanning' | 'verifying' | 'verified' | 'error'>('idle');
    const [error, setError] = useState<string | null>(null);

    const startVerification = async () => {
        setStep('scanning');
        setError(null);

        // Simulate scanning delay
        setTimeout(async () => {
            setStep('verifying');

            try {
                // Construct a dummy VP
                const dummyVP = {
                    "@context": ["https://www.w3.org/2018/credentials/v1"],
                    "type": ["VerifiablePresentation"],
                    "holder": "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
                    "verifiableCredential": [
                        {
                            "@context": ["https://www.w3.org/2018/credentials/v1"],
                            "type": ["VerifiableCredential"],
                            "issuer": "did:key:z6Mkqfvd627zWkTrk74W4xVqWjT5rJ6qYy5Xq3z1G5d3f2",
                            "issuanceDate": new Date().toISOString(),
                            "credentialSubject": {
                                "id": "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
                                "degree": {
                                    "type": "BachelorDegree",
                                    "name": "Bachelor of Science and Arts"
                                }
                            }
                        }
                    ]
                };

                const token = localStorage.getItem('mymolt_token');
                const res = await fetch('/api/identity/verify-vp', {
                    method: 'POST',
                    headers: {
                        'Content-Type': 'application/json',
                        ...(token ? { 'Authorization': `Bearer ${token}` } : {})
                    },
                    body: JSON.stringify({ vp: JSON.stringify(dummyVP) })
                });

                const data = await res.json();

                if (data.success) {
                    setStep('verified');
                    setTimeout(() => {
                        onConnected();
                        setStep('idle');
                    }, 2000);
                } else {
                    throw new Error(data.error || 'Verification failed');
                }
            } catch (e: any) {
                console.error(e);
                setError(e.message);
                setStep('error');
            }
        }, 2000);
    };

    return (
        <div className="p-6 bg-zinc-900 border border-zinc-800 rounded-xl relative overflow-hidden">
            <div className="absolute top-0 left-0 w-full h-1 bg-gradient-to-r from-blue-500 to-purple-500 opacity-20" />

            <div className="flex flex-col items-center text-center space-y-4">
                <AnimatePresence mode="wait">
                    {step === 'idle' && (
                        <motion.div
                            key="idle"
                            initial={{ opacity: 0, y: 10 }}
                            animate={{ opacity: 1, y: 0 }}
                            exit={{ opacity: 0, y: -10 }}
                            className="flex flex-col items-center"
                        >
                            <div className="w-12 h-12 rounded-full bg-blue-500/10 flex items-center justify-center text-blue-400 mb-3">
                                <ShieldCheck className="w-6 h-6" />
                            </div>
                            <h3 className="font-bold text-zinc-100 mb-1">SSI Wallet</h3>
                            <p className="text-xs text-zinc-400 mb-4 px-4">
                                Connect your self-sovereign identity wallet to verify credentials.
                            </p>
                            <button
                                onClick={startVerification}
                                className="flex items-center gap-2 px-4 py-2 bg-blue-600 hover:bg-blue-700 text-white rounded-lg text-sm font-medium transition-colors"
                            >
                                <QrCode className="w-4 h-4" />
                                Connect Wallet
                            </button>
                        </motion.div>
                    )}

                    {step === 'scanning' && (
                        <motion.div
                            key="scanning"
                            initial={{ opacity: 0, scale: 0.95 }}
                            animate={{ opacity: 1, scale: 1 }}
                            exit={{ opacity: 0, scale: 1.05 }}
                            className="flex flex-col items-center"
                        >
                            <div className="relative w-32 h-32 bg-white p-2 rounded-lg mb-4 overflow-hidden">
                                <div className="absolute inset-0 border-4 border-blue-500/30 rounded-lg animate-pulse z-10" />
                                <div className="w-full h-full bg-zinc-900 rounded flex items-center justify-center">
                                    <QrCode className="w-16 h-16 text-white" />
                                </div>
                                <div className="absolute top-0 left-0 w-full h-1 bg-blue-500 shadow-[0_0_10px_rgba(59,130,246,0.5)] animate-[scan_2s_ease-in-out_infinite] z-20" />
                            </div>
                            <p className="text-sm font-medium animate-pulse text-zinc-300">Scanning QR Code...</p>
                        </motion.div>
                    )}

                    {step === 'verifying' && (
                        <motion.div
                            key="verifying"
                            initial={{ opacity: 0 }}
                            animate={{ opacity: 1 }}
                            exit={{ opacity: 0 }}
                            className="flex flex-col items-center"
                        >
                            <RefreshCw className="w-12 h-12 text-purple-500 animate-spin mb-4" />
                            <p className="text-sm font-medium text-zinc-200">Verifying Credentials...</p>
                            <p className="text-xs text-zinc-500 mt-1">Checking signature validity</p>
                        </motion.div>
                    )}

                    {step === 'verified' && (
                        <motion.div
                            key="verified"
                            initial={{ opacity: 0, scale: 0.5 }}
                            animate={{ opacity: 1, scale: 1 }}
                            className="flex flex-col items-center"
                        >
                            <div className="w-16 h-16 rounded-full bg-green-500/20 flex items-center justify-center text-green-400 mb-4">
                                <ShieldCheck className="w-10 h-10" />
                            </div>
                            <h3 className="font-bold text-green-400">Identity Verified</h3>
                            <p className="text-xs text-zinc-500 mt-1">Wallet successfully connected</p>
                        </motion.div>
                    )}

                    {step === 'error' && (
                        <motion.div
                            key="error"
                            initial={{ opacity: 0 }}
                            animate={{ opacity: 1 }}
                            className="flex flex-col items-center"
                        >
                            <div className="text-red-500 font-bold mb-2">Error</div>
                            <p className="text-xs text-red-400 mb-4">{error}</p>
                            <button
                                onClick={() => setStep('idle')}
                                className="text-xs underline hover:text-zinc-200 text-zinc-400"
                            >
                                Try Again
                            </button>
                        </motion.div>
                    )}
                </AnimatePresence>
            </div>

            <style>{`
                @keyframes scan {
                    0%, 100% { top: 10%; }
                    50% { top: 90%; }
                }
            `}</style>
        </div>
    );
};
