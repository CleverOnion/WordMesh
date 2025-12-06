/**
 * 应用相关常量
 */

// 应用名称
export const APP_NAME = 'WordMesh';

// 应用版本
export const APP_VERSION = '0.1.0';

// 分页默认值
export const PAGINATION_DEFAULTS = {
  PAGE: 1,
  PAGE_SIZE: 20,
  MAX_PAGE_SIZE: 100,
} as const;

// 关联类型常量
export const ASSOCIATION_TYPES = {
  // 语义关联（基于义项）
  SEMANTIC: {
    SYNONYM: 'synonym', // 同义词
    ANTONYM: 'antonym', // 反义词
  },
  // 非语义关联（基于单词）
  NON_SEMANTIC: {
    SIMILAR_SPELLING: 'similar_spelling', // 形近词
    ROOT_PREFIX: 'root_prefix', // 词根/词缀
  },
} as const;

// 节点类型
export const NODE_TYPES = {
  WORD: 'word',
  SENSE: 'sense',
  ASSOCIATION: 'association',
} as const;

