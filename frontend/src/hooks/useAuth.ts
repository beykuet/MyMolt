import { useState, useEffect } from 'react';

const API_URL = import.meta.env.VITE_API_URL || ''; // Empty string means same origin

export function useAuth() {
    const [token, setToken] = useState<string | null>(localStorage.getItem('mymolt_token'));
    const [isPaired, setIsPaired] = useState<boolean>(false);
    const [loading, setLoading] = useState<boolean>(true);

    useEffect(() => {
        checkHealth();
        const interval = setInterval(checkHealth, 5000);
        return () => clearInterval(interval);
    }, [token]);

    const checkHealth = async () => {
        try {
            const headers: HeadersInit = {};
            if (token) headers['Authorization'] = `Bearer ${token}`;

            const res = await fetch(`${API_URL}/health`, { headers });

            if (res.ok) {
                const data = await res.json();
                // If pairing is disabled on server, we are "paired"
                if (data.pairing_enabled === false) {
                    setIsPaired(true);
                } else {
                    setIsPaired(!!token);
                }
            } else {
                setIsPaired(false);
            }
            setLoading(false);
        } catch (e) {
            console.error("Health check failed", e);
            setLoading(false);
        }
    };

    const login = async (code: string) => {
        try {
            const res = await fetch(`${API_URL}/pair`, {
                method: 'POST',
                headers: { 'X-Pairing-Code': code }
            });

            if (res.ok) {
                const data = await res.json();
                setToken(data.token);
                localStorage.setItem('mymolt_token', data.token);
                setIsPaired(true);
                return { success: true };
            } else {
                const err = await res.json();
                return { success: false, error: err.error || "Pairing failed" };
            }
        } catch (e) {
            return { success: false, error: "Network error" };
        }
    };

    const logout = () => {
        setToken(null);
        localStorage.removeItem('mymolt_token');
        setIsPaired(false);
    };

    return { token, isPaired, login, logout, loading };
}
