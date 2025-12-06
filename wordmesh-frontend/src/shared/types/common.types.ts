/**
 * 通用类型定义
 */

// 分页参数
export interface PaginationParams {
  page?: number;
  pageSize?: number;
}

// 分页响应
export interface PaginatedResponse<T> {
  items: T[];
  total: number;
  page: number;
  pageSize: number;
  totalPages: number;
}

// 时间戳类型（ISO 8601 字符串）
export type Timestamp = string;

// ID 类型
export type ID = number | string;

// 可选字段工具类型
export type Optional<T, K extends keyof T> = Omit<T, K> & Partial<Pick<T, K>>;

// 必需字段工具类型
export type Required<T, K extends keyof T> = T & { [P in K]-?: T[P] };

