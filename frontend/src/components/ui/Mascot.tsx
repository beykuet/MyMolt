import { motion } from 'framer-motion';
import { useState } from 'react';

export function Mascot() {
    const [mood, setMood] = useState<'idle' | 'listening' | 'thinking'>('idle');

    return (
        <div className="relative w-64 h-64 flex items-center justify-center">
            {/* Placeholder for the 3D/Video Chameleon */}
            <motion.div
                animate={{
                    y: [0, -10, 0],
                    rotate: mood === 'thinking' ? [0, 5, -5, 0] : 0
                }}
                transition={{
                    y: { duration: 4, repeat: Infinity, ease: "easeInOut" },
                    rotate: { duration: 2, repeat: Infinity }
                }}
                className="text-9xl filter drop-shadow-2xl cursor-pointer"
                onClick={() => setMood(m => m === 'idle' ? 'thinking' : 'idle')}
            >
                🦎
            </motion.div>

            {/* Ambient Glow */}
            <div className="absolute inset-0 bg-green-500/20 blur-3xl rounded-full z-[-1]" />
        </div>
    );
}
