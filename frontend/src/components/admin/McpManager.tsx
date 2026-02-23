import React, { useState, useEffect } from 'react';
import { Plug2, Plus, Trash2, RefreshCw, CheckCircle2, XCircle, ShieldCheck, Terminal, ChevronDown, ChevronUp, Play, Activity } from 'lucide-react';
import { apiClient } from '../../api/client';

interface McpServer {
    name: string;
    transport: 'stdio' | 'sse' | 'streamable-http';
    command?: string;
    url?: string;
    status: 'connected' | 'disconnected' | 'error';
    tools: string[];
    sigil_gated: boolean;
}

export const McpManager: React.FC = () => {
    const [servers, setServers] = useState<McpServer[]>([]);
    const [isAdding, setIsAdding] = useState(false);
    const [newServer, setNewServer] = useState<{
        name: string;
        transport: 'stdio' | 'sse' | 'streamable-http';
        command: string;
        sigil_gated: boolean;
    }>({
        name: '',
        transport: 'stdio',
        command: '',
        sigil_gated: true,
    });

    const [expandedServer, setExpandedServer] = useState<string | null>(null);
    const [testPayloads, setTestPayloads] = useState<Record<string, string>>({});
    const [testResults, setTestResults] = useState<Record<string, string>>({});
    const [testingTool, setTestingTool] = useState<string | null>(null);

    useEffect(() => {
        apiClient.fetch('/mcp/servers')
            .then(r => r.ok ? r.json() : [])
            .then(data => { if (Array.isArray(data)) setServers(data); })
            .catch(() => { });
    }, []);

    const handleTestTool = async (toolName: string) => {
        setTestingTool(toolName);
        setTestResults(prev => ({ ...prev, [toolName]: 'Connecting...' }));
        try {
            const rawPayload = testPayloads[toolName] || '{}';
            let parsedPayload = {};
            try { parsedPayload = JSON.parse(rawPayload); } catch {
                setTestResults(prev => ({ ...prev, [toolName]: 'Invalid JSON payload format.\n\nUse e.g.: { "arg1": "value" }' }));
                setTestingTool(null);
                return;
            }

            const res = await apiClient.fetch(`/mcp/tools/${encodeURIComponent(toolName)}/call`, {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ payload: parsedPayload }),
            });

            const data = await res.json();
            setTestResults(prev => ({ ...prev, [toolName]: JSON.stringify(data, null, 2) }));
        } catch (err: any) {
            setTestResults(prev => ({ ...prev, [toolName]: `Error: ${err.message}` }));
        } finally {
            setTestingTool(null);
        }
    };

    const handleAdd = async () => {
        if (!newServer.name.trim() || !newServer.command.trim()) return;
        try {
            const res = await apiClient.fetch('/mcp/servers', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify(newServer),
            });
            if (res.ok) {
                const added = await res.json();
                setServers(prev => [...prev, added]);
                setNewServer({ name: '', transport: 'stdio', command: '', sigil_gated: true });
                setIsAdding(false);
            }
        } catch { }
    };

    const handleRemove = async (name: string) => {
        try {
            const res = await apiClient.fetch(`/mcp/servers/${encodeURIComponent(name)}`, { method: 'DELETE' });
            if (res.ok) setServers(prev => prev.filter(s => s.name !== name));
        } catch { }
    };

    const statusIcon = (status: string) => {
        switch (status) {
            case 'connected': return <CheckCircle2 size={14} className="text-emerald-400" />;
            case 'error': return <XCircle size={14} className="text-red-400" />;
            default: return <XCircle size={14} className="text-white/20" />;
        }
    };

    const totalTools = servers.reduce((sum, s) => sum + s.tools.length, 0);

    return (
        <div>
            <h2 className="text-2xl font-bold tracking-tight mb-2">MCP Servers</h2>
            <p className="text-white/40 text-sm mb-8">
                Model Context Protocol connections — {servers.length} server{servers.length !== 1 ? 's' : ''}, {totalTools} tools
            </p>

            {/* Connected Servers */}
            <div className="bg-[#1e2435] rounded-2xl border border-white/[0.06] p-6 mb-6">
                <div className="flex items-center justify-between mb-5">
                    <div className="flex items-center gap-2">
                        <Plug2 size={16} className="text-mymolt-yellow" />
                        <h3 className="text-sm font-bold uppercase tracking-wider text-white/60">Connected Servers</h3>
                    </div>
                    <button
                        onClick={() => setIsAdding(true)}
                        className="flex items-center gap-1 text-xs font-medium text-mymolt-yellow hover:text-yellow-300 transition"
                    >
                        <Plus size={12} /> Add Server
                    </button>
                </div>

                <div className="space-y-3">
                    {servers.map((server) => (
                        <div
                            key={server.name}
                            className="p-4 rounded-xl bg-black/20 border border-white/[0.04] hover:border-white/[0.08] transition group"
                        >
                            <div className="flex items-center justify-between mb-2">
                                <div className="flex items-center gap-3">
                                    {statusIcon(server.status)}
                                    <span className="font-semibold text-sm">{server.name}</span>
                                    <span className="text-[10px] font-bold uppercase tracking-widest text-white/20 px-2 py-0.5 bg-white/[0.04] rounded-full">
                                        {server.transport}
                                    </span>
                                    {server.sigil_gated && (
                                        <span className="text-[10px] font-bold uppercase tracking-widest text-emerald-400/60 flex items-center gap-1">
                                            <ShieldCheck size={10} /> SIGIL
                                        </span>
                                    )}
                                </div>
                                <div className="flex items-center gap-2 opacity-0 group-hover:opacity-100 transition">
                                    <button className="p-1.5 hover:bg-white/[0.06] rounded-lg text-white/30 hover:text-white transition">
                                        <RefreshCw size={14} />
                                    </button>
                                    <button
                                        onClick={() => handleRemove(server.name)}
                                        className="p-1.5 hover:bg-red-500/10 rounded-lg text-white/30 hover:text-red-400 transition"
                                    >
                                        <Trash2 size={14} />
                                    </button>
                                </div>
                            </div>
                            {server.tools.length > 0 && (
                                <div className="flex flex-wrap gap-1.5 mt-2">
                                    {server.tools.map(tool => (
                                        <span
                                            key={tool}
                                            className="text-[10px] font-mono px-2 py-0.5 bg-white/[0.04] rounded text-white/40"
                                        >
                                            {tool}
                                        </span>
                                    ))}
                                </div>
                            )}

                            {/* Dropdown Details Button */}
                            <div className="flex justify-center mt-2 border-t border-white/[0.02] pt-2 relative opacity-50 hover:opacity-100 transition-opacity">
                                <button
                                    onClick={() => setExpandedServer(expandedServer === server.name ? null : server.name)}
                                    className="p-1 px-8 rounded-full hover:bg-white/[0.05] flex items-center gap-2 text-xs text-white/30 hover:text-white"
                                >
                                    {expandedServer === server.name ? <ChevronUp size={14} /> : <ChevronDown size={14} />}
                                    <span className="uppercase tracking-widest font-bold text-[9px]">{expandedServer === server.name ? 'Collapse' : 'Inspect Tools'}</span>
                                </button>
                            </div>

                            {/* Drill-down UI */}
                            {expandedServer === server.name && (
                                <div className="mt-4 pt-4 border-t border-mymolt-primary/20 space-y-4 animate-in slide-in-from-top-2 duration-300">
                                    <div className="flex items-center gap-2 mb-2">
                                        <Activity size={14} className="text-mymolt-primary" />
                                        <span className="text-xs font-bold uppercase tracking-widest text-white/50">Manual Tool Test Harness</span>
                                    </div>
                                    {server.tools.map(tool => (
                                        <div key={tool} className="bg-black/30 border border-white/[0.05] rounded-xl p-4 flex flex-col gap-3">
                                            <div className="font-mono text-xs text-mymolt-yellow font-bold">{tool}</div>
                                            <textarea
                                                className="w-full bg-black/40 border border-white/10 rounded-lg p-3 text-xs font-mono text-white/80 focus:outline-none focus:border-mymolt-primary min-h-[60px]"
                                                placeholder='{ "param_name": "example_value" }'
                                                value={testPayloads[tool] || ''}
                                                onChange={e => setTestPayloads(prev => ({ ...prev, [tool]: e.target.value }))}
                                            />
                                            <div className="flex justify-between items-center">
                                                <div className="flex-1 pr-4">
                                                    {testResults[tool] && (
                                                        <div className="bg-[#0f111a] border border-mymolt-primary/30 p-3 rounded-lg text-[10px] font-mono whitespace-pre-wrap max-h-[150px] overflow-auto text-emerald-400">
                                                            {testResults[tool]}
                                                        </div>
                                                    )}
                                                </div>
                                                <button
                                                    onClick={() => handleTestTool(tool)}
                                                    disabled={testingTool === tool}
                                                    className={`px-4 py-2 rounded-xl text-xs font-bold tracking-tight uppercase flex items-center gap-2 shrink-0 transition-all ${testingTool === tool ? 'bg-white/10 text-white/40 cursor-wait' : 'bg-mymolt-primary/20 hover:bg-mymolt-primary hover:text-black text-mymolt-primary border border-mymolt-primary/30 hover:border-mymolt-primary shadow-[0_0_10px_rgba(8,145,178,0.2)]'}`}
                                                >
                                                    <Play size={12} className={testingTool === tool ? "animate-pulse" : ""} />
                                                    {testingTool === tool ? 'Testing...' : 'Test Execution'}
                                                </button>
                                            </div>
                                        </div>
                                    ))}
                                    {server.tools.length === 0 && (
                                        <div className="text-center text-xs text-white/30 py-4 italic">No tools exported by this server.</div>
                                    )}
                                </div>
                            )}
                        </div>
                    ))}

                    {servers.length === 0 && !isAdding && (
                        <div className="text-center py-12 text-white/20">
                            <Plug2 size={40} className="mx-auto mb-3 opacity-50" />
                            <p className="font-medium text-sm">No MCP servers connected</p>
                            <p className="text-xs mt-1">Add servers to extend MyMolt with external tools</p>
                        </div>
                    )}
                </div>

                {/* Add form */}
                {isAdding && (
                    <div className="mt-4 p-4 rounded-xl bg-black/20 border border-mymolt-yellow/20 space-y-3">
                        <div className="grid grid-cols-2 gap-3">
                            <input
                                value={newServer.name}
                                onChange={(e) => setNewServer(prev => ({ ...prev, name: e.target.value }))}
                                placeholder="Server name (e.g. brave-search)"
                                autoFocus
                                className="bg-black/30 border border-white/10 rounded-xl px-3 py-2 text-sm focus:border-mymolt-yellow/50 outline-none transition"
                            />
                            <select
                                value={newServer.transport}
                                onChange={(e) => setNewServer(prev => ({ ...prev, transport: e.target.value as 'stdio' | 'sse' | 'streamable-http' }))}
                                className="bg-black/30 border border-white/10 rounded-xl px-3 py-2 text-sm focus:border-mymolt-yellow/50 outline-none appearance-none"
                            >
                                <option value="stdio">stdio</option>
                                <option value="sse">SSE</option>
                                <option value="streamable-http">Streamable HTTP</option>
                            </select>
                        </div>
                        <div className="flex items-center gap-3">
                            <Terminal size={14} className="text-white/30 flex-shrink-0" />
                            <input
                                value={newServer.command}
                                onChange={(e) => setNewServer(prev => ({ ...prev, command: e.target.value }))}
                                placeholder="npx -y @modelcontextprotocol/server-brave-search"
                                className="flex-1 bg-black/30 border border-white/10 rounded-xl px-3 py-2 text-sm font-mono focus:border-mymolt-yellow/50 outline-none transition"
                            />
                        </div>
                        <div className="flex items-center justify-between">
                            <label className="flex items-center gap-2 text-sm text-white/50 cursor-pointer">
                                <input
                                    type="checkbox"
                                    checked={newServer.sigil_gated}
                                    onChange={(e) => setNewServer(prev => ({ ...prev, sigil_gated: e.target.checked }))}
                                    className="rounded accent-mymolt-yellow"
                                />
                                SIGIL Gatekeeper (scan all in/out)
                            </label>
                            <div className="flex gap-2">
                                <button
                                    onClick={() => setIsAdding(false)}
                                    className="px-3 py-1.5 text-sm text-white/40 hover:text-white transition"
                                >
                                    Cancel
                                </button>
                                <button
                                    onClick={handleAdd}
                                    disabled={!newServer.name.trim() || !newServer.command.trim()}
                                    className="px-4 py-1.5 bg-mymolt-yellow text-black rounded-xl text-sm font-bold hover:bg-yellow-400 disabled:opacity-30 transition"
                                >
                                    Add
                                </button>
                            </div>
                        </div>
                    </div>
                )}
            </div>

            {/* SIGIL Gatekeeper Info */}
            <div className="bg-[#1e2435] rounded-2xl border border-white/[0.06] p-6">
                <div className="flex items-center gap-2 mb-4">
                    <ShieldCheck size={16} className="text-emerald-400" />
                    <h3 className="text-sm font-bold uppercase tracking-wider text-white/60">SIGIL Gatekeeper</h3>
                </div>
                <div className="space-y-2 text-sm text-white/50">
                    <p className="flex items-center gap-2"><CheckCircle2 size={14} className="text-emerald-400/60" /> Sensitivity scan on all tool arguments</p>
                    <p className="flex items-center gap-2"><CheckCircle2 size={14} className="text-emerald-400/60" /> Sensitivity scan on all tool results</p>
                    <p className="flex items-center gap-2"><CheckCircle2 size={14} className="text-emerald-400/60" /> Audit logging (DelegationCrossing events)</p>
                    <p className="flex items-center gap-2"><CheckCircle2 size={14} className="text-emerald-400/60" /> Trust level enforcement per-tool</p>
                </div>
            </div>
        </div>
    );
};
