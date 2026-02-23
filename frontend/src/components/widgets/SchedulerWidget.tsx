import React, { useState, useEffect } from 'react';
import { Calendar, Clock, Plus, Trash2, Play, Activity } from 'lucide-react';
import { motion, AnimatePresence } from 'framer-motion';
import { apiClient } from '../../api/client';

interface CronJob {
    id: string;
    expression: string;
    command: string;
    next_run: string;
    last_run: string | null;
    last_status: string | null;
}

export const SchedulerWidget: React.FC = () => {
    const [jobs, setJobs] = useState<CronJob[]>([]);
    const [loading, setLoading] = useState(true);
    const [isAdding, setIsAdding] = useState(false);
    const [newExpr, setNewExpr] = useState('0 9 * * *');
    const [newCmd, setNewCmd] = useState('agent -m "Good morning!"');

    const fetchJobs = async () => {
        try {
            const res = await apiClient.fetch('/system/cron');
            if (res.ok) {
                const data = await res.json();
                setJobs(data);
            }
        } catch (err) {
            console.error('Failed to fetch cron jobs', err);
        } finally {
            setLoading(false);
        }
    };

    useEffect(() => {
        fetchJobs();
        const interval = setInterval(fetchJobs, 15000);
        return () => clearInterval(interval);
    }, []);

    const handleAdd = async (e: React.FormEvent) => {
        e.preventDefault();
        try {
            await apiClient.fetch('/system/cron', {
                method: 'POST',
                body: JSON.stringify({ expression: newExpr, command: newCmd }),
            });
            setIsAdding(false);
            setNewExpr('0 9 * * *');
            setNewCmd('agent -m "Good morning!"');
            fetchJobs();
        } catch (err) {
            console.error('Failed to add job', err);
        }
    };

    const handleDelete = async (id: string) => {
        try {
            await apiClient.fetch(`/system/cron/${id}`, { method: 'DELETE' });
            fetchJobs();
        } catch (err) {
            console.error('Failed to delete job', err);
        }
    };

    const handleRunNow = async (id: string) => {
        try {
            await apiClient.fetch(`/system/cron/${id}/run`, { method: 'POST' });
            // Let the user know it's triggered
            alert("Execution Requested.");
            fetchJobs();
        } catch (err) {
            console.error('Failed to run job', err);
        }
    };

    return (
        <div className="flex flex-col h-full bg-mymolt-glass backdrop-blur-xl border border-mymolt-glassBorder rounded-3xl overflow-hidden shadow-2xl text-white font-sans col-span-1 md:col-span-2 relative">
            <div className="p-8 border-b border-mymolt-glassBorder bg-gradient-to-r from-mymolt-primary/10 to-transparent flex items-center justify-between">
                <div>
                    <h2 className="text-xl font-bold flex items-center gap-2">
                        <Calendar className="text-mymolt-primary" />
                        Task Scheduler
                    </h2>
                    <p className="text-xs text-white/50 mt-1 uppercase tracking-widest font-medium">
                        System Daemon & Cron Jobs
                    </p>
                </div>
                <button
                    onClick={() => setIsAdding(!isAdding)}
                    className="w-10 h-10 rounded-xl bg-mymolt-primary text-white flex items-center justify-center hover:bg-mymolt-primary/80 transition-colors shadow-[0_0_15px_rgba(8,145,178,0.3)]"
                >
                    <Plus size={20} className={isAdding ? 'rotate-45 transition-transform' : 'transition-transform'} />
                </button>
            </div>

            <AnimatePresence>
                {isAdding && (
                    <motion.div
                        initial={{ height: 0, opacity: 0 }}
                        animate={{ height: 'auto', opacity: 1 }}
                        exit={{ height: 0, opacity: 0 }}
                        className="border-b border-mymolt-glassBorder bg-black/20 overflow-hidden"
                    >
                        <form onSubmit={handleAdd} className="p-6 space-y-4">
                            <div>
                                <label className="block text-xs font-bold uppercase tracking-widest text-white/50 mb-2">Cron Expression</label>
                                <input
                                    type="text"
                                    value={newExpr}
                                    onChange={(e) => setNewExpr(e.target.value)}
                                    className="w-full bg-black/40 border border-white/10 rounded-xl px-4 py-3 text-sm focus:outline-none focus:border-mymolt-primary transition-colors font-mono"
                                    placeholder="0 * * * *"
                                />
                            </div>
                            <div>
                                <label className="block text-xs font-bold uppercase tracking-widest text-white/50 mb-2">Command</label>
                                <input
                                    type="text"
                                    value={newCmd}
                                    onChange={(e) => setNewCmd(e.target.value)}
                                    className="w-full bg-black/40 border border-white/10 rounded-xl px-4 py-3 text-sm focus:outline-none focus:border-mymolt-primary transition-colors font-mono"
                                    placeholder="agent -m 'Scan network'"
                                />
                            </div>
                            <div className="flex justify-end pt-2">
                                <button type="submit" className="px-6 py-2 bg-mymolt-primary hover:bg-mymolt-primary/80 text-white rounded-xl font-bold transition-all text-sm shadow-[0_0_15px_rgba(8,145,178,0.4)]">
                                    Schedule Task
                                </button>
                            </div>
                        </form>
                    </motion.div>
                )}
            </AnimatePresence>

            <div className="flex-1 overflow-y-auto p-4 space-y-3 min-h-[300px]">
                {loading ? (
                    <div className="h-full flex flex-col items-center justify-center text-white/30">
                        <Activity className="animate-pulse mb-2" size={32} />
                        <p className="text-sm font-medium">Syncing Schedules...</p>
                    </div>
                ) : jobs.length === 0 ? (
                    <div className="h-full flex flex-col items-center justify-center opacity-40 border border-dashed border-white/10 rounded-2xl m-4">
                        <Calendar size={48} className="mb-4" />
                        <p className="text-xl font-light">No tasks scheduled</p>
                    </div>
                ) : (
                    jobs.map(job => {
                        const nextRunStr = new Date(job.next_run).toLocaleString().replace(/Invalid Date/, 'Pending');
                        const lastRunStr = job.last_run ? new Date(job.last_run).toLocaleString() : 'Never';
                        const isOk = job.last_status === 'ok';
                        return (
                            <motion.div
                                initial={{ opacity: 0, y: 10 }}
                                animate={{ opacity: 1, y: 0 }}
                                key={job.id}
                                className="bg-white/5 border border-white/10 rounded-2xl p-4 hover:bg-white/10 transition-colors"
                            >
                                <div className="flex justify-between items-start mb-3">
                                    <div className="flex items-center gap-3">
                                        <div className="bg-black/40 px-3 py-1.5 rounded-lg border border-white/5 font-mono text-xs text-mymolt-primary">
                                            {job.expression}
                                        </div>
                                        {job.last_status && (
                                            <span className={`text-[10px] font-bold uppercase tracking-widest px-2 py-1 rounded-full border ${isOk ? 'bg-mymolt-success/10 text-mymolt-success border-mymolt-success/20' : 'bg-red-500/10 text-red-500 border-red-500/20'}`}>
                                                {job.last_status}
                                            </span>
                                        )}
                                    </div>
                                    <div className="flex items-center gap-2">
                                        <button onClick={() => handleRunNow(job.id)} className="p-2 hover:bg-white/10 rounded-xl transition-colors group relative" title="Run Now">
                                            <Play size={16} className="text-white/50 group-hover:text-mymolt-primary" />
                                        </button>
                                        <button onClick={() => handleDelete(job.id)} className="p-2 hover:bg-red-500/10 rounded-xl transition-colors group relative" title="Delete">
                                            <Trash2 size={16} className="text-white/50 group-hover:text-red-400" />
                                        </button>
                                    </div>
                                </div>

                                <div className="font-mono text-sm text-white/80 break-all mb-4 pl-1">
                                    <span className="text-white/30 mr-2">$</span>
                                    {job.command}
                                </div>

                                <div className="grid grid-cols-2 gap-4 text-xs">
                                    <div className="flex items-center gap-2 text-white/50">
                                        <Clock size={14} className="text-mymolt-primary/50" />
                                        <span>Next: <span className="text-white font-medium">{nextRunStr}</span></span>
                                    </div>
                                    <div className="flex items-center gap-2 text-white/30">
                                        <Activity size={14} className="text-white/20" />
                                        <span>Last: {lastRunStr}</span>
                                    </div>
                                </div>
                            </motion.div>
                        );
                    })
                )}
            </div>
        </div>
    );
};
