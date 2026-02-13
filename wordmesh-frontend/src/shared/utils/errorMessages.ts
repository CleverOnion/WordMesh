/**
 * 错误消息映射
 * 将后端错误码转换为用户友好的中文提示
 */

import { ErrorCode } from '../types/api.types';

/**
 * 错误码到用户友好消息的映射
 */
export const ERROR_MESSAGES: Record<number, string> = {
  // 业务错误
  [ErrorCode.BUSINESS_ERROR]: '操作失败，请稍后重试',
  [ErrorCode.VALIDATION_ERROR]: '输入信息有误，请检查后重试',

  // 认证错误
  [ErrorCode.AUTH_ERROR]: '认证失败，请重新登录',
  [ErrorCode.INVALID_CREDENTIALS]: '用户名或密码错误',
  [ErrorCode.TOKEN_EXPIRED]: '登录已过期，请重新登录',
  [ErrorCode.TOKEN_INVALID]: '登录状态无效，请重新登录',
  [ErrorCode.REFRESH_DISABLED]: '刷新令牌已禁用，请重新登录',

  // 服务器错误
  [ErrorCode.INTERNAL_ERROR]: '服务器内部错误，请稍后重试',

  // Word 相关错误（根据后端定义）
  4201: '该单词已存在于您的网络中',
  4202: '该单词不在您的网络中',
  4203: '该词义已存在',
  4204: '主词义冲突',

  // Link 相关错误
  4301: '该关联已存在',
  4302: '不能创建单词与自身的关联',
  4303: '关联目标不存在',
  4304: '关联类型无效',
  4305: '关联数量已达上限',
};

/**
 * 根据错误码获取用户友好的错误消息
 * @param code 错误码
 * @param defaultMessage 默认消息（如果错误码未映射）
 * @returns 用户友好的错误消息
 */
export function getErrorMessageByCode(code: number, defaultMessage?: string): string {
  return ERROR_MESSAGES[code] || defaultMessage || '操作失败，请稍后重试';
}

/**
 * 根据错误对象获取用户友好的错误消息
 * @param error 错误对象（可以是 AppError、ApiResponse 或普通 Error）
 * @returns 用户友好的错误消息
 */
export function getUserFriendlyMessage(error: unknown): string {
  // 如果是 AppError，使用 code 查找消息
  if (error && typeof error === 'object' && 'code' in error) {
    const code = (error as { code: number }).code;
    const message = (error as { message?: string }).message;
    return getErrorMessageByCode(code, message);
  }

  // 如果是普通 Error，返回消息
  if (error instanceof Error) {
    return error.message;
  }

  return '操作失败，请稍后重试';
}






