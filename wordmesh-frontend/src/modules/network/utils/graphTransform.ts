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
    size: 10,
    color: '#3b82f6',
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
    size: 8,
    color: '#10b981',
  };
}

/**
 * 将词-词关联转换为网络边
 */
export function wordLinkToEdge(link: WordLinkRecord): NetworkEdge {
  return {
    id: link.link_id,
    source: `word-${link.word_a_id}`,
    target: `word-${link.word_b_id}`,
    type: 'word-word',
    kind: link.kind,
    label: link.kind === 'similar_form' ? '形近' : '词根',
    color: '#8b5cf6',
  };
}

/**
 * 将义-词关联转换为网络边
 */
export function senseWordLinkToEdge(link: SenseWordLinkRecord): NetworkEdge {
  return {
    id: link.link_id,
    source: `sense-${link.sense_id}`,
    target: `word-${link.target_word_id}`,
    type: 'sense-word',
    kind: link.kind,
    label: link.kind === 'synonym' ? '同义' : link.kind === 'antonym' ? '反义' : '相关',
    color: '#f59e0b',
  };
}

/**
 * 构建完整的网络图
 */
export function buildNetworkGraph(
  words: UserWordAggregate[],
  wordLinks: WordLinkRecord[],
  senseWordLinks: SenseWordLinkRecord[]
): NetworkGraph {
  const nodes: NetworkNode[] = [];
  const edges: NetworkEdge[] = [];

  // 添加单词节点
  words.forEach((word) => {
    const wordNode = wordToNode(word);
    nodes.push(wordNode);

    // 添加义项节点，并创建义项到单词的连线
    word.user_word.senses.forEach((sense) => {
      if (sense.id) {
        const senseNode = senseToNode(
          sense.id,
          sense.text,
          word.word.id,
          word.user_word.id || 0
        );
        nodes.push(senseNode);

        // 创建义项节点到单词节点的连线
        edges.push({
          id: `sense-${sense.id}-to-word-${word.user_word.id}`,
          source: senseNode.id,
          target: wordNode.id,
          type: 'sense-word',
          kind: 'belongs_to',
          label: '属于',
          color: '#94a3b8', // 使用灰色表示归属关系
        });
      }
    });
  });

  // 添加词-词关联边
  wordLinks.forEach((link) => {
    edges.push(wordLinkToEdge(link));
  });

  // 添加义-词关联边
  senseWordLinks.forEach((link) => {
    edges.push(senseWordLinkToEdge(link));
  });

  return { nodes, edges };
}

