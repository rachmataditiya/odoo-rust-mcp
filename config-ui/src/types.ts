export interface InstanceConfig {
  [key: string]: {
    url: string;
    db: string;
    apiKey?: string;
    username?: string;
    password?: string;
    version?: number;
  };
}

export interface ToolConfig {
  name: string;
  description?: string;
  guards?: {
    requiresEnvTrue?: string;
  };
  [key: string]: any;
}

export interface PromptConfig {
  name: string;
  description?: string;
  content: string;
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
  type: 'loading' | 'success' | 'error';
}

export interface ToolCategory {
  name: string;
  description: string;
  icon: string;
  color: string;
  bgColor: string;
  tools: string[];
  envVar: string;
}
