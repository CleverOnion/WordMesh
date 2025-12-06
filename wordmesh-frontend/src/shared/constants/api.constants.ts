/**
 * API 相关常量
 */

// API 基础 URL
export const API_BASE_URL =
  process.env.NEXT_PUBLIC_API_BASE_URL || 'http://127.0.0.1:8080';

// API 版本前缀
export const API_VERSION_PREFIX = '/api/v1';

// API 完整基础路径
export const API_BASE_PATH = `${API_BASE_URL}${API_VERSION_PREFIX}`;

// 请求超时时间（毫秒）
export const REQUEST_TIMEOUT = 30000;

// Token 存储键名
export const TOKEN_STORAGE_KEY = {
  ACCESS_TOKEN: 'wordmesh_access_token',
  REFRESH_TOKEN: 'wordmesh_refresh_token',
} as const;

// 默认请求头
export const DEFAULT_HEADERS = {
  'Content-Type': 'application/json',
} as const;

