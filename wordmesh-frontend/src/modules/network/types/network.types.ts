/**
 * 网络可视化模块类型定义
 */

// 节点类型
export type NodeType = 'word' | 'sense';

// 网络节点
export interface NetworkNode {
  id: string;
  type: NodeType;
  label: string;
  wordId?: number;
  senseId?: number;
  userWordId?: number;
  x?: number;
  y?: number;
  fx?: number | null;
  fy?: number | null;
  size?: number;
  color?: string;
}

// 网络边（关联）
export interface NetworkEdge {
  id: string;
  source: string;
  target: string;
  type: 'word-word' | 'sense-word';
  kind: string;
  label?: string;
  color?: string;
}

// 网络图数据
export interface NetworkGraph {
  nodes: NetworkNode[];
  edges: NetworkEdge[];
}

// 布局算法类型
export type LayoutAlgorithm = 'force' | 'hierarchical' | 'circular';

// 视图控制
export interface ViewControls {
  zoom: number;
  panX: number;
  panY: number;
}

