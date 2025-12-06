/**
 * 网络图 Hook
 * 管理网络图数据、布局和交互
 */

'use client';

import { useState, useCallback, useMemo, useRef, useEffect } from 'react';
import type { NetworkGraph, NetworkNode, NetworkEdge, LayoutAlgorithm } from '../types/network.types';
import { applyLayout } from '../utils/graphLayout';

export function useNetworkGraph(initialGraph?: NetworkGraph) {
  const [graph, setGraph] = useState<NetworkGraph>(initialGraph || { nodes: [], edges: [] });
  const graphRef = useRef(graph);
  const [layout, setLayout] = useState<LayoutAlgorithm>('force');
  const [selectedNode, setSelectedNode] = useState<NetworkNode | null>(null);
  const [highlightedNodes, setHighlightedNodes] = useState<Set<string>>(new Set());

  // 保持 ref 与 state 同步
  useEffect(() => {
    graphRef.current = graph;
  }, [graph]);

  // 应用布局
  const applyLayoutToGraph = useCallback(
    (algorithm: LayoutAlgorithm) => {
      setLayout(algorithm);
      setGraph((prev) => {
        const laidOutNodes = applyLayout(prev.nodes, algorithm);
        return { ...prev, nodes: laidOutNodes };
      });
    },
    []
  );

  // 选择节点
  const selectNode = useCallback((nodeId: string | null) => {
    if (!nodeId) {
      setSelectedNode(null);
      setHighlightedNodes(new Set());
      return;
    }

    const currentGraph = graphRef.current;
    const node = currentGraph.nodes.find((n) => n.id === nodeId);
    if (node) {
      setSelectedNode(node);
      // 高亮相关节点
      const related = new Set<string>([nodeId]);
      currentGraph.edges.forEach((edge) => {
        if (edge.source === nodeId) related.add(edge.target);
        if (edge.target === nodeId) related.add(edge.source);
      });
      setHighlightedNodes(related);
    }
  }, []);

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

