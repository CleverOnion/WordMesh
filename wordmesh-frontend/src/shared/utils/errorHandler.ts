/**
 * 错误处理工具
 */

import { ErrorCode, type ApiResponse, type FieldError } from '../types/api.types';

// 应用错误类
export class AppError extends Error {
  constructor(
    public code: number,
    message: string,
    public fieldErrors?: FieldError[],
    public traceId?: string
  ) {
    super(message);
    this.name = 'AppError';
  }
}

// 网络错误类
export class NetworkError extends Error {
  constructor(message: string, public status?: number) {
    super(message);
    this.name = 'NetworkError';
  }
}

// 解析 API 响应错误
export function parseApiError(response: ApiResponse): AppError {
  const { code, message, data, traceId } = response;

  // 参数校验错误，data 包含字段级错误
  if (code === ErrorCode.VALIDATION_ERROR && Array.isArray(data)) {
    return new AppError(code, message, data as FieldError[], traceId);
  }

  // 其他错误
  return new AppError(code, message, undefined, traceId);
}

// 检查响应是否成功
export function isSuccessResponse<T>(
  response: ApiResponse<T>
): response is { code: 2000; message: string; data: T; traceId?: string; timestamp?: number } {
  return response.code === ErrorCode.SUCCESS;
}

// 获取错误消息（用于 UI 显示）
export function getErrorMessage(error: unknown): string {
  if (error instanceof AppError) {
    return error.message;
  }
  if (error instanceof NetworkError) {
    return error.message;
  }
  if (error instanceof Error) {
    return error.message;
  }
  return '发生未知错误';
}

// 获取字段错误（用于表单验证）
export function getFieldErrors(error: unknown): FieldError[] | undefined {
  if (error instanceof AppError && error.fieldErrors) {
    return error.fieldErrors;
  }
  return undefined;
}

