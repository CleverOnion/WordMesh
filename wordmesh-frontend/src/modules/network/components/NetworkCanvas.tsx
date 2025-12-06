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
    <div className={cn('relative border rounded-lg overflow-hidden bg-background', className)}>
      <svg
        ref={svgRef}
        width={width}
        height={height}
        className="w-full h-full"
        onMouseMove={handleMouseMove}
        onMouseUp={handleMouseUp}
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
        </defs>

        {/* 渲染边 */}
        <g>
          {graph.edges.map((edge) => {
            const sourceNode = graph.nodes.find((n) => n.id === edge.source);
            const targetNode = graph.nodes.find((n) => n.id === edge.target);

            if (!sourceNode || !targetNode || !sourceNode.x || !sourceNode.y || !targetNode.x || !targetNode.y) {
              return null;
            }

            return (
              <line
                key={edge.id}
                x1={sourceNode.x}
                y1={sourceNode.y}
                x2={targetNode.x}
                y2={targetNode.y}
                stroke={edge.color || '#666'}
                strokeWidth={2}
                markerEnd="url(#arrowhead)"
                className="transition-opacity duration-200"
                opacity={edge.source === draggedNode || edge.target === draggedNode ? 1 : 0.6}
              />
            );
          })}
        </g>

        {/* 渲染节点 */}
        <g>
          {graph.nodes.map((node) => {
            if (node.x === undefined || node.y === undefined) return null;

            const isHighlighted = (node as any).highlighted;
            const isSelected = (node as any).selected;
            const nodeSize = node.size || 10;

            return (
              <g
                key={node.id}
                className="cursor-pointer transition-all duration-200"
                onClick={() => onNodeClick?.(node)}
                onMouseDown={(e) => handleMouseDown(node, e)}
              >
                <circle
                  cx={node.x}
                  cy={node.y}
                  r={isSelected ? nodeSize + 3 : nodeSize}
                  fill={node.color || '#3b82f6'}
                  stroke={isSelected ? '#fbbf24' : isHighlighted ? '#f59e0b' : 'transparent'}
                  strokeWidth={isSelected ? 3 : isHighlighted ? 2 : 0}
                  className="hover:opacity-80"
                />
                <text
                  x={node.x}
                  y={node.y + nodeSize + 15}
                  textAnchor="middle"
                  fontSize="12"
                  fill="currentColor"
                  className="pointer-events-none select-none"
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

