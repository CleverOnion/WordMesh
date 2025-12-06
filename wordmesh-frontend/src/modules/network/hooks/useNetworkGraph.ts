/**
 * 网络图 Hook
 * 管理网络图数据、布局和交互
 */

'use client';

import { useState, useCallback, useMemo, useRef, useEffect } from 'react';
import type { NetworkGraph, NetworkNode, NetworkEdge, LayoutAlgorithm } from '../types/network.types';
import { applyLayout } from '../utils/graphLayout';
import type { UserWordAggregate } from '@/modules/word';
import type { WordLinkRecord, SenseWordLinkRecord } from '@/modules/association';
import {
  expandWordNode,
  expandSenseGroup,
  expandWordLinkGroup,
} from '../utils/graphTransform';

export function useNetworkGraph(
  initialGraph?: NetworkGraph,
  words?: UserWordAggregate[],
  wordLinks?: WordLinkRecord[],
  senseWordLinks?: SenseWordLinkRecord[]
) {
  const [graph, setGraph] = useState<NetworkGraph>(initialGraph || { nodes: [], edges: [] });
  const graphRef = useRef(graph);
  const [layout, setLayout] = useState<LayoutAlgorithm>('force');
  const [selectedNode, setSelectedNode] = useState<NetworkNode | null>(null);
  const [highlightedNodes, setHighlightedNodes] = useState<Set<string>>(new Set());
  const [expandedNodes, setExpandedNodes] = useState<Set<string>>(new Set());

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

  // 移除节点及其所有子节点
  // keepParent: 如果为 true，只删除子节点，保留父节点；如果为 false，删除父节点和所有子节点
  const removeNodeAndChildren = useCallback(
    (
      nodeId: string,
      currentNodes: NetworkNode[],
      currentEdges: NetworkEdge[],
      keepParent: boolean = false
    ) => {
      const nodesToRemove = new Set<string>();
      const edgesToRemove = new Set<string>();

      // 如果不保留父节点，则添加父节点到删除列表
      if (!keepParent) {
        nodesToRemove.add(nodeId);
      }

      // 找到所有子节点
      const findChildren = (parentId: string) => {
        currentNodes.forEach((node) => {
          if (node.parentId === parentId) {
            nodesToRemove.add(node.id);
            findChildren(node.id);
          }
        });
      };
      findChildren(nodeId);

      // 找到所有相关的边（只删除连接到要删除节点的边）
      currentEdges.forEach((edge) => {
        // 如果边的源或目标是要删除的节点，则删除这条边
        if (nodesToRemove.has(edge.source) || nodesToRemove.has(edge.target)) {
          edgesToRemove.add(edge.id);
        }
      });

      return {
        nodes: currentNodes.filter((n) => !nodesToRemove.has(n.id)),
        edges: currentEdges.filter((e) => !edgesToRemove.has(e.id)),
      };
    },
    []
  );

  // 切换节点展开/折叠
  const toggleNodeExpansion = useCallback(
    (nodeId: string) => {
      if (!words || !wordLinks || !senseWordLinks) return;

      const currentGraph = graphRef.current;
      const node = currentGraph.nodes.find((n) => n.id === nodeId);
      if (!node) return;

      const isExpanded = expandedNodes.has(nodeId);

      setGraph((prev) => {
        let newNodes = [...prev.nodes];
        let newEdges = [...prev.edges];

        if (isExpanded) {
          // 折叠：只移除子节点，保留父节点本身
          // 对于所有类型的节点（word, sense-group, word-link-group），都只删除子节点
          const result = removeNodeAndChildren(nodeId, newNodes, newEdges, true);
          newNodes = result.nodes;
          newEdges = result.edges;

          // 更新父节点的 expanded 状态
          newNodes = newNodes.map((n) =>
            n.id === nodeId ? { ...n, expanded: false } : n
          );

          setExpandedNodes((prevExpanded) => {
            const next = new Set(prevExpanded);
            next.delete(nodeId);
            return next;
          });
        } else {
          // 展开：根据节点类型添加子节点
          if (node.type === 'word') {
            // 展开单词节点
            const word = words.find((w) => (w.user_word.id || 0) === node.userWordId);
            if (word) {
              const { nodes: childNodes, edges: childEdges } = expandWordNode(
                word,
                wordLinks,
                newNodes
              );
              newNodes = [...newNodes, ...childNodes];
              newEdges = [...newEdges, ...childEdges];
            }
          } else if (node.type === 'sense-group') {
            // 展开义的根节点
            const userWordId = node.userWordId;
            if (userWordId) {
              const word = words.find((w) => (w.user_word.id || 0) === userWordId);
              if (word) {
                const { nodes: childNodes, edges: childEdges } = expandSenseGroup(
                  word,
                  newNodes
                );
                newNodes = [...newNodes, ...childNodes];
                newEdges = [...newEdges, ...childEdges];

                // 添加义-词关联边
                senseWordLinks.forEach((link) => {
                  if (word.user_word.senses.some((s) => s.id === link.sense_id)) {
                    const edgeExists = newEdges.some((e) => e.id === link.link_id);
                    if (!edgeExists) {
                      newEdges.push({
                        id: link.link_id,
                        source: `sense-${link.sense_id}`,
                        target: `word-${link.target_word_id}`,
                        type: 'sense-word',
                        kind: link.kind,
                        label:
                          link.kind === 'synonym'
                            ? '同义'
                            : link.kind === 'antonym'
                            ? '反义'
                            : '相关',
                        color:
                          link.kind === 'synonym'
                            ? '#10b981'
                            : link.kind === 'antonym'
                            ? '#ef4444'
                            : '#f59e0b',
                      });
                    }
                  }
                });
              }
            }
          } else if (node.type === 'word-link-group') {
            // 展开词-词关联根节点
            const userWordId = node.userWordId;
            const linkKind = node.linkKind as 'similar_form' | 'root_affix';
            if (userWordId && linkKind) {
              const { nodes: childNodes, edges: childEdges } = expandWordLinkGroup(
                userWordId,
                linkKind,
                wordLinks,
                words,
                newNodes
              );
              newNodes = [...newNodes, ...childNodes];
              newEdges = [...newEdges, ...childEdges];
            }
          }

          // 更新节点的 expanded 状态
          newNodes = newNodes.map((n) =>
            n.id === nodeId ? { ...n, expanded: true } : n
          );

          setExpandedNodes((prevExpanded) => new Set(prevExpanded).add(nodeId));
        }

        // 应用布局
        const laidOutNodes = applyLayout(newNodes, layout);
        return { nodes: laidOutNodes, edges: newEdges };
      });
    },
    [words, wordLinks, senseWordLinks, layout, removeNodeAndChildren, expandedNodes]
  );

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
    toggleNodeExpansion,
    expandedNodes,
  };
}

