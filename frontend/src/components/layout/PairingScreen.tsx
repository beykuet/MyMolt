import { useState } from 'react';
import { motion } from 'framer-motion';
import { Mascot } from '../ui/Mascot';
import { Lock } from 'lucide-react';

interface PairingScreenProps {
    onPair: (code: string) => Promise<{ success: boolean; error?: string }>;
}

export function PairingScreen({ onPair }: PairingScreenProps) {
    const [code, setCode] = useState('');
    const [error, setError] = useState<string | null>(null);
    const [loading, setLoading] = useState(false);

    const handleSubmit = async (e: React.FormEvent) => {
        e.preventDefault();
        setLoading(true);
        setError(null);

        const result = await onPair(code);
        if (!result.success) {
            setError(result.error || 'Failed to pair');
        }
        setLoading(false);
    };

    return (
        <div className="min-h-screen bg-zinc-50 dark:bg-zinc-950 flex flex-col items-center justify-center p-4">
            <Mascot />

            <motion.div
                initial={{ opacity: 0, y: 20 }}
                animate={{ opacity: 1, y: 0 }}
                className="mt-8 bg-white dark:bg-zinc-900 p-8 rounded-2xl shadow-xl w-full max-w-md border border-zinc-200 dark:border-zinc-800"
            >
                <div className="text-center mb-6">
                    <h1 className="text-2xl font-bold text-zinc-800 dark:text-white mb-2">MyMolt Core</h1>
                    <p className="text-zinc-500 dark:text-zinc-400">Enter the pairing code from your server logs to connect.</p>
                </div>

                <form onSubmit={handleSubmit} className="flex flex-col gap-4">
                    <div>
                        <input
                            type="text"
                            value={code}
                            onChange={(e) => setCode(e.target.value)}
                            placeholder="Pairing Code (e.g., 123456)"
                            className="w-full px-4 py-3 rounded-xl border border-zinc-300 dark:border-zinc-700 bg-transparent text-center text-lg tracking-widest focus:ring-2 focus:ring-blue-500 outline-none transition-all"
                        />
                    </div>

                    {error && (
                        <div className="text-red-500 text-sm text-center bg-red-50 dark:bg-red-900/20 p-2 rounded-lg">
                            {error}
                        </div>
                    )}

                    <button
                        type="submit"
                        disabled={loading || !code}
                        className="w-full bg-gradient-to-r from-blue-600 to-purple-600 hover:from-blue-700 hover:to-purple-700 text-white font-bold py-3 rounded-xl shadow-lg disabled:opacity-50 disabled:cursor-not-allowed transition-all flex items-center justify-center gap-2"
                    >
                        {loading ? 'Verifying...' : (
                            <>
                                <Lock size={18} />
                                Secure Connect
                            </>
                        )}
                    </button>
                </form>
            </motion.div>
        </div>
    );
}
