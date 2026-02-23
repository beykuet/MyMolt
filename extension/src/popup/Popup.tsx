import React, { useState, useEffect } from 'react';
import { createRoot } from 'react-dom/client';
import type { MyMoltConfig, UserRole } from '../shared/types';
import { testConnection, saveConfig } from '../shared/api';

const Popup: React.FC = () => {
    const [config, setConfig] = useState<MyMoltConfig>({
        host: 'http://localhost:3000',
        token: '',
        role: 'Adult',
        userName: '',
        connected: false,
    });
    const [testing, setTesting] = useState(false);

    useEffect(() => {
        chrome.storage.local.get('mymolt_config', (result) => {
            if (result['mymolt_config']) setConfig(result['mymolt_config']);
        });
    }, []);

    const handleTest = async () => {
        setTesting(true);
        const ok = await testConnection();
        setConfig(prev => ({ ...prev, connected: ok }));
        setTesting(false);
    };

    const handleSave = async () => {
        await saveConfig(config);
        handleTest();
    };

    return (
        <div style={{ padding: 20, minHeight: 360 }}>
            {/* Logo + title */}
            <div style={{ display: 'flex', alignItems: 'center', gap: 10, marginBottom: 20 }}>
                <div style={{
                    width: 40, height: 40, borderRadius: 14,
                    background: 'linear-gradient(135deg, #ffcc00, #f59e0b)',
                    display: 'flex', alignItems: 'center', justifyContent: 'center',
                    fontWeight: 900, fontSize: 20, color: '#000',
                }}>M</div>
                <div>
                    <div style={{ fontWeight: 700, fontSize: 16 }}>MyMolt</div>
                    <div style={{
                        fontSize: 9, textTransform: 'uppercase', letterSpacing: '0.15em', fontWeight: 700,
                        color: config.connected ? '#34d399' : '#ef4444',
                    }}>
                        {config.connected ? '● Connected' : '○ Disconnected'}
                    </div>
                </div>
            </div>

            {/* Settings */}
            <div style={{ display: 'flex', flexDirection: 'column', gap: 12 }}>
                <div>
                    <label style={labelStyle}>MyMolt Host</label>
                    <input
                        value={config.host}
                        onChange={(e) => setConfig(prev => ({ ...prev, host: e.target.value }))}
                        placeholder="http://localhost:3000"
                        style={inputStyle}
                    />
                </div>
                <div>
                    <label style={labelStyle}>Token</label>
                    <input
                        type="password"
                        value={config.token}
                        onChange={(e) => setConfig(prev => ({ ...prev, token: e.target.value }))}
                        placeholder="Paste your MyMolt token"
                        style={inputStyle}
                    />
                </div>
                <div>
                    <label style={labelStyle}>Role</label>
                    <select
                        value={config.role}
                        onChange={(e) => setConfig(prev => ({ ...prev, role: e.target.value as UserRole }))}
                        style={{ ...inputStyle, appearance: 'none' }}
                    >
                        <option value="Root">Root</option>
                        <option value="Adult">Adult</option>
                        <option value="Senior">Senior</option>
                        <option value="Child">Child</option>
                    </select>
                </div>
                <div>
                    <label style={labelStyle}>Name</label>
                    <input
                        value={config.userName}
                        onChange={(e) => setConfig(prev => ({ ...prev, userName: e.target.value }))}
                        placeholder="Your name"
                        style={inputStyle}
                    />
                </div>
            </div>

            {/* Actions */}
            <div style={{ display: 'flex', gap: 8, marginTop: 16 }}>
                <button onClick={handleSave} style={primaryBtnStyle}>
                    Save & Connect
                </button>
                <button onClick={handleTest} disabled={testing} style={secondaryBtnStyle}>
                    {testing ? '...' : 'Test'}
                </button>
            </div>

            {/* Footer */}
            <div style={{
                marginTop: 20, paddingTop: 12, borderTop: '1px solid rgba(255,255,255,0.06)',
                fontSize: 9, textTransform: 'uppercase', letterSpacing: '0.12em',
                color: 'rgba(255,255,255,0.15)', textAlign: 'center', fontWeight: 700,
            }}>
                Your identity, your agent, your shield.
            </div>
        </div>
    );
};

const labelStyle: React.CSSProperties = {
    display: 'block',
    fontSize: 10,
    fontWeight: 700,
    textTransform: 'uppercase',
    letterSpacing: '0.1em',
    color: 'rgba(255,255,255,0.4)',
    marginBottom: 4,
};

const inputStyle: React.CSSProperties = {
    width: '100%',
    background: 'rgba(255,255,255,0.04)',
    border: '1px solid rgba(255,255,255,0.08)',
    borderRadius: 10,
    padding: '8px 12px',
    color: '#e2e8f0',
    fontSize: 13,
    outline: 'none',
};

const primaryBtnStyle: React.CSSProperties = {
    flex: 1,
    padding: '10px',
    borderRadius: 10,
    border: 'none',
    background: 'linear-gradient(135deg, #ffcc00, #f59e0b)',
    color: '#000',
    fontWeight: 700,
    fontSize: 13,
    cursor: 'pointer',
};

const secondaryBtnStyle: React.CSSProperties = {
    padding: '10px 16px',
    borderRadius: 10,
    border: '1px solid rgba(255,255,255,0.08)',
    background: 'rgba(255,255,255,0.04)',
    color: 'rgba(255,255,255,0.5)',
    fontWeight: 600,
    fontSize: 13,
    cursor: 'pointer',
};

const root = createRoot(document.getElementById('root')!);
root.render(<Popup />);
