import { useState, useCallback } from 'react';
import type { ConfigType, StatusMessage } from '../types';

const API_BASE = '/api/config';

export function useConfig(type: ConfigType) {
  const [status, setStatus] = useState<StatusMessage | null>(null);
  const [loading, setLoading] = useState(false);

  const showStatus = useCallback((message: string, type: StatusMessage['type']) => {
    setStatus({ message, type });
    if (type === 'success') {
      setTimeout(() => setStatus(null), 3000);
    } else if (type === 'loading') {
      // Keep loading status visible
    }
  }, []);

  const load = useCallback(async () => {
    setLoading(true);
    showStatus('⏳ Loading...', 'loading');
    
    try {
      const response = await fetch(`${API_BASE}/${type}`);
      if (!response.ok) {
        throw new Error(`HTTP ${response.status}`);
      }

      let data = await response.json();
      
      // For tools and prompts, extract array from object if needed
      if ((type === 'tools' || type === 'prompts') && data && !Array.isArray(data)) {
        const key = type === 'tools' ? 'tools' : 'prompts';
        if (data[key] && Array.isArray(data[key])) {
          data = data[key];
        }
      }

      showStatus('✅ Loaded successfully', 'success');
      return data;
    } catch (error) {
      const message = error instanceof Error ? error.message : 'Unknown error';
      showStatus(`❌ Error: ${message}`, 'error');
      throw error;
    } finally {
      setLoading(false);
    }
  }, [type, showStatus]);

  const save = useCallback(async (config: any) => {
    setLoading(true);
    showStatus('⏳ Saving...', 'loading');

    try {
      // For tools and prompts, ensure we send array directly
      let payload = config;
      if (type === 'tools' || type === 'prompts') {
        if (!Array.isArray(config)) {
          const key = type === 'tools' ? 'tools' : 'prompts';
          if (config[key] && Array.isArray(config[key])) {
            payload = config[key];
          } else {
            throw new Error(`${type} must be an array`);
          }
        }
      }

      const response = await fetch(`${API_BASE}/${type}`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(payload),
      });

      if (!response.ok) {
        const error = await response.json().catch(() => ({ error: `HTTP ${response.status}` }));
        throw new Error(error.error || `HTTP ${response.status}`);
      }

      showStatus('✅ Saved successfully! Hot reload applied.', 'success');
      return true;
    } catch (error) {
      const message = error instanceof Error ? error.message : 'Unknown error';
      showStatus(`❌ Error: ${message}`, 'error');
      throw error;
    } finally {
      setLoading(false);
    }
  }, [type, showStatus]);

  return {
    load,
    save,
    status,
    loading,
  };
}
