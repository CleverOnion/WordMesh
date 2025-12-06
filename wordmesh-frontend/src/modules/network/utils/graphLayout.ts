/**
 * 图形布局算法
 */

import type { NetworkNode, LayoutAlgorithm } from '../types/network.types';

/**
 * 力导向布局（简单实现）
 */
export function forceLayout(
  nodes: NetworkNode[],
  iterations: number = 100
): NetworkNode[] {
  const k = Math.sqrt((800 * 600) / nodes.length);
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
        const force = (k * k) / distance;

        const fx = (dx / distance) * force * 0.01;
        const fy = (dy / distance) * force * 0.01;

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
 */
export function circularLayout(nodes: NetworkNode[]): NetworkNode[] {
  const centerX = 400;
  const centerY = 300;
  const radius = Math.min(300, Math.max(100, nodes.length * 5));
  const angleStep = (2 * Math.PI) / nodes.length;

  return nodes.map((node, index) => ({
    ...node,
    x: centerX + radius * Math.cos(index * angleStep),
    y: centerY + radius * Math.sin(index * angleStep),
  }));
}

/**
 * 层次布局
 */
export function hierarchicalLayout(nodes: NetworkNode[]): NetworkNode[] {
  const words = nodes.filter((n) => n.type === 'word');
  const senses = nodes.filter((n) => n.type === 'sense');

  const wordY = 100;
  const senseY = 400;
  const wordSpacing = 600 / Math.max(1, words.length);
  const senseSpacing = 600 / Math.max(1, senses.length);

  const positioned: NetworkNode[] = [];

  words.forEach((node, index) => {
    positioned.push({
      ...node,
      x: 100 + index * wordSpacing,
      y: wordY,
    });
  });

  senses.forEach((node, index) => {
    positioned.push({
      ...node,
      x: 100 + index * senseSpacing,
      y: senseY,
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

