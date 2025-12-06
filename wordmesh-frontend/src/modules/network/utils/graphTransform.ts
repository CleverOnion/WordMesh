/**
 * 图形数据转换工具
 * 将单词、义项、关联数据转换为网络图数据
 */

import type { UserWordAggregate } from '@/modules/word';
import type { WordLinkRecord, SenseWordLinkRecord } from '@/modules/association';
import type { NetworkNode, NetworkEdge, NetworkGraph } from '../types/network.types';

/**
 * 将单词聚合数据转换为网络节点
 */
export function wordToNode(word: UserWordAggregate): NetworkNode {
  return {
    id: `word-${word.user_word.id}`,
    type: 'word',
    label: word.word.text,
    wordId: word.word.id,
    userWordId: word.user_word.id || undefined,
    size: 20,
    color: '#3b82f6',
    expanded: false,
  };
}

/**
 * 将义项转换为网络节点
 */
export function senseToNode(
  senseId: number,
  senseText: string,
  wordId: number,
  userWordId: number
): NetworkNode {
  return {
    id: `sense-${senseId}`,
    type: 'sense',
    label: senseText,
    wordId,
    senseId,
    userWordId,
    size: 15,
    color: '#10b981',
  };
}

/**
 * 创建义的根节点
 */
export function createSenseGroupNode(userWordId: number): NetworkNode {
  return {
    id: `sense-group-${userWordId}`,
    type: 'sense-group',
    label: '义项',
    userWordId,
    size: 15,
    color: '#10b981',
    expanded: false,
    parentId: `word-${userWordId}`,
  };
}

/**
 * 创建词-词关联根节点
 */
export function createWordLinkGroupNode(
  userWordId: number,
  kind: 'similar_form' | 'root_affix'
): NetworkNode {
  const labels: Record<'similar_form' | 'root_affix', string> = {
    similar_form: '形近',
    root_affix: '词根',
  };
  return {
    id: `word-link-group-${userWordId}-${kind}`,
    type: 'word-link-group',
    label: labels[kind],
    userWordId,
    size: 15,
    color: '#8b5cf6',
    expanded: false,
    parentId: `word-${userWordId}`,
    linkKind: kind,
  };
}

/**
 * 将词-词关联转换为网络边
 */
export function wordLinkToEdge(link: WordLinkRecord): NetworkEdge {
  const isSimilarForm = link.kind === 'similar_form';
  return {
    id: link.link_id,
    source: `word-${link.word_a_id}`,
    target: `word-${link.word_b_id}`,
    type: 'word-word',
    kind: link.kind,
    label: isSimilarForm ? '形近' : '词根',
    color: isSimilarForm ? '#8b5cf6' : '#3b82f6',
    strokeDasharray: isSimilarForm ? '5,5' : '8,4',
  };
}

/**
 * 将义-词关联转换为网络边
 */
export function senseWordLinkToEdge(link: SenseWordLinkRecord): NetworkEdge {
  const colorMap: Record<string, string> = {
    synonym: '#10b981',
    antonym: '#ef4444',
    related: '#f59e0b',
  };
  const labelMap: Record<string, string> = {
    synonym: '同义',
    antonym: '反义',
    related: '相关',
  };
  return {
    id: link.link_id,
    source: `sense-${link.sense_id}`,
    target: `word-${link.target_word_id}`,
    type: 'sense-word',
    kind: link.kind,
    label: labelMap[link.kind] || '相关',
    color: colorMap[link.kind] || '#f59e0b',
  };
}

/**
 * 创建归属关系边（义项到单词）
 */
export function createBelongsToEdge(senseId: number, userWordId: number): NetworkEdge {
  return {
    id: `sense-${senseId}-to-word-${userWordId}`,
    source: `sense-${senseId}`,
    target: `word-${userWordId}`,
    type: 'sense-word',
    kind: 'belongs_to',
    label: '属于',
    color: '#94a3b8',
  };
}

/**
 * 构建初始网络图（只包含单词节点）
 */
export function buildNetworkGraph(
  words: UserWordAggregate[],
  _wordLinks: WordLinkRecord[],
  _senseWordLinks: SenseWordLinkRecord[]
): NetworkGraph {
  const nodes: NetworkNode[] = [];
  const edges: NetworkEdge[] = [];

  // 只添加单词节点，不添加子节点
  words.forEach((word) => {
    nodes.push(wordToNode(word));
  });

  // 注意：关联边在展开时动态添加，这里不预先添加

  return { nodes, edges };
}

/**
 * 展开单词节点
 * 创建义的根节点和词-词关联根节点
 */
export function expandWordNode(
  word: UserWordAggregate,
  wordLinks: WordLinkRecord[],
  _existingNodes: NetworkNode[]
): { nodes: NetworkNode[]; edges: NetworkEdge[] } {
  const nodes: NetworkNode[] = [];
  const edges: NetworkEdge[] = [];
  const userWordId = word.user_word.id || 0;
  const wordNodeId = `word-${userWordId}`;

  // 检查是否已有义项，如果有则创建义的根节点
  if (word.user_word.senses.length > 0) {
    const senseGroupNode = createSenseGroupNode(userWordId);
    nodes.push(senseGroupNode);
    // 创建从义根节点到单词节点的边
    edges.push({
      id: `sense-group-${userWordId}-to-word-${userWordId}`,
      source: senseGroupNode.id,
      target: wordNodeId,
      type: 'sense-word',
      kind: 'belongs_to',
      label: '属于',
      color: '#94a3b8',
    });
  }

  // 按类型分组词-词关联
  const linksByKind = new Map<'similar_form' | 'root_affix', WordLinkRecord[]>();
  wordLinks.forEach((link) => {
    if (
      (link.word_a_id === userWordId || link.word_b_id === userWordId) &&
      (link.kind === 'similar_form' || link.kind === 'root_affix')
    ) {
      if (!linksByKind.has(link.kind)) {
        linksByKind.set(link.kind, []);
      }
      linksByKind.get(link.kind)!.push(link);
    }
  });

  // 为每种关联类型创建根节点
  linksByKind.forEach((links, kind) => {
    if (links.length > 0) {
      const groupNode = createWordLinkGroupNode(userWordId, kind);
      nodes.push(groupNode);
      // 创建从关联根节点到单词节点的边
      edges.push({
        id: `word-link-group-${userWordId}-${kind}-to-word-${userWordId}`,
        source: groupNode.id,
        target: wordNodeId,
        type: 'word-word',
        kind: 'group',
        label: '关联',
        color: '#8b5cf6',
        strokeDasharray: '3,3',
      });
    }
  });

  return { nodes, edges };
}

/**
 * 展开义的根节点
 * 创建所有义项节点
 */
export function expandSenseGroup(
  word: UserWordAggregate,
  _existingNodes: NetworkNode[]
): { nodes: NetworkNode[]; edges: NetworkEdge[] } {
  const nodes: NetworkNode[] = [];
  const edges: NetworkEdge[] = [];
  const userWordId = word.user_word.id || 0;
  const senseGroupId = `sense-group-${userWordId}`;

  // 创建所有义项节点
  word.user_word.senses.forEach((sense) => {
    if (sense.id) {
      const senseNode = senseToNode(
        sense.id,
        sense.text,
        word.word.id,
        userWordId
      );
      senseNode.parentId = senseGroupId;
      nodes.push(senseNode);

      // 创建从义项节点到义根节点的边
      edges.push({
        id: `sense-${sense.id}-to-sense-group-${userWordId}`,
        source: senseNode.id,
        target: senseGroupId,
        type: 'sense-word',
        kind: 'belongs_to',
        label: '属于',
        color: '#94a3b8',
      });
    }
  });

  return { nodes, edges: edges };
}

/**
 * 展开词-词关联根节点
 * 创建关联的词节点
 */
export function expandWordLinkGroup(
  userWordId: number,
  kind: 'similar_form' | 'root_affix',
  wordLinks: WordLinkRecord[],
  words: UserWordAggregate[],
  existingNodes: NetworkNode[]
): { nodes: NetworkNode[]; edges: NetworkEdge[] } {
  const nodes: NetworkNode[] = [];
  const edges: NetworkEdge[] = [];
  const groupId = `word-link-group-${userWordId}-${kind}`;

  // 找到相关的关联
  const relatedLinks = wordLinks.filter(
    (link) =>
      link.kind === kind &&
      (link.word_a_id === userWordId || link.word_b_id === userWordId)
  );

  // 找到关联的词节点（如果不存在则创建）
  const relatedWordIds = new Set<number>();
  relatedLinks.forEach((link) => {
    const otherWordId = link.word_a_id === userWordId ? link.word_b_id : link.word_a_id;
    relatedWordIds.add(otherWordId);
  });

  // 检查这些词节点是否已存在
  relatedWordIds.forEach((relatedWordId) => {
    const existingNode = existingNodes.find((n) => n.userWordId === relatedWordId);
    if (!existingNode) {
      // 如果词节点不存在，需要从 words 中找到并创建
      const word = words.find((w) => (w.user_word.id || 0) === relatedWordId);
      if (word) {
        const wordNode = wordToNode(word);
        nodes.push(wordNode);
      }
    }

    // 创建从关联根节点到词节点的边
    edges.push({
      id: `${groupId}-to-word-${relatedWordId}`,
      source: groupId,
      target: `word-${relatedWordId}`,
      type: 'word-word',
      kind: kind,
      label: kind === 'similar_form' ? '形近' : '词根',
      color: kind === 'similar_form' ? '#8b5cf6' : '#3b82f6',
      strokeDasharray: kind === 'similar_form' ? '5,5' : '8,4',
    });
  });

  return { nodes, edges };
}

