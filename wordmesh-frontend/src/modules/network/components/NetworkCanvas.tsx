/**
 * 网络画布组件
 * 使用 SVG 渲染网络图
 */

'use client';

import { useEffect, useRef, useState } from 'react';
import type { NetworkGraph, NetworkNode, NetworkEdge } from '../types/network.types';
import { cn } from '@/lib/utils';

interface NetworkCanvasProps {
  graph: NetworkGraph;
  onNodeClick?: (node: NetworkNode) => void;
  onNodeDrag?: (nodeId: string, x: number, y: number) => void;
  width?: number;
  height?: number;
  className?: string;
}

export function NetworkCanvas({
  graph,
  onNodeClick,
  onNodeDrag,
  width = 800,
  height = 600,
  className,
}: NetworkCanvasProps) {
  const svgRef = useRef<SVGSVGElement>(null);
  const [draggedNode, setDraggedNode] = useState<string | null>(null);
  const [dragStart, setDragStart] = useState<{ x: number; y: number } | null>(null);
  const [hoveredNode, setHoveredNode] = useState<string | null>(null);
  const [selectedNode, setSelectedNode] = useState<string | null>(null);

  // 处理节点拖拽
  const handleMouseDown = (node: NetworkNode, e: React.MouseEvent) => {
    e.stopPropagation();
    setDraggedNode(node.id);
    if (svgRef.current) {
      const rect = svgRef.current.getBoundingClientRect();
      setDragStart({
        x: e.clientX - rect.left,
        y: e.clientY - rect.top,
      });
    }
  };

  const handleMouseMove = (e: React.MouseEvent) => {
    if (draggedNode && dragStart && svgRef.current) {
      const rect = svgRef.current.getBoundingClientRect();
      const x = e.clientX - rect.left;
      const y = e.clientY - rect.top;
      onNodeDrag?.(draggedNode, x, y);
    }
  };

  const handleMouseUp = () => {
    setDraggedNode(null);
    setDragStart(null);
  };

  const handleNodeClick = (node: NetworkNode, e: React.MouseEvent) => {
    e.stopPropagation();
    setSelectedNode(node.id);
    onNodeClick?.(node);
  };

  const handleNodeHover = (nodeId: string | null) => {
    setHoveredNode(nodeId);
  };

  useEffect(() => {
    if (draggedNode) {
      window.addEventListener('mousemove', handleMouseMove as any);
      window.addEventListener('mouseup', handleMouseUp);
      return () => {
        window.removeEventListener('mousemove', handleMouseMove as any);
        window.removeEventListener('mouseup', handleMouseUp);
      };
    }
  }, [draggedNode, dragStart]);

  return (
    <div className={cn('relative overflow-hidden bg-transparent', className)}>
      <svg
        ref={svgRef}
        width={width}
        height={height}
        className="w-full h-full"
        onMouseMove={handleMouseMove}
        onMouseUp={handleMouseUp}
        onClick={() => setSelectedNode(null)}
      >
        <defs>
          <marker
            id="arrowhead"
            markerWidth="10"
            markerHeight="10"
            refX="9"
            refY="3"
            orient="auto"
          >
            <polygon points="0 0, 10 3, 0 6" fill="#666" />
          </marker>
          {/* 节点发光效果 */}
          <filter id="glow">
            <feGaussianBlur stdDeviation="3" result="coloredBlur" />
            <feMerge>
              <feMergeNode in="coloredBlur" />
              <feMergeNode in="SourceGraphic" />
            </feMerge>
          </filter>
          {/* 悬停发光效果 */}
          <filter id="hover-glow">
            <feGaussianBlur stdDeviation="5" result="coloredBlur" />
            <feMerge>
              <feMergeNode in="coloredBlur" />
              <feMergeNode in="SourceGraphic" />
            </feMerge>
          </filter>
        </defs>

        {/* 渲染边 */}
        <g>
          {graph.edges.map((edge) => {
            const sourceNode = graph.nodes.find((n) => n.id === edge.source);
            const targetNode = graph.nodes.find((n) => n.id === edge.target);

            if (!sourceNode || !targetNode || !sourceNode.x || !sourceNode.y || !targetNode.x || !targetNode.y) {
              return null;
            }

            const isConnectedToHovered =
              edge.source === hoveredNode || edge.target === hoveredNode;
            const isConnectedToSelected =
              edge.source === selectedNode || edge.target === selectedNode;
            const isConnectedToDragged =
              edge.source === draggedNode || edge.target === draggedNode;

            return (
              <line
                key={edge.id}
                x1={sourceNode.x}
                y1={sourceNode.y}
                x2={targetNode.x}
                y2={targetNode.y}
                stroke={edge.color || '#666'}
                strokeWidth={
                  isConnectedToDragged ? 3 : isConnectedToHovered || isConnectedToSelected ? 2.5 : 2
                }
                markerEnd="url(#arrowhead)"
                className="transition-all duration-200 ease-out"
                opacity={
                  isConnectedToDragged
                    ? 1
                    : isConnectedToHovered || isConnectedToSelected
                    ? 0.8
                    : 0.4
                }
              />
            );
          })}
        </g>

        {/* 渲染节点 */}
        <g>
          {graph.nodes.map((node) => {
            if (node.x === undefined || node.y === undefined) return null;

            const isHighlighted = (node as any).highlighted;
            const isNodeSelected = selectedNode === node.id;
            const isNodeHovered = hoveredNode === node.id;
            const isNodeDragged = draggedNode === node.id;
            const nodeSize = node.size || 10;
            
            // 动态计算节点大小和样式
            const baseRadius = nodeSize;
            const hoverRadius = baseRadius + 2;
            const selectedRadius = baseRadius + 4;
            const currentRadius = isNodeDragged
              ? selectedRadius
              : isNodeSelected
              ? selectedRadius
              : isNodeHovered
              ? hoverRadius
              : baseRadius;

            const nodeColor = node.color || '#3b82f6';
            const strokeColor = isNodeSelected
              ? '#fbbf24'
              : isNodeHovered
              ? '#60a5fa'
              : isHighlighted
              ? '#f59e0b'
              : 'transparent';
            const strokeWidth = isNodeSelected ? 3 : isNodeHovered ? 2 : isHighlighted ? 2 : 0;

            return (
              <g
                key={node.id}
                className="cursor-pointer transition-all duration-200 ease-out gpu-accelerated"
                onClick={(e) => handleNodeClick(node, e)}
                onMouseDown={(e) => handleMouseDown(node, e)}
                onMouseEnter={() => handleNodeHover(node.id)}
                onMouseLeave={() => handleNodeHover(null)}
              >
                {/* 节点外圈发光效果 */}
                {(isNodeHovered || isNodeSelected) && (
                  <circle
                    cx={node.x}
                    cy={node.y}
                    r={currentRadius + 4}
                    fill="none"
                    stroke={strokeColor}
                    strokeWidth={1}
                    opacity={0.3}
                    className="transition-all duration-200"
                    filter={isNodeHovered ? 'url(#hover-glow)' : 'url(#glow)'}
                  />
                )}
                {/* 节点主体 */}
                <circle
                  cx={node.x}
                  cy={node.y}
                  r={currentRadius}
                  fill={nodeColor}
                  stroke={strokeColor}
                  strokeWidth={strokeWidth}
                  className="transition-all duration-200 ease-out"
                  style={{
                    filter: isNodeHovered || isNodeSelected ? 'url(#glow)' : 'none',
                    transformOrigin: `${node.x}px ${node.y}px`,
                  }}
                />
                {/* 节点标签 */}
                <text
                  x={node.x}
                  y={node.y + currentRadius + 18}
                  textAnchor="middle"
                  fontSize={isNodeHovered || isNodeSelected ? '13' : '12'}
                  fill="currentColor"
                  className="pointer-events-none select-none transition-all duration-200"
                  fontWeight={isNodeSelected ? 'bold' : 'normal'}
                  style={{
                    textShadow: isNodeHovered || isNodeSelected
                      ? '0 1px 2px rgba(0,0,0,0.1)'
                      : 'none',
                  }}
                >
                  {node.label}
                </text>
              </g>
            );
          })}
        </g>
      </svg>
    </div>
  );
}

