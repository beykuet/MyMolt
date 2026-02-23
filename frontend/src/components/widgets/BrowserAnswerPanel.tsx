import React, { useState, useRef, useEffect } from 'react';
import { Bot, X, Mic, Send, Square, Loader2, ExternalLink, Image, Play, Volume2 } from 'lucide-react';
import { useAudio } from '../../hooks/useAudio';
import { apiClient } from '../../api/client';
import { motion, AnimatePresence } from 'framer-motion';
import type { UserRole } from '../../types';

interface Source {
    title: string;
    url: string;
    favicon?: string;
}

interface MediaItem {
    type: 'image' | 'video' | 'audio';
    url: string;
    caption: string;
}

interface AnswerMessage {
    id: string;
    sender: 'user' | 'agent';
    content: string;
    sources?: Source[];
    media?: MediaItem[];
    timestamp: Date;
}

interface BrowserAnswerPanelProps {
    isOpen: boolean;
    onClose: () => void;
    pageUrl: string;
    pageText: string;
    role: UserRole;
}

export const BrowserAnswerPanel: React.FC<BrowserAnswerPanelProps> = ({
    isOpen,
    onClose,
    pageUrl,
    pageText,
    role,
}) => {
    const [messages, setMessages] = useState<AnswerMessage[]>([]);
    const [input, setInput] = useState('');
    const [isThinking, setIsThinking] = useState(false);
    const { isRecording, startRecording, stopRecording } = useAudio();
    const scrollRef = useRef<HTMLDivElement>(null);

    useEffect(() => {
        if (scrollRef.current) {
            scrollRef.current.scrollTop = scrollRef.current.scrollHeight;
        }
    }, [messages]);

    // Reset conversation when page changes
    useEffect(() => {
        setMessages([]);
    }, [pageUrl]);

    const handleAsk = async (question: string) => {
        if (!question.trim()) return;

        const userMsg: AnswerMessage = {
            id: Math.random().toString(),
            sender: 'user',
            content: question,
            timestamp: new Date(),
        };
        setMessages(prev => [...prev, userMsg]);
        setInput('');
        setIsThinking(true);

        try {
            const res = await apiClient.fetch('/browse/ask', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({
                    url: pageUrl,
                    page_text: pageText.slice(0, 12000), // Limit context
                    question,
                    role,
                    conversation: messages.map(m => ({ role: m.sender, content: m.content })),
                }),
            });

            if (res.ok) {
                const data = await res.json();
                const agentMsg: AnswerMessage = {
                    id: Math.random().toString(),
                    sender: 'agent',
                    content: data.answer || 'I couldn\'t find an answer to that.',
                    sources: data.sources || [],
                    media: data.media || [],
                    timestamp: new Date(),
                };
                setMessages(prev => [...prev, agentMsg]);
            }
        } catch {
            setMessages(prev => [...prev, {
                id: Math.random().toString(),
                sender: 'agent',
                content: 'Sorry, I had trouble processing that. Please try again.',
                timestamp: new Date(),
            }]);
        }
        setIsThinking(false);
    };

    const handleMicClick = async () => {
        if (isRecording) {
            const base64 = await stopRecording();
            if (base64) {
                // Voice question — would send to STT then ask
                handleAsk('[Voice question — STT pending]');
            }
        } else {
            await startRecording();
        }
    };

    const isSeniorOrChild = role === 'Senior' || role === 'Child';

    return (
        <AnimatePresence>
            {isOpen && (
                <motion.div
                    initial={{ width: 0, opacity: 0 }}
                    animate={{ width: 420, opacity: 1 }}
                    exit={{ width: 0, opacity: 0 }}
                    transition={{ duration: 0.3, ease: [0.23, 1, 0.32, 1] }}
                    className="h-full border-l border-white/[0.06] bg-[#0f1520] flex flex-col overflow-hidden flex-shrink-0"
                >
                    {/* Header */}
                    <div className="flex items-center justify-between p-4 border-b border-white/[0.06] bg-gradient-to-r from-[#0f1520] to-[#151a28]">
                        <div className="flex items-center gap-2">
                            <div className="w-8 h-8 rounded-xl bg-gradient-to-br from-mymolt-yellow/30 to-amber-500/20 flex items-center justify-center border border-mymolt-yellow/10">
                                <Bot size={14} className="text-mymolt-yellow" />
                            </div>
                            <div>
                                <h3 className="text-sm font-bold text-white">MyMolt Answer</h3>
                                <p className="text-[9px] text-white/30 uppercase tracking-widest font-bold">
                                    {isSeniorOrChild ? 'Simple Mode' : 'Full Detail'}
                                </p>
                            </div>
                        </div>
                        <button onClick={onClose} className="p-1.5 rounded-lg hover:bg-white/[0.06] text-white/30 hover:text-white transition">
                            <X size={16} />
                        </button>
                    </div>

                    {/* Messages */}
                    <div ref={scrollRef} className="flex-1 overflow-y-auto p-4 space-y-4">
                        {messages.length === 0 && (
                            <div className="flex flex-col items-center justify-center h-full text-center space-y-3">
                                <div className="w-16 h-16 rounded-2xl bg-gradient-to-br from-mymolt-yellow/10 to-amber-500/5 flex items-center justify-center border border-mymolt-yellow/10">
                                    <Bot size={28} strokeWidth={1.2} className="text-mymolt-yellow/40" />
                                </div>
                                <p className={`font-medium text-white/40 ${isSeniorOrChild ? 'text-lg' : 'text-sm'}`}>
                                    Ask me about this page
                                </p>
                                <div className="flex flex-col gap-1.5 w-full max-w-xs">
                                    {['Summarize this page', 'Explain the main points', 'Is this information reliable?'].map(hint => (
                                        <button
                                            key={hint}
                                            onClick={() => handleAsk(hint)}
                                            className={`text-left px-3 py-2 rounded-xl bg-white/[0.03] border border-white/[0.05] text-white/30 hover:text-white/60 hover:bg-white/[0.06] transition ${isSeniorOrChild ? 'text-base' : 'text-xs'}`}
                                        >
                                            {hint}
                                        </button>
                                    ))}
                                </div>
                            </div>
                        )}

                        {messages.map((msg) => (
                            <div key={msg.id} className={`flex ${msg.sender === 'user' ? 'justify-end' : 'justify-start'}`}>
                                <div className={`
                                    max-w-[90%] px-3.5 py-2.5 rounded-2xl
                                    ${msg.sender === 'user'
                                        ? 'bg-gradient-to-br from-[#0055d4] to-[#003da0] text-white rounded-tr-sm'
                                        : 'bg-white/[0.05] text-white/85 rounded-tl-sm border border-white/[0.05]'}
                                `}>
                                    <p className={`leading-relaxed whitespace-pre-wrap ${isSeniorOrChild ? 'text-base' : 'text-[13px]'}`}>
                                        {msg.content}
                                    </p>

                                    {/* Media */}
                                    {msg.media && msg.media.length > 0 && (
                                        <div className="mt-3 space-y-2">
                                            {msg.media.map((m, i) => (
                                                <div key={i} className="rounded-xl overflow-hidden border border-white/[0.06] bg-black/20">
                                                    {m.type === 'image' && (
                                                        <div className="p-2">
                                                            <div className="flex items-center gap-2 text-xs text-white/40 mb-1">
                                                                <Image size={12} /> {m.caption}
                                                            </div>
                                                            <img src={m.url} alt={m.caption} className="rounded-lg w-full" />
                                                        </div>
                                                    )}
                                                    {m.type === 'video' && (
                                                        <div className="p-2">
                                                            <div className="flex items-center gap-2 text-xs text-white/40 mb-1">
                                                                <Play size={12} /> {m.caption}
                                                            </div>
                                                            <div className="aspect-video bg-black/40 rounded-lg flex items-center justify-center">
                                                                <Play size={32} className="text-white/30" />
                                                            </div>
                                                        </div>
                                                    )}
                                                    {m.type === 'audio' && (
                                                        <button className="flex items-center gap-2 px-3 py-2 w-full text-sm text-white/50 hover:text-white transition">
                                                            <Volume2 size={14} className="text-mymolt-yellow" /> {m.caption}
                                                        </button>
                                                    )}
                                                </div>
                                            ))}
                                        </div>
                                    )}

                                    {/* Sources */}
                                    {msg.sources && msg.sources.length > 0 && (
                                        <div className="mt-3 pt-2 border-t border-white/[0.04]">
                                            <p className="text-[9px] font-bold uppercase tracking-widest text-white/25 mb-1.5">Sources</p>
                                            <div className="space-y-1">
                                                {msg.sources.map((s, i) => (
                                                    <a
                                                        key={i}
                                                        href={s.url}
                                                        target="_blank"
                                                        rel="noopener noreferrer"
                                                        className="flex items-center gap-1.5 text-[11px] text-blue-400/70 hover:text-blue-300 transition"
                                                    >
                                                        <ExternalLink size={10} />
                                                        {s.title}
                                                    </a>
                                                ))}
                                            </div>
                                        </div>
                                    )}
                                </div>
                            </div>
                        ))}

                        {isThinking && (
                            <div className="flex justify-start">
                                <div className="bg-white/[0.05] border border-white/[0.05] px-4 py-2.5 rounded-2xl rounded-tl-sm flex gap-1.5 items-center">
                                    <motion.div animate={{ scale: [0.8, 1.2, 0.8] }} transition={{ repeat: Infinity, duration: 1.2 }} className="w-1.5 h-1.5 bg-mymolt-yellow/60 rounded-full" />
                                    <motion.div animate={{ scale: [0.8, 1.2, 0.8] }} transition={{ repeat: Infinity, duration: 1.2, delay: 0.15 }} className="w-1.5 h-1.5 bg-blue-400/60 rounded-full" />
                                    <motion.div animate={{ scale: [0.8, 1.2, 0.8] }} transition={{ repeat: Infinity, duration: 1.2, delay: 0.3 }} className="w-1.5 h-1.5 bg-mymolt-yellow/60 rounded-full" />
                                </div>
                            </div>
                        )}
                    </div>

                    {/* Input */}
                    <div className="p-3 border-t border-white/[0.06]">
                        <form
                            onSubmit={(e) => { e.preventDefault(); handleAsk(input); }}
                            className="relative"
                        >
                            <input
                                type="text"
                                value={input}
                                onChange={(e) => setInput(e.target.value)}
                                placeholder={isRecording ? '🎤 Listening...' : 'Ask about this page...'}
                                className={`w-full bg-white/[0.04] border rounded-xl py-2.5 pl-4 pr-20 text-white placeholder-white/20 focus:outline-none transition-all ${isSeniorOrChild ? 'text-base' : 'text-sm'} ${isRecording
                                        ? 'border-red-500/40 ring-1 ring-red-500/10'
                                        : 'border-white/[0.06] focus:border-mymolt-yellow/30'
                                    }`}
                                readOnly={isRecording}
                            />
                            <div className="absolute right-1 top-1 bottom-1 flex gap-1">
                                <button
                                    type="button"
                                    onClick={handleMicClick}
                                    className={`w-8 rounded-lg flex items-center justify-center transition ${isRecording
                                            ? 'bg-red-500 text-white animate-pulse'
                                            : 'bg-white/[0.04] text-white/25 hover:text-white/60 border border-white/[0.04]'
                                        }`}
                                >
                                    {isRecording ? <Square size={12} /> : <Mic size={12} />}
                                </button>
                                <button
                                    type="submit"
                                    disabled={!input.trim() || isThinking}
                                    className="w-8 bg-gradient-to-br from-mymolt-yellow to-amber-500 disabled:from-white/[0.04] disabled:to-white/[0.04] disabled:text-white/15 text-black rounded-lg flex items-center justify-center transition"
                                >
                                    {isThinking ? <Loader2 size={12} className="animate-spin text-white/30" /> : <Send size={12} />}
                                </button>
                            </div>
                        </form>
                    </div>
                </motion.div>
            )}
        </AnimatePresence>
    );
};
