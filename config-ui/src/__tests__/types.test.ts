import { describe, it, expect } from 'vitest';
import type {
  InstanceConfig,
  ToolConfig,
  PromptConfig,
  ServerConfig,
  ConfigType,
  StatusMessage,
  ToolCategory,
} from '../types';

describe('Type Definitions', () => {
  describe('InstanceConfig', () => {
    it('should create valid instance config', () => {
      const config: InstanceConfig = {
        default: {
          url: 'http://localhost:8069',
          db: 'test_db',
          apiKey: 'test_key',
          username: 'admin',
          password: 'password',
          version: 19,
        },
      };

      expect(config.default.url).toBe('http://localhost:8069');
      expect(config.default.db).toBe('test_db');
      expect(config.default.apiKey).toBe('test_key');
      expect(config.default.version).toBe(19);
    });

    it('should handle optional fields in instance config', () => {
      const config: InstanceConfig = {
        prod: {
          url: 'https://prod.example.com',
          db: 'production',
        },
      };

      expect(config.prod.apiKey).toBeUndefined();
      expect(config.prod.username).toBeUndefined();
      expect(config.prod.version).toBeUndefined();
    });

    it('should support multiple instances', () => {
      const config: InstanceConfig = {
        dev: {
          url: 'http://dev.local:8069',
          db: 'dev',
          apiKey: 'dev_key',
        },
        staging: {
          url: 'http://staging.local:8069',
          db: 'staging',
          apiKey: 'staging_key',
        },
        production: {
          url: 'https://prod.example.com',
          db: 'production',
          apiKey: 'prod_key',
        },
      };

      expect(Object.keys(config)).toHaveLength(3);
      expect(config.dev.url).toBe('http://dev.local:8069');
      expect(config.staging.url).toBe('http://staging.local:8069');
      expect(config.production.url).toBe('https://prod.example.com');
    });
  });

  describe('ToolConfig', () => {
    it('should create valid tool config', () => {
      const tool: ToolConfig = {
        name: 'search_partners',
        description: 'Search for partners',
        guards: {
          requiresEnvTrue: 'ENABLE_PARTNER_SEARCH',
        },
      };

      expect(tool.name).toBe('search_partners');
      expect(tool.description).toBe('Search for partners');
      expect(tool.guards?.requiresEnvTrue).toBe('ENABLE_PARTNER_SEARCH');
    });

    it('should allow additional properties in tool config', () => {
      const tool: ToolConfig = {
        name: 'create_order',
        description: 'Create sales order',
        category: 'sales',
        enabled: true,
        version: '1.0',
        customField: 'custom_value',
      };

      expect(tool.name).toBe('create_order');
      expect((tool as any).category).toBe('sales');
      expect((tool as any).customField).toBe('custom_value');
    });

    it('should handle tools without description', () => {
      const tool: ToolConfig = {
        name: 'simple_tool',
      };

      expect(tool.name).toBe('simple_tool');
      expect(tool.description).toBeUndefined();
    });
  });

  describe('PromptConfig', () => {
    it('should create valid prompt config', () => {
      const prompt: PromptConfig = {
        name: 'odoo_models',
        description: 'Common Odoo models',
        content: 'List of Odoo models...',
      };

      expect(prompt.name).toBe('odoo_models');
      expect(prompt.description).toBe('Common Odoo models');
      expect(prompt.content.length).toBeGreaterThan(0);
    });

    it('should support markdown content in prompts', () => {
      const prompt: PromptConfig = {
        name: 'markdown_guide',
        description: 'Markdown guide',
        content: `# Heading
        ## Sub Heading
        - List item 1
        - List item 2`,
      };

      expect(prompt.content).toContain('# Heading');
      expect(prompt.content).toContain('## Sub Heading');
    });

    it('should allow additional properties', () => {
      const prompt: PromptConfig = {
        name: 'advanced',
        description: 'Advanced prompt',
        content: 'Content...',
        tags: ['sales', 'crm'],
        version: 2,
      };

      expect((prompt as any).tags).toEqual(['sales', 'crm']);
      expect((prompt as any).version).toBe(2);
    });
  });

  describe('ServerConfig', () => {
    it('should create valid server config', () => {
      const config: ServerConfig = {
        serverName: 'odoo-mcp-server',
        instructions: 'Server instructions...',
        protocolVersionDefault: '2024-11-05',
      };

      expect(config.serverName).toBe('odoo-mcp-server');
      expect(config.instructions).toBe('Server instructions...');
      expect(config.protocolVersionDefault).toBe('2024-11-05');
    });

    it('should allow additional server config properties', () => {
      const config: ServerConfig = {
        serverName: 'test-server',
        logLevel: 'debug',
        enableCache: true,
        maxConnections: 100,
      };

      expect((config as any).logLevel).toBe('debug');
      expect((config as any).enableCache).toBe(true);
      expect((config as any).maxConnections).toBe(100);
    });

    it('should handle minimal server config', () => {
      const config: ServerConfig = {
        serverName: 'minimal',
      };

      expect(config.serverName).toBe('minimal');
      expect(config.instructions).toBeUndefined();
      expect(config.protocolVersionDefault).toBeUndefined();
    });
  });

  describe('StatusMessage', () => {
    it('should create loading status', () => {
      const status: StatusMessage = {
        message: 'Loading data...',
        type: 'loading',
      };

      expect(status.type).toBe('loading');
      expect(status.message).toContain('Loading');
    });

    it('should create success status', () => {
      const status: StatusMessage = {
        message: 'Config saved successfully',
        type: 'success',
      };

      expect(status.type).toBe('success');
      expect(status.message).toContain('successfully');
    });

    it('should create error status', () => {
      const status: StatusMessage = {
        message: 'Failed to save config: Invalid JSON',
        type: 'error',
      };

      expect(status.type).toBe('error');
      expect(status.message).toContain('Failed');
    });

    it('should create warning status', () => {
      const status: StatusMessage = {
        message: 'Deprecated field detected',
        type: 'warning',
      };

      expect(status.type).toBe('warning');
      expect(status.message).toContain('Deprecated');
    });
  });

  describe('ToolCategory', () => {
    it('should create valid tool category', () => {
      const category: ToolCategory = {
        name: 'Sales',
        description: 'Sales operations',
        icon: 'shopping-cart',
        color: 'blue',
        bgColor: 'bg-blue-100',
        tools: ['search_orders', 'create_order', 'confirm_order'],
        envVar: 'ENABLE_SALES_TOOLS',
      };

      expect(category.name).toBe('Sales');
      expect(category.tools).toHaveLength(3);
      expect(category.tools).toContain('create_order');
    });

    it('should create inventory category', () => {
      const category: ToolCategory = {
        name: 'Inventory',
        description: 'Stock and warehouse management',
        icon: 'package',
        color: 'green',
        bgColor: 'bg-green-100',
        tools: ['search_moves', 'transfer_stock'],
        envVar: 'ENABLE_INVENTORY_TOOLS',
      };

      expect(category.name).toBe('Inventory');
      expect(category.color).toBe('green');
    });

    it('should handle empty tool list', () => {
      const category: ToolCategory = {
        name: 'Accounting',
        description: 'Financial operations',
        icon: 'dollar-sign',
        color: 'purple',
        bgColor: 'bg-purple-100',
        tools: [],
        envVar: 'ENABLE_ACCOUNTING_TOOLS',
      };

      expect(category.tools).toHaveLength(0);
      expect(category.envVar).toBe('ENABLE_ACCOUNTING_TOOLS');
    });
  });

  describe('ConfigType', () => {
    it('should accept valid config types', () => {
      const types: ConfigType[] = ['instances', 'server', 'tools', 'prompts'];

      expect(types).toHaveLength(4);
      expect(types).toContain('instances');
      expect(types).toContain('server');
      expect(types).toContain('tools');
      expect(types).toContain('prompts');
    });
  });

  describe('Mixed Type Tests', () => {
    it('should handle complex config structure', () => {
      const instances: InstanceConfig = {
        default: {
          url: 'http://localhost:8069',
          db: 'dev',
          apiKey: 'key123',
        },
      };

      const tools: ToolConfig[] = [
        { name: 'tool1', description: 'Tool 1' },
        { name: 'tool2', description: 'Tool 2' },
      ];

      const prompts: PromptConfig[] = [
        { name: 'prompt1', description: 'Prompt 1', content: 'Content 1' },
      ];

      const server: ServerConfig = {
        serverName: 'test-server',
      };

      expect(Object.keys(instances)).toHaveLength(1);
      expect(tools).toHaveLength(2);
      expect(prompts).toHaveLength(1);
      expect(server.serverName).toBe('test-server');
    });

    it('should handle array configuration types', () => {
      const toolsList: ToolConfig[] = Array.from({ length: 5 }, (_, i) => ({
        name: `tool_${i + 1}`,
        description: `Tool ${i + 1}`,
      }));

      const promptsList: PromptConfig[] = Array.from({ length: 3 }, (_, i) => ({
        name: `prompt_${i + 1}`,
        description: `Prompt ${i + 1}`,
        content: `Content for prompt ${i + 1}`,
      }));

      expect(toolsList).toHaveLength(5);
      expect(promptsList).toHaveLength(3);
      expect(toolsList[0].name).toBe('tool_1');
      expect(promptsList[2].name).toBe('prompt_3');
    });
  });
});
