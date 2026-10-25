export interface InstanceItem {
  url: string;
  db: string;
  apiKey?: string;
  username?: string;
  password?: string;
  version?: number | string;
  readOnly?: boolean;
}

export interface InstanceConfig {
  url?: string;
  db?: string;
  apiKey?: string;
  username?: string;
  password?: string;
  version?: number | string;
  readOnly?: boolean;
  [key: string]: any;
}

export interface ToolConfig {
  name: string;
  description?: string;
  inputSchema?: {
    type?: string;
    properties?: Record<string, any>;
    required?: string[];
    [key: string]: any;
  };
  guards?: {
    requiresEnvTrue?: string;
    [key: string]: any;
  };
  [key: string]: any;
}

export interface PromptConfig {
  name: string;
  description?: string;
  content: string;
  arguments?: Array<{
    name: string;
    description?: string;
    required?: boolean;
  }>;
  [key: string]: any;
}

export interface ServerConfig {
  serverName?: string;
  instructions?: string;
  protocolVersionDefault?: string;
  [key: string]: any;
}

export type ConfigType = 'instances' | 'server' | 'tools' | 'prompts';

export interface StatusMessage {
  message: string;
  type: 'loading' | 'success' | 'error' | 'warning' | 'info';
}

export interface ToolCategory {
  name: string;
  description?: string;
  icon?: string;
  color?: string;
  bgColor?: string;
  tools: string[];
  envVar?: string;
}

export type TabType = 'instances' | 'server' | 'tools' | 'prompts' | 'security';

export interface AuthStatus {
  authenticated: boolean;
  auth_enabled: boolean;
  username: string | null;
}

export interface McpAuthStatus {
  enabled: boolean;
  token_configured: boolean;
}

export interface AuthContextType {
  isAuthenticated: boolean;
  authEnabled: boolean;
  username: string | null;
  token: string | null;
  loading: boolean;
  login: (username: string, password: string) => Promise<void>;
  logout: () => Promise<void>;
  checkAuth: () => Promise<void>;
}
