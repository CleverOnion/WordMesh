/**
 * API 客户端封装
 * 提供统一的请求接口，处理认证、错误、超时等
 */

import {
  API_BASE_PATH,
  DEFAULT_HEADERS,
  REQUEST_TIMEOUT,
  TOKEN_STORAGE_KEY,
} from '../constants/api.constants';
import {
  type ApiResponse,
  type RequestConfig,
  ErrorCode,
} from '../types/api.types';
import { NetworkError, parseApiError, isSuccessResponse } from '../utils/errorHandler';

// Token 存储工具
class TokenStorage {
  private static getStorage(): Storage | null {
    if (typeof window === 'undefined') return null;
    return window.localStorage;
  }

  static getAccessToken(): string | null {
    const storage = this.getStorage();
    return storage?.getItem(TOKEN_STORAGE_KEY.ACCESS_TOKEN) ?? null;
  }

  static setAccessToken(token: string): void {
    const storage = this.getStorage();
    storage?.setItem(TOKEN_STORAGE_KEY.ACCESS_TOKEN, token);
  }

  static getRefreshToken(): string | null {
    const storage = this.getStorage();
    return storage?.getItem(TOKEN_STORAGE_KEY.REFRESH_TOKEN) ?? null;
  }

  static setRefreshToken(token: string | null): void {
    const storage = this.getStorage();
    if (token) {
      storage?.setItem(TOKEN_STORAGE_KEY.REFRESH_TOKEN, token);
    } else {
      storage?.removeItem(TOKEN_STORAGE_KEY.REFRESH_TOKEN);
    }
  }

  static clearTokens(): void {
    const storage = this.getStorage();
    storage?.removeItem(TOKEN_STORAGE_KEY.ACCESS_TOKEN);
    storage?.removeItem(TOKEN_STORAGE_KEY.REFRESH_TOKEN);
  }
}

// 构建查询字符串
function buildQueryString(params: Record<string, string | number | boolean>): string {
  const searchParams = new URLSearchParams();
  Object.entries(params).forEach(([key, value]) => {
    searchParams.append(key, String(value));
  });
  const query = searchParams.toString();
  return query ? `?${query}` : '';
}

// 构建请求 URL
function buildUrl(endpoint: string, params?: Record<string, string | number | boolean>): string {
  const baseUrl = endpoint.startsWith('http') ? endpoint : `${API_BASE_PATH}${endpoint}`;
  if (params) {
    return `${baseUrl}${buildQueryString(params)}`;
  }
  return baseUrl;
}

// 构建请求头
function buildHeaders(customHeaders?: Record<string, string>): HeadersInit {
  const headers = new Headers(DEFAULT_HEADERS);

  // 添加认证 token
  const accessToken = TokenStorage.getAccessToken();
  if (accessToken) {
    headers.set('Authorization', `Bearer ${accessToken}`);
  }

  // 合并自定义头部
  if (customHeaders) {
    Object.entries(customHeaders).forEach(([key, value]) => {
      headers.set(key, value);
    });
  }

  return headers;
}

// 处理响应
async function handleResponse<T>(response: Response): Promise<ApiResponse<T>> {
  // 检查 HTTP 状态码
  if (!response.ok) {
    throw new NetworkError(
      `HTTP ${response.status}: ${response.statusText}`,
      response.status
    );
  }

  // 解析 JSON
  const data = await response.json();

  // 检查业务码
  if (!isSuccessResponse(data)) {
    throw parseApiError(data);
  }

  return data;
}

// 请求函数
async function request<T>(
  endpoint: string,
  config: RequestConfig = {}
): Promise<ApiResponse<T>> {
  const {
    method = 'GET',
    headers: customHeaders,
    body,
    params,
    signal,
  } = config;

  const url = buildUrl(endpoint, params);
  const headers = buildHeaders(customHeaders);

  // 创建 AbortController 用于超时控制
  const controller = new AbortController();
  const timeoutId = setTimeout(() => controller.abort(), REQUEST_TIMEOUT);

  // 合并 signal
  const requestSignal = signal
    ? (() => {
        const mergedController = new AbortController();
        signal.addEventListener('abort', () => mergedController.abort());
        controller.signal.addEventListener('abort', () => mergedController.abort());
        return mergedController.signal;
      })()
    : controller.signal;

  try {
    const response = await fetch(url, {
      method,
      headers,
      body: body ? JSON.stringify(body) : undefined,
      signal: requestSignal,
    });

    clearTimeout(timeoutId);
    return await handleResponse<T>(response);
  } catch (error) {
    clearTimeout(timeoutId);

    if (error instanceof NetworkError || error instanceof Error) {
      throw error;
    }

    if (error instanceof DOMException && error.name === 'AbortError') {
      throw new NetworkError('请求超时');
    }

    throw new NetworkError('网络请求失败');
  }
}

// API 客户端
export const apiClient = {
  // GET 请求
  get<T>(endpoint: string, config?: Omit<RequestConfig, 'method' | 'body'>): Promise<ApiResponse<T>> {
    return request<T>(endpoint, { ...config, method: 'GET' });
  },

  // POST 请求
  post<T>(
    endpoint: string,
    body?: unknown,
    config?: Omit<RequestConfig, 'method' | 'body'>
  ): Promise<ApiResponse<T>> {
    return request<T>(endpoint, { ...config, method: 'POST', body });
  },

  // PUT 请求
  put<T>(
    endpoint: string,
    body?: unknown,
    config?: Omit<RequestConfig, 'method' | 'body'>
  ): Promise<ApiResponse<T>> {
    return request<T>(endpoint, { ...config, method: 'PUT', body });
  },

  // PATCH 请求
  patch<T>(
    endpoint: string,
    body?: unknown,
    config?: Omit<RequestConfig, 'method' | 'body'>
  ): Promise<ApiResponse<T>> {
    return request<T>(endpoint, { ...config, method: 'PATCH', body });
  },

  // DELETE 请求
  delete<T>(endpoint: string, config?: Omit<RequestConfig, 'method' | 'body'>): Promise<ApiResponse<T>> {
    return request<T>(endpoint, { ...config, method: 'DELETE' });
  },
};

// 导出 Token 存储工具
export { TokenStorage };

