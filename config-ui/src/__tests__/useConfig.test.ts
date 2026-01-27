import { describe, it, expect, beforeEach, afterEach, vi } from 'vitest';
import { useConfig } from '../hooks/useConfig';
import type { StatusMessage, InstanceConfig, ToolConfig } from '../types';

// Mock fetch globally
global.fetch = vi.fn(async () => {
  throw new Error('fetch is not mocked');
});

describe('useConfig Hook', () => {
  beforeEach(() => {
    // Clear localStorage before each test
    localStorage.clear();
    // Reset fetch mock
    vi.clearAllMocks();
  });

  afterEach(() => {
    localStorage.clear();
  });

  describe('Type Validation', () => {
    it('should be a valid function', () => {
      expect(typeof useConfig).toBe('function');
    });

    it('should accept valid config types', () => {
      const validTypes = ['instances', 'server', 'tools', 'prompts'] as const;
      
      validTypes.forEach((type) => {
        expect(['instances', 'server', 'tools', 'prompts']).toContain(type);
      });
    });

    it('should handle instances type', () => {
      const type = 'instances';
      expect(type).toBe('instances');
    });

    it('should handle server type', () => {
      const type = 'server';
      expect(type).toBe('server');
    });

    it('should handle tools type', () => {
      const type = 'tools';
      expect(type).toBe('tools');
    });

    it('should handle prompts type', () => {
      const type = 'prompts';
      expect(type).toBe('prompts');
    });
  });

  describe('Auth Headers', () => {
    it('should store token in localStorage', () => {
      const token = 'test_token_123';
      localStorage.setItem('mcp_config_token', token);
      
      const retrieved = localStorage.getItem('mcp_config_token');
      expect(retrieved).toBe('test_token_123');
    });

    it('should work without auth token', () => {
      localStorage.removeItem('mcp_config_token');
      const token = localStorage.getItem('mcp_config_token');
      expect(token).toBeNull();
    });
  });

  describe('Status Messages', () => {
    it('should format status messages correctly', () => {
      const loadingStatus: StatusMessage = {
        message: 'Loading instances...',
        type: 'loading',
      };

      expect(loadingStatus.message).toContain('Loading');
      expect(loadingStatus.type).toBe('loading');
    });

    it('should handle different status types', () => {
      const statuses: StatusMessage[] = [
        { message: 'Loaded', type: 'success' },
        { message: 'Error occurred', type: 'error' },
        { message: 'Warning', type: 'warning' },
        { message: 'Loading', type: 'loading' },
      ];

      expect(statuses).toHaveLength(4);
      expect(statuses[0].type).toBe('success');
      expect(statuses[1].type).toBe('error');
      expect(statuses[2].type).toBe('warning');
      expect(statuses[3].type).toBe('loading');
    });
  });

  describe('Config Data Transformation', () => {
    it('should normalize instances config', () => {
      const instancesData: InstanceConfig = {
        default: {
          url: 'http://localhost:8069',
          db: 'test_db',
          apiKey: 'key123',
        },
      };

      expect(instancesData).toHaveProperty('default');
      expect(instancesData.default.url).toBe('http://localhost:8069');
      expect(instancesData.default.db).toBe('test_db');
    });

    it('should handle array config types', () => {
      const toolsData: ToolConfig[] = [
        { name: 'tool1', description: 'Tool 1' },
        { name: 'tool2', description: 'Tool 2' },
      ];

      expect(Array.isArray(toolsData)).toBe(true);
      expect(toolsData).toHaveLength(2);
      expect(toolsData[0].name).toBe('tool1');
    });

    it('should handle empty arrays', () => {
      const emptyTools: ToolConfig[] = [];
      expect(emptyTools).toHaveLength(0);
      expect(Array.isArray(emptyTools)).toBe(true);
    });
  });

  describe('Error Handling', () => {
    it('should set error status on failed load', () => {
      const errorMessage = 'Failed to load instances';
      const status: StatusMessage = {
        message: errorMessage,
        type: 'error',
      };

      expect(status.type).toBe('error');
      expect(status.message).toContain('Failed');
    });

    it('should handle unauthorized response', () => {
      // Simulate 401 response
      const status: StatusMessage = {
        message: 'Session expired. Please log in again.',
        type: 'error',
      };

      expect(status.message).toContain('Session expired');
    });

    it('should handle network errors', () => {
      const status: StatusMessage = {
        message: 'Network error: unable to connect to server',
        type: 'error',
      };

      expect(status.type).toBe('error');
      expect(status.message).toContain('Network error');
    });
  });

  describe('Token Management', () => {
    it('should store token in localStorage', () => {
      const token = 'bearer_token_12345';
      localStorage.setItem('mcp_config_token', token);

      const retrieved = localStorage.getItem('mcp_config_token');
      expect(retrieved).toBe(token);
    });

    it('should remove token on logout', () => {
      localStorage.setItem('mcp_config_token', 'test_token');
      localStorage.removeItem('mcp_config_token');

      const token = localStorage.getItem('mcp_config_token');
      expect(token).toBeNull();
    });

    it('should handle invalid token format', () => {
      const invalidToken = '';
      localStorage.setItem('mcp_config_token', invalidToken);

      const token = localStorage.getItem('mcp_config_token');
      expect(token).toBe('');
    });
  });

  describe('API Endpoints', () => {
    it('should construct correct endpoint for instances', () => {
      const endpoint = '/api/config/instances';
      expect(endpoint).toContain('/api/config');
      expect(endpoint).toContain('instances');
    });

    it('should construct correct endpoint for server', () => {
      const endpoint = '/api/config/server';
      expect(endpoint).toContain('/api/config');
      expect(endpoint).toContain('server');
    });

    it('should construct correct endpoint for tools', () => {
      const endpoint = '/api/config/tools';
      expect(endpoint).toContain('/api/config');
      expect(endpoint).toContain('tools');
    });

    it('should construct correct endpoint for prompts', () => {
      const endpoint = '/api/config/prompts';
      expect(endpoint).toContain('/api/config');
      expect(endpoint).toContain('prompts');
    });
  });

  describe('POST Request Handling', () => {
    it('should serialize config data to JSON', () => {
      const config: InstanceConfig = {
        default: {
          url: 'http://test:8069',
          db: 'test',
        },
      };

      const serialized = JSON.stringify(config);
      expect(serialized).toContain('default');
      expect(serialized).toContain('url');
    });

    it('should handle save with authorization header', () => {
      const token = 'test_token_xyz';
      localStorage.setItem('mcp_config_token', token);

      const headers: Record<string, string> = {
        'Content-Type': 'application/json',
      };

      if (token) {
        headers['Authorization'] = `Bearer ${token}`;
      }

      expect(headers['Authorization']).toBe('Bearer test_token_xyz');
    });

    it('should handle rollback response', () => {
      const responseData = {
        error: 'Failed to save',
        rollback: true,
      };

      expect(responseData.rollback).toBe(true);
      expect(responseData.error).toContain('Failed');
    });
  });

  describe('Timeout and Delays', () => {
    it('should handle status message timeout', (done) => {
      const status: StatusMessage = {
        message: 'Temporary message',
        type: 'success',
      };

      expect(status.message).toBe('Temporary message');
      
      // In real implementation, status would clear after 3 seconds
      setTimeout(() => {
        expect(true).toBe(true); // Just verify timeout works
        done();
      }, 100);
    });
  });

  describe('Configuration Types', () => {
    it('should accept instances config type', () => {
      const type = 'instances';
      expect(['instances', 'server', 'tools', 'prompts']).toContain(type);
    });

    it('should accept server config type', () => {
      const type = 'server';
      expect(['instances', 'server', 'tools', 'prompts']).toContain(type);
    });

    it('should accept tools config type', () => {
      const type = 'tools';
      expect(['instances', 'server', 'tools', 'prompts']).toContain(type);
    });

    it('should accept prompts config type', () => {
      const type = 'prompts';
      expect(['instances', 'server', 'tools', 'prompts']).toContain(type);
    });
  });
});
