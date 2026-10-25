import { useState, useCallback } from 'react';
import type { ConfigType, StatusMessage, InstanceConfig, ServerConfig, ToolConfig, PromptConfig } from '../types';

type ConfigData = InstanceConfig | ServerConfig | ToolConfig[] | PromptConfig[];

const TOKEN_STORAGE_KEY = 'mcp_config_token';

function getAuthHeaders(): HeadersInit {
  const headers: HeadersInit = {
    'Content-Type': 'application/json',
  };
  const token = localStorage.getItem(TOKEN_STORAGE_KEY);
  if (token) {
    headers['Authorization'] = `Bearer ${token}`;
  }
  return headers;
}

function handleUnauthorized(response: Response) {
  if (response.status === 401) {
    // Clear token and reload page to show login
    localStorage.removeItem(TOKEN_STORAGE_KEY);
    window.location.reload();
  }
}

export function useConfig(type: ConfigType) {
  const [status, setStatus] = useState<StatusMessage | null>(null);
  const [loading, setLoading] = useState(false);

  const load = useCallback(async (): Promise<ConfigData> => {
    setLoading(true);
    setStatus({ message: `Loading ${type}...`, type: 'loading' });

    try {
      const response = await fetch(`/api/config/${type}`, {
        headers: getAuthHeaders(),
      });

      if (response.status === 401) {
        handleUnauthorized(response);
        throw new Error('Session expired. Please log in again.');
      }

      if (!response.ok) {
        const errorData = await response.json().catch(() => ({ error: 'Unknown error' }));
        throw new Error(errorData.error || `Failed to load ${type}`);
      }

      const data = await response.json();

      let normalizedData: ConfigData;
      if (type === 'tools' || type === 'prompts') {
        normalizedData = Array.isArray(data) ? data : (data[type] || []);
      } else {
        normalizedData = data;
      }

      setStatus({ message: `${type.charAt(0).toUpperCase() + type.slice(1)} loaded successfully`, type: 'success' });
      setTimeout(() => setStatus(null), 3000);

      return normalizedData;
    } catch (error) {
      const message = error instanceof Error ? error.message : `Failed to load ${type}`;
      setStatus({ message, type: 'error' });
      throw error;
    } finally {
      setLoading(false);
    }
  }, [type]);

  const save = useCallback(async (config: ConfigData): Promise<void> => {
    setLoading(true);
    setStatus({ message: `Saving ${type}...`, type: 'loading' });

    try {
      const response = await fetch(`/api/config/${type}`, {
        method: 'POST',
        headers: getAuthHeaders(),
        body: JSON.stringify(config),
      });

      if (response.status === 401) {
        handleUnauthorized(response);
        throw new Error('Session expired. Please log in again.');
      }

      const responseData = await response.json().catch(() => ({}));

      if (!response.ok) {
        // Handle rollback notification
        if (responseData.rollback) {
          throw new Error(
            `${responseData.error || 'Failed to save'} (Previous config restored automatically)`
          );
        }
        throw new Error(responseData.error || `Failed to save ${type}`);
      }

      // Build success message with optional warning
      const successMessage = `${type.charAt(0).toUpperCase() + type.slice(1)} saved successfully. Hot reload applied.`;
      
      if (responseData.warning) {
        // Show warning alongside success
        setStatus({
          message: `${successMessage} Warning: ${responseData.warning}`,
          type: 'warning'
        });
        setTimeout(() => setStatus(null), 6000); // Keep warning visible longer
      } else {
        setStatus({
          message: successMessage,
          type: 'success'
        });
        setTimeout(() => setStatus(null), 3000);
      }
    } catch (error) {
      const message = error instanceof Error ? error.message : `Failed to save ${type}`;
      setStatus({ message, type: 'error' });
      throw error;
    } finally {
      setLoading(false);
    }
  }, [type]);

  return { load, save, status, loading };
}
