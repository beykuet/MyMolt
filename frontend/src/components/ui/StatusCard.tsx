import type { ReactNode } from 'react';
import { motion } from 'framer-motion';

interface StatusCardProps {
    title: string;
    children: ReactNode;
    icon?: ReactNode;
    className?: string;
    onClick?: () => void;
}

export function StatusCard({ title, children, icon, className = '', onClick }: StatusCardProps) {
    return (
        <motion.div
            whileHover={{ scale: 1.02 }}
            className={`bg-white/80 dark:bg-zinc-900/80 backdrop-blur-md rounded-2xl p-6 shadow-sm border border-zinc-200 dark:border-zinc-800 flex flex-col gap-2 ${className}`}
            onClick={onClick}
        >
            <div className="flex items-center justify-between text-zinc-500 dark:text-zinc-400 mb-2">
                <h3 className="font-medium text-sm uppercase tracking-wider">{title}</h3>
                {icon}
            </div>
            <div className="text-zinc-800 dark:text-zinc-100 font-semibold text-lg">
                {children}
            </div>
        </motion.div>
    );
}
