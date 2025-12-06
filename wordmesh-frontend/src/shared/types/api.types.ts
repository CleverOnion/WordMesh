/**
 * API 统一响应类型定义
 * 所有接口均返回 HTTP 200 状态码，通过业务码 code 标识成功或错误
 */

// 字段级错误信息
export interface FieldError {
  field: string;
  message: string;
}

// 统一响应结构
export interface ApiResponse<T = unknown> {
  code: number;
  message: string;
  data: T | FieldError[] | null;
  traceId?: string;
  timestamp?: number;
}

// 成功响应（code = 2000）
export type SuccessResponse<T> = ApiResponse<T> & {
  code: 2000;
  data: T;
};

// 错误响应（code != 2000）
export type ErrorResponse = ApiResponse<null | FieldError[]> & {
  code: Exclude<number, 2000>;
};

// 业务错误码枚举
export enum ErrorCode {
  SUCCESS = 2000,
  BUSINESS_ERROR = 4000,
  VALIDATION_ERROR = 4001,
  AUTH_ERROR = 4010,
  INVALID_CREDENTIALS = 4011,
  TOKEN_EXPIRED = 4012,
  TOKEN_INVALID = 4013,
  REFRESH_DISABLED = 4014,
  INTERNAL_ERROR = 5000,
}

// HTTP 请求方法
export type HttpMethod = 'GET' | 'POST' | 'PUT' | 'PATCH' | 'DELETE';

// 请求配置
export interface RequestConfig {
  method?: HttpMethod;
  headers?: Record<string, string>;
  body?: unknown;
  params?: Record<string, string | number | boolean>;
  signal?: AbortSignal;
}

