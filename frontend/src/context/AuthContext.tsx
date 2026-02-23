import React, { createContext, useContext, useState, useCallback } from 'react';
import type { User, UserRole } from '../types';

interface AuthContextType {
    user: User | null;
    login: (role: UserRole, token?: string) => Promise<boolean>;
    logout: () => void;
}

const AuthContext = createContext<AuthContextType | undefined>(undefined);

export const AuthProvider: React.FC<{ children: React.ReactNode }> = ({ children }) => {
    const [user, setUser] = useState<User | null>(() => {
        // Persist session
        const saved = localStorage.getItem('mymolt_user');
        return saved ? JSON.parse(saved) : null;
    });

    const login = useCallback(async (role: UserRole, token: string = '') => {
        try {
            let finalToken = token;

            // 1. If Root and token is a 6-digit code, exchange it for a real token
            if (role === 'Root' && /^\d{6}$/.test(token)) {
                console.log("🔐 Authenticating Root via pairing code...");
                const res = await fetch('/pair', {
                    method: 'POST',
                    headers: { 'X-Pairing-Code': token }
                });

                if (!res.ok) {
                    const error = await res.json();
                    throw new Error(error.error || 'Pairing failed');
                }

                const data = await res.json();
                finalToken = data.token;
                console.log("🔐 Pairing successful, permanent token received.");
            }

            // 2. Verify we have a working session by hitting a protected endpoint
            // (Only for Root/non-empty tokens to ensure we don't 'flicker' in)
            if (finalToken) {
                const res = await fetch('/api/system/status', {
                    headers: { 'Authorization': `Bearer ${finalToken}` }
                });
                if (res.status === 401) throw new Error('Invalid or expired session');
            }

            const newUser = { role, username: role, token: finalToken };
            setUser(newUser);
            localStorage.setItem('mymolt_user', JSON.stringify(newUser));
            return true;
        } catch (e) {
            console.error("Login failed:", e);
            alert(e instanceof Error ? e.message : "Login failed");
            return false;
        }
    }, []);

    const logout = useCallback(() => {
        setUser(null);
        localStorage.removeItem('mymolt_user');
    }, []);

    return (
        <AuthContext.Provider value={{ user, login, logout }}>
            {children}
        </AuthContext.Provider>
    );
};

export const useAuth = () => {
    const context = useContext(AuthContext);
    if (!context) throw new Error("useAuth must be used within AuthProvider");
    return context;
};
