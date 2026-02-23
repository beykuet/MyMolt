const BASE_URL = '/api';

export const apiClient = {
    async fetch(path: string, options: RequestInit = {}) {
        const user = JSON.parse(localStorage.getItem('mymolt_user') || 'null');
        const token = user?.token;

        const headers = new Headers(options.headers);
        if (token) {
            headers.set('Authorization', `Bearer ${token}`);
        }

        if (options.body && !(options.body instanceof FormData) && !headers.has('Content-Type')) {
            headers.set('Content-Type', 'application/json');
        }

        const response = await fetch(`${BASE_URL}${path}`, {
            ...options,
            headers,
        });

        if (response.status === 401) {
            localStorage.removeItem('mymolt_user');
            window.location.href = '/';
        }

        return response;
    },

    async get<T>(path: string): Promise<T> {
        const res = await this.fetch(path);
        if (!res.ok) throw new Error(`GET ${path} failed: ${res.statusText}`);
        return res.json();
    },

    async post<T>(path: string, body: any): Promise<T> {
        const res = await this.fetch(path, {
            method: 'POST',
            body: JSON.stringify(body)
        });
        if (!res.ok) throw new Error(`POST ${path} failed: ${res.statusText}`);
        // Handle empty bodies (DELETE often returns 204 or empty JSON)
        try {
            return await res.json();
        } catch {
            return {} as T;
        }
    },

    async delete<T>(path: string): Promise<T> {
        const res = await this.fetch(path, {
            method: 'DELETE',
        });
        if (!res.ok) throw new Error(`DELETE ${path} failed: ${res.statusText}`);
        try {
            return await res.json();
        } catch {
            return {} as T;
        }
    }
};
