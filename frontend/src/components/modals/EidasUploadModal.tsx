import { useState } from 'react';
import { apiClient } from '../../api/client';
import { X, ShieldCheck, Upload, Loader2, CheckCircle2, FileText } from 'lucide-react';
import { motion, AnimatePresence } from 'framer-motion';

interface Props {
    isOpen: boolean;
    onClose: () => void;
    onSuccess: () => void;
}

export default function EidasUploadModal({ isOpen, onClose, onSuccess }: Props) {
    const [step, setStep] = useState<1 | 2 | 3>(1);
    const [fileData, setFileData] = useState<string | null>(null);
    const [fileName, setFileName] = useState<string | null>(null);
    const [loading, setLoading] = useState(false);
    const [error, setError] = useState<string | null>(null);

    if (!isOpen) return null;

    const handleFileChange = (e: React.ChangeEvent<HTMLInputElement>) => {
        const file = e.target.files?.[0];
        if (!file) return;

        setFileName(file.name);
        setError(null);

        const reader = new FileReader();
        reader.onload = (event) => {
            const result = event.target?.result;
            if (typeof result === 'string') {
                // Keep base64 payload
                setFileData(result.split(',')[1] || result);
                setStep(2);
            }
        };
        reader.onerror = () => setError('Failed to read file.');
        reader.readAsDataURL(file);
    };

    const handleVerify = async () => {
        if (!fileData) return;
        setLoading(true);
        setError(null);

        try {
            const res = await apiClient.fetch('/identity/eidas/verify', {
                method: 'POST',
                body: JSON.stringify({
                    provider: 'eIDAS',
                    payload: fileData
                })
            });

            if (!res.ok) {
                const errData = await res.json().catch(() => null);
                throw new Error(errData?.error || 'Verification failed. Invalid or revoked certificate.');
            }

            setStep(3);
            setTimeout(() => {
                onSuccess();
                onClose();
                setStep(1);
                setFileData(null);
            }, 2000);
        } catch (err: any) {
            setError(err.message);
        } finally {
            setLoading(false);
        }
    };

    return (
        <AnimatePresence>
            <motion.div
                initial={{ opacity: 0 }}
                animate={{ opacity: 1 }}
                exit={{ opacity: 0 }}
                className="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/60 backdrop-blur-sm"
            >
                <motion.div
                    initial={{ scale: 0.95, opacity: 0 }}
                    animate={{ scale: 1, opacity: 1 }}
                    exit={{ scale: 0.95, opacity: 0 }}
                    className="bg-mymolt-glass border border-mymolt-glassBorder rounded-3xl w-full max-w-lg shadow-2xl overflow-hidden backdrop-blur-3xl"
                >
                    {/* Header */}
                    <div className="p-6 border-b border-white/5 flex justify-between items-center bg-gradient-to-r from-mymolt-primary/20 to-transparent">
                        <div>
                            <h3 className="text-xl font-bold flex items-center gap-2 text-white">
                                <ShieldCheck className="w-6 h-6 text-mymolt-primary" />
                                eIDAS Identity Binding
                            </h3>
                            <p className="text-xs text-white/50 mt-1 font-medium tracking-tight">EU Electronic Identification & Trust Services</p>
                        </div>
                        <button onClick={onClose} className="p-2 hover:bg-white/10 rounded-full transition-colors text-white">
                            <X className="w-5 h-5" />
                        </button>
                    </div>

                    {/* Progress Bar */}
                    <div className="flex w-full h-1 bg-white/5">
                        <div className={`h-full bg-mymolt-primary transition-all duration-500 w-${step}/3`} />
                    </div>

                    {/* Body */}
                    <div className="p-8 text-white min-h-[300px] flex flex-col justify-center">
                        {step === 1 && (
                            <div className="flex flex-col items-center text-center space-y-6">
                                <div className="w-20 h-20 rounded-full bg-mymolt-primary/10 border border-mymolt-primary/20 flex items-center justify-center">
                                    <FileText size={32} className="text-mymolt-primary" />
                                </div>
                                <div>
                                    <h4 className="text-lg font-bold">Select eIDAS Certificate</h4>
                                    <p className="text-sm text-white/50 mt-2 max-w-sm">Upload your valid national Identity Certificate (.cer, .pem, .p12) to elevate your Soul trust boundary to Level 3.</p>
                                </div>
                                <label className="cursor-pointer group relative overflow-hidden rounded-2xl">
                                    <div className="absolute inset-0 bg-mymolt-primary/20 group-hover:bg-mymolt-primary/30 transition-colors" />
                                    <div className="relative px-8 py-4 flex items-center gap-3 font-bold border border-mymolt-primary/50 rounded-2xl text-mymolt-primary group-hover:text-white transition-colors">
                                        <Upload size={20} /> Browse Vault
                                    </div>
                                    <input type="file" className="hidden" accept=".pem,.cer,.crt,.p12" onChange={handleFileChange} />
                                </label>
                                {error && <p className="text-xs font-bold text-red-400 mt-4">{error}</p>}
                            </div>
                        )}

                        {step === 2 && (
                            <div className="flex flex-col items-center text-center space-y-6">
                                <div className="w-full bg-black/30 border border-white/10 rounded-2xl p-6 relative overflow-hidden">
                                    <div className="absolute top-0 right-0 w-32 h-32 bg-mymolt-primary/10 rounded-full blur-2xl -translate-y-1/2 translate-x-1/2" />
                                    <h4 className="font-mono text-sm mb-2 opacity-50">DOCUMENT LOADED</h4>
                                    <p className="font-bold text-xl text-mymolt-primary break-all">{fileName}</p>
                                </div>
                                {error && (
                                    <div className="w-full bg-red-500/10 border border-red-500/20 text-red-500 rounded-xl p-4 text-sm font-bold">
                                        {error}
                                    </div>
                                )}
                                <button
                                    onClick={handleVerify}
                                    disabled={loading}
                                    className={`w-full py-4 rounded-2xl font-black uppercase tracking-widest text-xs transition-all flex items-center justify-center gap-2 shadow-lg ${loading ? 'bg-white/10 text-white/40 cursor-wait' : 'bg-mymolt-primary text-black hover:scale-[1.02] shadow-mymolt-primary/20'}`}
                                >
                                    {loading ? <><Loader2 className="animate-spin" size={16} /> Verifying Signature...</> : 'Cryptographically Verify'}
                                </button>
                                <button onClick={() => setStep(1)} className="text-xs text-white/40 hover:text-white transition">Cancel</button>
                            </div>
                        )}

                        {step === 3 && (
                            <div className="py-8 flex flex-col items-center text-center space-y-4">
                                <div className="w-24 h-24 bg-emerald-500/10 rounded-full flex items-center justify-center border border-emerald-500/20 mb-2 relative">
                                    <div className="absolute inset-0 bg-emerald-400/20 rounded-full blur-xl animate-pulse" />
                                    <CheckCircle2 className="w-12 h-12 text-emerald-400 relative z-10" />
                                </div>
                                <h4 className="text-2xl font-bold text-white tracking-tight">Identity Locked</h4>
                                <p className="text-sm text-white/50 max-w-[280px] font-medium leading-relaxed">
                                    Your Soul is now cryptographically bound to an eIDAS authority. Trust Level elevated.
                                </p>
                            </div>
                        )}
                    </div>
                </motion.div>
            </motion.div>
        </AnimatePresence>
    );
}
