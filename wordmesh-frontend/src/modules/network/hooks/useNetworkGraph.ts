/**
 * 网络图 Hook
 * 管理网络图数据、布局和交互
 */

'use client';

import { useState, useCallback, useMemo } from 'react';
import type { NetworkGraph, NetworkNode, NetworkEdge, LayoutAlgorithm } from '../types/network.types';
import { applyLayout } from '../utils/graphLayout';

export function useNetworkGraph(initialGraph?: NetworkGraph) {
  const [graph, setGraph] = useState<NetworkGraph>(initialGraph || { nodes: [], edges: [] });
  const [layout, setLayout] = useState<LayoutAlgorithm>('force');
  const [selectedNode, setSelectedNode] = useState<NetworkNode | null>(null);
  const [highlightedNodes, setHighlightedNodes] = useState<Set<string>>(new Set());

  // 应用布局
  const applyLayoutToGraph = useCallback(
    (algorithm: LayoutAlgorithm) => {
      setLayout(algorithm);
      const laidOutNodes = applyLayout(graph.nodes, algorithm);
      setGraph({ ...graph, nodes: laidOutNodes });
    },
    [graph]
  );

  // 选择节点
  const selectNode = useCallback((nodeId: string | null) => {
    if (!nodeId) {
      setSelectedNode(null);
      setHighlightedNodes(new Set());
      return;
    }

    const node = graph.nodes.find((n) => n.id === nodeId);
    if (node) {
      setSelectedNode(node);
      // 高亮相关节点
      const related = new Set<string>([nodeId]);
      graph.edges.forEach((edge) => {
        if (edge.source === nodeId) related.add(edge.target);
        if (edge.target === nodeId) related.add(edge.source);
      });
      setHighlightedNodes(related);
    }
  }, [graph]);

  // 更新节点位置
  const updateNodePosition = useCallback((nodeId: string, x: number, y: number) => {
    setGraph((prev) => ({
      ...prev,
      nodes: prev.nodes.map((node) =>
        node.id === nodeId ? { ...node, x, y, fx: x, fy: y } : node
      ),
    }));
  }, []);

  // 释放节点（允许力导向布局移动）
  const releaseNode = useCallback((nodeId: string) => {
    setGraph((prev) => ({
      ...prev,
      nodes: prev.nodes.map((node) =>
        node.id === nodeId ? { ...node, fx: null, fy: null } : node
      ),
    }));
  }, []);

  // 重置布局
  const resetLayout = useCallback(() => {
    applyLayoutToGraph(layout);
  }, [layout, applyLayoutToGraph]);

  // 计算后的节点（应用高亮）
  const computedNodes = useMemo(() => {
    return graph.nodes.map((node) => ({
      ...node,
      highlighted: highlightedNodes.has(node.id),
      selected: selectedNode?.id === node.id,
    }));
  }, [graph.nodes, highlightedNodes, selectedNode]);

  return {
    graph: { ...graph, nodes: computedNodes },
    layout,
    selectedNode,
    setGraph,
    applyLayout: applyLayoutToGraph,
    selectNode,
    updateNodePosition,
    releaseNode,
    resetLayout,
    setLayout,
  };
}

