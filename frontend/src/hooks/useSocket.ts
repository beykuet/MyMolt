import { useEffect, useRef, useState, useCallback } from 'react';
import type { WsMessage } from '../types';

const WS_URL = `${window.location.protocol === 'https:' ? 'wss:' : 'ws:'}//${window.location.host}/api/ws`;

export function useSocket() {
    const [isConnected, setIsConnected] = useState(false);
    const socketRef = useRef<WebSocket | null>(null);
    const [messages, setMessages] = useState<WsMessage[]>([]);

    useEffect(() => {
        const token = localStorage.getItem('mymolt_token');
        if (!token) return;

        const ws = new WebSocket(`${WS_URL}?token=${token}`);
        socketRef.current = ws;

        ws.onopen = () => {
            console.log('Connected to WebSocket');
            setIsConnected(true);
        };

        ws.onmessage = (event) => {
            try {
                const msg: WsMessage = JSON.parse(event.data);
                setMessages(prev => [...prev, msg]);
            } catch (e) {
                console.error('Failed to parse WS message', e);
            }
        };

        ws.onclose = () => {
            console.log('Disconnected from WebSocket');
            setIsConnected(false);
        };

        return () => {
            ws.close();
        };
    }, []);

    const sendMessage = useCallback((msg: WsMessage) => {
        if (socketRef.current?.readyState === WebSocket.OPEN) {
            socketRef.current.send(JSON.stringify(msg));
        }
    }, []);

    return { isConnected, messages, sendMessage };
}
