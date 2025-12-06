/**
 * 网络视图页面
 */

'use client';

import { useState, useEffect } from 'react';
import { NetworkCanvas, NetworkControls, useNetworkGraph, buildNetworkGraph } from '@/modules/network';
import { useWordList } from '@/modules/word';
import { useAssociationList } from '@/modules/association';
import { NetworkNodeDetail } from '@/modules/network';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Network } from 'lucide-react';
import type { NetworkNode } from '@/modules/network';

export default function NetworkPage() {
  const { words = [], isLoading: wordsLoading } = useWordList();
  const [selectedWordId, setSelectedWordId] = useState<number | null>(null);
  
  // 获取关联数据（这里简化处理，实际应该根据选中的单词获取）
  const { links: wordLinks } = useAssociationList('word', selectedWordId || 0);
  const { links: senseWordLinks } = useAssociationList('sense', 0);

  const networkGraph = useNetworkGraph();
  const [selectedNode, setSelectedNode] = useState<NetworkNode | null>(null);

  // 构建网络图
  useEffect(() => {
    if (words.length > 0) {
      const graph = buildNetworkGraph(
        words,
        wordLinks.filter((link): link is import('@/modules/association').WordLinkRecord => 
          'word_a_id' in link
        ),
        senseWordLinks.filter((link): link is import('@/modules/association').SenseWordLinkRecord => 
          'sense_id' in link
        )
      );
      networkGraph.setGraph(graph);
      networkGraph.applyLayout('force');
    }
  }, [words, wordLinks, senseWordLinks, networkGraph]);

  const handleNodeClick = (node: NetworkNode) => {
    setSelectedNode(node);
  };

  return (
    <div className="space-y-6">
      {/* 页面标题 */}
      <div>
        <h1 className="text-3xl font-bold flex items-center gap-2">
          <Network className="size-8" />
          网络视图
        </h1>
        <p className="text-muted-foreground mt-2">
          可视化您的单词知识网络
        </p>
      </div>

      <div className="grid gap-6 lg:grid-cols-4">
        {/* 网络图 */}
        <div className="lg:col-span-3">
          <Card>
            <NetworkControls
              layout={networkGraph.layout}
              onLayoutChange={networkGraph.applyLayout}
              onReset={networkGraph.resetLayout}
            />
            <CardContent className="p-0">
              {wordsLoading ? (
                <div className="flex items-center justify-center h-[600px]">
                  <p className="text-muted-foreground">加载中...</p>
                </div>
              ) : networkGraph.graph.nodes.length > 0 ? (
                <NetworkCanvas
                  graph={networkGraph.graph}
                  onNodeClick={handleNodeClick}
                  onNodeDrag={networkGraph.updateNodePosition}
                  width={800}
                  height={600}
                  className="w-full"
                />
              ) : (
                <div className="flex items-center justify-center h-[600px]">
                  <p className="text-muted-foreground">
                    还没有单词，请先添加一些单词到您的词网
                  </p>
                </div>
              )}
            </CardContent>
          </Card>
        </div>

        {/* 节点详情 */}
        <div className="lg:col-span-1">
          {selectedNode ? (
            <NetworkNodeDetail node={selectedNode} />
          ) : (
            <Card>
              <CardContent className="py-12 text-center">
                <p className="text-muted-foreground">
                  点击网络图中的节点查看详情
                </p>
              </CardContent>
            </Card>
          )}
        </div>
      </div>
    </div>
  );
}

