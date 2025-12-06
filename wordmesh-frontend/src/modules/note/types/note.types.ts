/**
 * 笔记模块类型定义
 */

// 节点类型
export type NoteNodeType = 'word' | 'sense';

// 笔记节点标识
export interface NoteNode {
  type: NoteNodeType;
  id: number;
}

// 更新笔记请求
export interface UpdateNoteRequest {
  note: string | null;
}

