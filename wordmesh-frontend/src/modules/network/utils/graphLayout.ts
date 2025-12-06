/**
 * 图形布局算法
 */

import type { NetworkNode, LayoutAlgorithm } from '../types/network.types';

/**
 * 力导向布局（简单实现）
 * 适应更大的节点尺寸（单词节点20，其他节点15）
 */
export function forceLayout(
  nodes: NetworkNode[],
  iterations: number = 100
): NetworkNode[] {
  // 根据节点大小调整间距：基础间距 = 最大节点大小 * 3
  const maxNodeSize = Math.max(...nodes.map((n) => n.size || 15), 20);
  const minDistance = maxNodeSize * 3;
  const k = Math.sqrt((800 * 600) / nodes.length) * 1.5; // 增加基础间距系数
  const nodesWithPos = nodes.map((node) => ({
    ...node,
    x: node.x || Math.random() * 800,
    y: node.y || Math.random() * 600,
    vx: 0,
    vy: 0,
  }));

  for (let iter = 0; iter < iterations; iter++) {
    // 节点间斥力
    for (let i = 0; i < nodesWithPos.length; i++) {
      for (let j = i + 1; j < nodesWithPos.length; j++) {
        const dx = nodesWithPos[i].x! - nodesWithPos[j].x!;
        const dy = nodesWithPos[i].y! - nodesWithPos[j].y!;
        const distance = Math.sqrt(dx * dx + dy * dy) || 1;
        
        // 计算理想距离（基于节点大小）
        const idealDistance = minDistance + (nodesWithPos[i].size || 15) + (nodesWithPos[j].size || 15);
        const force = (k * k) / Math.max(distance, idealDistance * 0.5);

        const fx = (dx / distance) * force * 0.015; // 稍微增加力的大小
        const fy = (dy / distance) * force * 0.015;

        nodesWithPos[i].vx = (nodesWithPos[i].vx || 0) + fx;
        nodesWithPos[i].vy = (nodesWithPos[i].vy || 0) + fy;
        nodesWithPos[j].vx = (nodesWithPos[j].vx || 0) - fx;
        nodesWithPos[j].vy = (nodesWithPos[j].vy || 0) - fy;
      }
    }

    // 更新位置
    for (const node of nodesWithPos) {
      node.x = (node.x || 0) + (node.vx || 0);
      node.y = (node.y || 0) + (node.vy || 0);
      node.vx = (node.vx || 0) * 0.9;
      node.vy = (node.vy || 0) * 0.9;
    }
  }

  return nodesWithPos.map(({ vx, vy, ...node }) => node);
}

/**
 * 圆形布局
 * 适应更大的节点尺寸
 */
export function circularLayout(nodes: NetworkNode[]): NetworkNode[] {
  const centerX = 400;
  const centerY = 300;
  const maxNodeSize = Math.max(...nodes.map((n) => n.size || 15), 20);
  const minRadius = maxNodeSize * 4; // 最小半径基于节点大小
  const radius = Math.min(400, Math.max(minRadius, nodes.length * 8));
  const angleStep = (2 * Math.PI) / nodes.length;

  return nodes.map((node, index) => ({
    ...node,
    x: centerX + radius * Math.cos(index * angleStep),
    y: centerY + radius * Math.sin(index * angleStep),
  }));
}

/**
 * 层次布局
 * 适应更大的节点尺寸
 */
export function hierarchicalLayout(nodes: NetworkNode[]): NetworkNode[] {
  const words = nodes.filter((n) => n.type === 'word');
  const otherNodes = nodes.filter((n) => n.type !== 'word');
  const maxNodeSize = Math.max(...nodes.map((n) => n.size || 15), 20);
  const minSpacing = maxNodeSize * 3;

  const wordY = 100;
  const otherY = 400;
  const wordSpacing = Math.max(minSpacing, 700 / Math.max(1, words.length));
  const otherSpacing = Math.max(minSpacing, 700 / Math.max(1, otherNodes.length));

  const positioned: NetworkNode[] = [];

  words.forEach((node, index) => {
    positioned.push({
      ...node,
      x: 100 + index * wordSpacing,
      y: wordY,
    });
  });

  otherNodes.forEach((node, index) => {
    positioned.push({
      ...node,
      x: 100 + index * otherSpacing,
      y: otherY,
    });
  });

  return positioned;
}

/**
 * 应用布局算法
 */
export function applyLayout(
  nodes: NetworkNode[],
  algorithm: LayoutAlgorithm
): NetworkNode[] {
  switch (algorithm) {
    case 'force':
      return forceLayout(nodes);
    case 'circular':
      return circularLayout(nodes);
    case 'hierarchical':
      return hierarchicalLayout(nodes);
    default:
      return nodes;
  }
}

