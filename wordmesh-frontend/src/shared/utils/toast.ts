/**
 * Toast 工具函数
 * 提供统一的错误、成功、警告、信息提示
 */

import { toast as sonnerToast } from 'sonner';
import { getUserFriendlyMessage } from './errorMessages';

/**
 * 显示错误提示
 * @param error 错误对象或错误消息
 * @param options 额外选项
 */
export function showErrorToast(
  error: unknown,
  options?: {
    title?: string;
    duration?: number;
  }
) {
  const message = getUserFriendlyMessage(error);
  const title = options?.title || '错误';

  sonnerToast.error(title, {
    description: message,
    duration: options?.duration || 5000,
  });
}

/**
 * 显示成功提示
 * @param message 成功消息
 * @param options 额外选项
 */
export function showSuccessToast(
  message: string,
  options?: {
    title?: string;
    duration?: number;
  }
) {
  const title = options?.title || '成功';

  sonnerToast.success(title, {
    description: message,
    duration: options?.duration || 3000,
  });
}

/**
 * 显示警告提示
 * @param message 警告消息
 * @param options 额外选项
 */
export function showWarningToast(
  message: string,
  options?: {
    title?: string;
    duration?: number;
  }
) {
  const title = options?.title || '警告';

  sonnerToast.warning(title, {
    description: message,
    duration: options?.duration || 4000,
  });
}

/**
 * 显示信息提示
 * @param message 信息消息
 * @param options 额外选项
 */
export function showInfoToast(
  message: string,
  options?: {
    title?: string;
    duration?: number;
  }
) {
  const title = options?.title || '提示';

  sonnerToast.info(title, {
    description: message,
    duration: options?.duration || 3000,
  });
}

/**
 * 显示加载提示
 * @param message 加载消息
 * @returns toast ID，可用于后续更新或关闭
 */
export function showLoadingToast(message: string): string | number {
  return sonnerToast.loading(message);
}

/**
 * 更新 toast（通常用于将 loading 转为 success/error）
 * @param toastId toast ID
 * @param type 新的类型
 * @param message 新的消息
 */
export function updateToast(
  toastId: string | number,
  type: 'success' | 'error' | 'warning' | 'info',
  message: string
) {
  sonnerToast[type](message, {
    id: toastId,
  });
}

/**
 * 关闭 toast
 * @param toastId toast ID
 */
export function dismissToast(toastId: string | number) {
  sonnerToast.dismiss(toastId);
}

