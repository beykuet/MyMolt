import { Wifi, Battery, LogOut } from 'lucide-react';

interface HeaderProps {
    status: 'online' | 'offline' | 'connecting';
    onLogout?: () => void;
}

export function Header({ status, onLogout }: HeaderProps) {
    const statusColor = {
        'online': 'text-green-500',
        'offline': 'text-red-500',
        'connecting': 'text-yellow-500',
    }[status];

    return (
        <header className="fixed top-0 left-0 right-0 h-16 bg-white/50 dark:bg-zinc-950/50 backdrop-blur-md border-b border-zinc-200 dark:border-zinc-800 z-40 flex items-center justify-between px-6 md:pl-28">
            <div className="flex items-center gap-3">
                <span className="font-bold text-lg text-zinc-800 dark:text-white">MyMolt Core</span>
                <span className="px-2 py-0.5 rounded-full bg-zinc-100 dark:bg-zinc-800 text-xs font-medium text-zinc-500 dark:text-zinc-400">
                    v0.1.0
                </span>
            </div>

            <div className="flex items-center gap-6">
                <div className="flex items-center gap-2 text-sm text-zinc-500 dark:text-zinc-400">
                    <Wifi size={16} className={statusColor} />
                    <span className="capitalize hidden md:inline">{status}</span>
                </div>
                <div className="flex items-center gap-2 text-sm text-zinc-500 dark:text-zinc-400">
                    <Battery size={16} />
                    <span className="hidden md:inline">100%</span>
                </div>
                <div className="w-8 h-8 rounded-full bg-gradient-to-tr from-purple-500 to-pink-500 ring-2 ring-white dark:ring-zinc-900 shadow-md" />

                {onLogout && (
                    <button
                        onClick={onLogout}
                        className="p-2 -mr-2 text-zinc-400 hover:text-red-500 transition-colors"
                        title="Logout"
                    >
                        <LogOut size={20} />
                    </button>
                )}
            </div>
        </header>
    );
}
