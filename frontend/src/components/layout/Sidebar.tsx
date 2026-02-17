import { Home, Settings, Mic, LayoutGrid, Shield } from 'lucide-react';
import { motion } from 'framer-motion';

const navItems = [
    { icon: <Home size={24} />, label: 'Home', active: true },
    { icon: <LayoutGrid size={24} />, label: 'Dashboard' },
    { icon: <Mic size={24} />, label: 'Voice' },
    { icon: <Shield size={24} />, label: 'Security' },
    { icon: <Settings size={24} />, label: 'Settings' },
];

export function Sidebar() {
    return (
        <motion.aside
            initial={{ x: -100 }}
            animate={{ x: 0 }}
            className="fixed left-0 top-0 h-full w-20 bg-white dark:bg-zinc-950 border-r border-zinc-200 dark:border-zinc-800 flex flex-col items-center py-8 z-50 hidden md:flex"
        >
            <div className="mb-12">
                <div className="w-10 h-10 bg-gradient-to-br from-green-400 to-blue-500 rounded-xl shadow-lg" />
            </div>

            <nav className="flex flex-col gap-8 w-full items-center">
                {navItems.map((item, idx) => (
                    <button
                        key={idx}
                        className={`p-3 rounded-xl transition-all ${item.active
                                ? 'bg-blue-50 dark:bg-blue-900/20 text-blue-600 dark:text-blue-400'
                                : 'text-zinc-400 hover:text-zinc-600 dark:hover:text-zinc-200 hover:bg-zinc-100 dark:hover:bg-zinc-800'
                            }`}
                    >
                        {item.icon}
                    </button>
                ))}
            </nav>
        </motion.aside>
    );
}
