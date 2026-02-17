import { motion } from 'framer-motion';
import { Mic } from 'lucide-react';

interface VoiceButtonProps {
    isListening: boolean;
    onClick: () => void;
}

export function VoiceButton({ isListening, onClick }: VoiceButtonProps) {
    return (
        <button onClick={onClick} className="relative group focus:outline-none">
            {/* Pulse Effect */}
            {isListening && (
                <span className="absolute inset-0 rounded-full bg-blue-400 opacity-75 animate-ping z-0"></span>
            )}

            {/* Main Button */}
            <motion.div
                whileHover={{ scale: 1.05 }}
                whileTap={{ scale: 0.95 }}
                className={`relative z-10 w-20 h-20 rounded-full flex items-center justify-center shadow-lg transition-all duration-300 ${isListening
                        ? 'bg-red-500 hover:bg-red-600 text-white'
                        : 'bg-gradient-to-tr from-blue-500 to-purple-600 text-white group-hover:shadow-blue-500/50'
                    }`}
            >
                <Mic size={32} className={isListening ? 'animate-pulse' : ''} />
            </motion.div>

            <span className="absolute -bottom-8 left-1/2 -translate-x-1/2 text-sm font-medium text-zinc-500 dark:text-zinc-400 opacity-0 group-hover:opacity-100 transition-opacity">
                {isListening ? 'Stop' : 'Talk'}
            </span>
        </button>
    );
}
