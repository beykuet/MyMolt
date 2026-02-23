import React, { useState, useEffect, useRef } from 'react';
import { Send, Bot, Sparkles, Brain, Loader2, Mic, Square, Shield, Zap } from 'lucide-react';
import { useAudio } from '../../hooks/useAudio';
import { motion, AnimatePresence } from 'framer-motion';

interface Message {
    id: string;
    sender: 'user' | 'agent' | 'thought' | 'system';
    content: string;
    timestamp: Date;
}

export const ChatWidget: React.FC = () => {
    const [messages, setMessages] = useState<Message[]>([]);
    const [input, setInput] = useState('');
    const [isTyping, setIsTyping] = useState(false);
    const { isRecording, startRecording, stopRecording } = useAudio();
    const scrollRef = useRef<HTMLDivElement>(null);
    const socketRef = useRef<WebSocket | null>(null);

    useEffect(() => {
        const token = localStorage.getItem('root_token');
        const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
        const socket = new WebSocket(`${protocol}//${window.location.host}/api/ws?token=${token}`);

        socket.onmessage = (event) => {
            const msg = JSON.parse(event.data);
            if (msg.type === 'text') {
                setMessages(prev => [...prev, {
                    id: Math.random().toString(),
                    sender: msg.payload.sender as 'user' | 'agent' | 'system',
                    content: msg.payload.content,
                    timestamp: new Date()
                }]);
                setIsTyping(false);
            } else if (msg.type === 'thought') {
                setMessages(prev => [...prev, {
                    id: Math.random().toString(),
                    sender: 'thought',
                    content: msg.payload.content,
                    timestamp: new Date()
                }]);
            }
        };

        socketRef.current = socket;
        return () => socket.close();
    }, []);

    useEffect(() => {
        if (scrollRef.current) {
            scrollRef.current.scrollTop = scrollRef.current.scrollHeight;
        }
    }, [messages]);

    const handleSend = () => {
        if (!input.trim() || !socketRef.current) return;

        const userMsg: Message = {
            id: Math.random().toString(),
            sender: 'user',
            content: input,
            timestamp: new Date()
        };

        setMessages(prev => [...prev, userMsg]);
        socketRef.current.send(JSON.stringify({
            type: 'text',
            payload: { content: input, sender: 'user' }
        }));
        setInput('');
        setIsTyping(true);
    };

    const handleMicClick = async () => {
        if (isRecording) {
            const base64 = await stopRecording();
            if (base64 && socketRef.current) {
                socketRef.current.send(JSON.stringify({
                    type: 'audio',
                    payload: { data: base64, format: 'webm' }
                }));
                setIsTyping(true);
            }
        } else {
            await startRecording();
        }
    };

    return (
        <div className="flex flex-col h-full max-w-4xl mx-auto rounded-3xl overflow-hidden shadow-2xl relative">
            {/* Ambient glow */}
            <div className="absolute inset-0 bg-gradient-to-br from-[#0a1628] via-[#0f1d35] to-[#1a1040] -z-10" />
            <div className="absolute top-0 left-1/4 w-96 h-96 bg-blue-500/[0.07] rounded-full blur-3xl -z-10" />
            <div className="absolute bottom-0 right-1/4 w-80 h-80 bg-yellow-500/[0.05] rounded-full blur-3xl -z-10" />

            {/* Header */}
            <div className="p-5 border-b border-white/[0.08] flex items-center justify-between bg-gradient-to-r from-[#0044aa]/80 via-[#0055d4]/60 to-[#3b2db0]/40 backdrop-blur-xl">
                <div className="flex items-center gap-3">
                    <div className="relative">
                        <div className="w-11 h-11 rounded-2xl bg-gradient-to-br from-mymolt-yellow to-amber-500 flex items-center justify-center shadow-lg shadow-yellow-500/30">
                            <Bot className="text-black" size={22} />
                        </div>
                        <span className="absolute -bottom-0.5 -right-0.5 w-3 h-3 rounded-full bg-emerald-400 border-2 border-[#0055d4] shadow-[0_0_6px_rgba(52,211,153,0.6)]" />
                    </div>
                    <div>
                        <h2 className="text-lg font-bold text-white tracking-tight">MyMolt</h2>
                        <div className="flex items-center gap-2">
                            <span className="text-[10px] font-bold uppercase tracking-widest text-blue-300/60">Sovereign Chat</span>
                            <span className="text-white/20">·</span>
                            <span className="text-[10px] text-emerald-400/70 font-medium flex items-center gap-1">
                                <Zap size={8} /> Online
                            </span>
                        </div>
                    </div>
                </div>
                <div className="flex gap-1">
                    <button className="p-2.5 hover:bg-white/[0.08] rounded-xl transition-all text-blue-300/50 hover:text-mymolt-yellow">
                        <Brain size={18} />
                    </button>
                    <button className="p-2.5 hover:bg-white/[0.08] rounded-xl transition-all text-blue-300/50 hover:text-mymolt-yellow">
                        <Sparkles size={18} />
                    </button>
                    <button className="p-2.5 hover:bg-white/[0.08] rounded-xl transition-all text-blue-300/50 hover:text-emerald-400">
                        <Shield size={18} />
                    </button>
                </div>
            </div>

            {/* Messages */}
            <div ref={scrollRef} className="flex-1 overflow-y-auto p-6 space-y-5 scroll-smooth">
                <AnimatePresence>
                    {messages.length === 0 && (
                        <motion.div
                            initial={{ opacity: 0, y: 20 }}
                            animate={{ opacity: 1, y: 0 }}
                            transition={{ duration: 0.6, ease: [0.23, 1, 0.32, 1] }}
                            className="flex flex-col items-center justify-center h-full text-center space-y-6"
                        >
                            <div className="relative">
                                <div className="w-24 h-24 rounded-3xl bg-gradient-to-br from-mymolt-yellow/20 to-amber-500/10 flex items-center justify-center border border-mymolt-yellow/10">
                                    <Bot size={40} strokeWidth={1.2} className="text-mymolt-yellow/60" />
                                </div>
                                <div className="absolute -inset-8 bg-gradient-to-r from-blue-500/10 via-transparent to-yellow-500/10 rounded-full blur-2xl -z-10 animate-pulse" />
                            </div>
                            <div>
                                <p className="text-xl font-bold text-white/70 mb-1">How can I help you?</p>
                                <p className="text-sm text-white/25">Your identity, your agent, your shield.</p>
                            </div>
                            <div className="flex gap-2 flex-wrap justify-center max-w-md">
                                {['📅 "What\'s on my calendar?"', '📝 "Write a note"', '🔍 "Search my contacts"', '🛡️ "Security status"'].map((hint) => (
                                    <button
                                        key={hint}
                                        onClick={() => setInput(hint.replace(/^[^"]+\"/, '').replace('"', ''))}
                                        className="text-xs px-3 py-1.5 rounded-full bg-white/[0.04] border border-white/[0.06] text-white/30 hover:text-white/60 hover:bg-white/[0.08] hover:border-white/[0.1] transition-all"
                                    >
                                        {hint}
                                    </button>
                                ))}
                            </div>
                        </motion.div>
                    )}
                    {messages.map((msg) => (
                        <motion.div
                            key={msg.id}
                            initial={{ opacity: 0, y: 12, scale: 0.97 }}
                            animate={{ opacity: 1, y: 0, scale: 1 }}
                            transition={{ duration: 0.3, ease: [0.23, 1, 0.32, 1] }}
                            className={`flex ${msg.sender === 'user' ? 'justify-end' : 'justify-start'}`}
                        >
                            {msg.sender === 'agent' && (
                                <div className="w-8 h-8 rounded-xl bg-gradient-to-br from-mymolt-yellow/30 to-amber-600/20 flex items-center justify-center mr-2 mt-1 flex-shrink-0 border border-mymolt-yellow/10">
                                    <Bot size={14} className="text-mymolt-yellow" />
                                </div>
                            )}
                            {msg.sender === 'thought' && (
                                <div className="w-8 h-8 rounded-xl bg-purple-500/10 flex items-center justify-center mr-2 mt-1 flex-shrink-0 border border-purple-400/10">
                                    <Brain size={14} className="text-purple-400/70" />
                                </div>
                            )}
                            <div className={`
                                max-w-[75%] px-4 py-3 rounded-2xl shadow-sm
                                ${msg.sender === 'user'
                                    ? 'bg-gradient-to-br from-[#0055d4] to-[#003da0] text-white rounded-tr-sm shadow-blue-500/20 shadow-lg'
                                    : msg.sender === 'thought'
                                        ? 'bg-purple-500/[0.08] border border-purple-400/10 text-purple-200/80 text-sm italic rounded-tl-sm'
                                        : 'bg-white/[0.06] text-white/90 rounded-tl-sm border border-white/[0.06]'}
                            `}>
                                <div className="flex items-center gap-2 mb-0.5">
                                    {msg.sender === 'thought' && <span className="text-[9px] font-bold uppercase tracking-widest text-purple-400/50">Thinking</span>}
                                    {msg.sender === 'agent' && <span className="text-[9px] font-bold uppercase tracking-widest text-mymolt-yellow/50">MyMolt</span>}
                                    {msg.sender === 'user' && <span className="text-[9px] font-bold uppercase tracking-widest text-blue-200/40">You</span>}
                                </div>
                                <p className="leading-relaxed whitespace-pre-wrap text-[14px]">{msg.content}</p>
                                <div className="mt-1.5 text-[9px] opacity-25 text-right font-mono">
                                    {msg.timestamp.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })}
                                </div>
                            </div>
                        </motion.div>
                    ))}
                    {isTyping && (
                        <motion.div initial={{ opacity: 0, y: 8 }} animate={{ opacity: 1, y: 0 }} className="flex justify-start">
                            <div className="w-8 h-8 rounded-xl bg-gradient-to-br from-mymolt-yellow/30 to-amber-600/20 flex items-center justify-center mr-2 flex-shrink-0 border border-mymolt-yellow/10">
                                <Bot size={14} className="text-mymolt-yellow" />
                            </div>
                            <div className="bg-white/[0.06] border border-white/[0.06] px-5 py-3 rounded-2xl rounded-tl-sm flex gap-1.5 items-center">
                                <motion.div animate={{ scale: [0.8, 1.2, 0.8] }} transition={{ repeat: Infinity, duration: 1.2 }} className="w-2 h-2 bg-mymolt-yellow/60 rounded-full" />
                                <motion.div animate={{ scale: [0.8, 1.2, 0.8] }} transition={{ repeat: Infinity, duration: 1.2, delay: 0.15 }} className="w-2 h-2 bg-blue-400/60 rounded-full" />
                                <motion.div animate={{ scale: [0.8, 1.2, 0.8] }} transition={{ repeat: Infinity, duration: 1.2, delay: 0.3 }} className="w-2 h-2 bg-mymolt-yellow/60 rounded-full" />
                            </div>
                        </motion.div>
                    )}
                </AnimatePresence>
            </div>

            {/* Input bar */}
            <div className="p-4 bg-gradient-to-t from-[#0a1225]/80 to-transparent backdrop-blur-xl border-t border-white/[0.06]">
                <form onSubmit={(e) => { e.preventDefault(); handleSend(); }} className="relative">
                    <input
                        type="text"
                        value={input}
                        onChange={(e) => setInput(e.target.value)}
                        placeholder={isRecording ? "🎤 Listening..." : "Message MyMolt..."}
                        className={`w-full bg-white/[0.04] border rounded-2xl py-3.5 pl-5 pr-28 text-white placeholder-white/20 focus:outline-none transition-all text-[14px] ${isRecording
                                ? 'border-red-500/40 ring-2 ring-red-500/10 bg-red-500/[0.03]'
                                : 'border-white/[0.08] focus:border-mymolt-yellow/30 focus:ring-2 focus:ring-mymolt-yellow/10 focus:bg-white/[0.06]'
                            }`}
                        readOnly={isRecording}
                    />
                    <div className="absolute right-1.5 top-1.5 bottom-1.5 flex gap-1.5">
                        <button
                            type="button"
                            onClick={handleMicClick}
                            className={`w-10 rounded-xl transition-all flex items-center justify-center ${isRecording
                                ? 'bg-red-500 hover:bg-red-600 text-white shadow-lg shadow-red-500/30 animate-pulse'
                                : 'bg-white/[0.06] hover:bg-white/[0.1] text-white/30 hover:text-white border border-white/[0.06]'
                                }`}
                        >
                            {isRecording ? <Square size={16} /> : <Mic size={16} />}
                        </button>
                        <button
                            type="submit"
                            disabled={!input.trim() || isTyping || isRecording}
                            className="w-10 bg-gradient-to-br from-mymolt-yellow to-amber-500 hover:from-yellow-400 hover:to-amber-400 disabled:from-white/[0.04] disabled:to-white/[0.04] disabled:text-white/15 disabled:border disabled:border-white/[0.06] text-black font-semibold rounded-xl transition-all shadow-lg shadow-yellow-500/20 disabled:shadow-none flex items-center justify-center"
                        >
                            {isTyping ? <Loader2 size={16} className="animate-spin text-white/30" /> : <Send size={16} />}
                        </button>
                    </div>
                </form>
                <div className="flex items-center justify-center gap-3 mt-2.5">
                    <span className="flex items-center gap-1 text-[9px] text-white/15 uppercase tracking-[0.15em] font-bold">
                        <Shield size={8} /> E2E Encrypted
                    </span>
                    <span className="w-0.5 h-0.5 rounded-full bg-white/10" />
                    <span className="text-[9px] text-white/15 uppercase tracking-[0.15em] font-bold">Sovereign Runtime</span>
                </div>
            </div>
        </div>
    );
};
