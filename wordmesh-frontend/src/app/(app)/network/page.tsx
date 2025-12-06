/**
 * 网络视图页面 - 主页面
 * 全屏布局，集成浮动工具栏和抽屉功能
 */

'use client';

import { useState, useEffect } from 'react';
import { NetworkCanvas, NetworkControls, useNetworkGraph, buildNetworkGraph } from '@/modules/network';
import { AnimatedGridBackground } from '@/modules/network/components/AnimatedGridBackground';
import { useWordList } from '@/modules/word';
import { useAssociationList } from '@/modules/association';
import { NetworkNodeDetail } from '@/modules/network';
import { FloatingToolbar } from '@/modules/app/components/FloatingToolbar';
import { WordManagementDrawer } from '@/modules/app/components/WordManagementDrawer';
import { SearchDrawer } from '@/modules/app/components/SearchDrawer';
import { SettingsDrawer } from '@/modules/app/components/SettingsDrawer';
import { Card, CardContent } from '@/components/ui/card';
import { X } from 'lucide-react';
import { Button } from '@/components/ui/button';
import type { NetworkNode } from '@/modules/network';

export default function NetworkPage() {
  const { words = [], isLoading: wordsLoading } = useWordList();
  const [selectedWordId, setSelectedWordId] = useState<number | null>(null);
  
  // 获取关联数据
  const { links: wordLinks } = useAssociationList('word', selectedWordId || 0);
  const { links: senseWordLinks } = useAssociationList('sense', 0);

  const networkGraph = useNetworkGraph();
  const [selectedNode, setSelectedNode] = useState<NetworkNode | null>(null);

  // 抽屉状态
  const [isWordsDrawerOpen, setIsWordsDrawerOpen] = useState(false);
  const [isSearchDrawerOpen, setIsSearchDrawerOpen] = useState(false);
  const [isSettingsDrawerOpen, setIsSettingsDrawerOpen] = useState(false);
  const [isNodeDetailOpen, setIsNodeDetailOpen] = useState(false);

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
  }, [words, wordLinks, senseWordLinks, networkGraph.setGraph, networkGraph.applyLayout]);

  const handleNodeClick = (node: NetworkNode) => {
    setSelectedNode(node);
    setIsNodeDetailOpen(true);
  };

  // 获取视口尺寸
  const [viewportSize, setViewportSize] = useState({ width: 1920, height: 1080 });

  useEffect(() => {
    const updateSize = () => {
      setViewportSize({
        width: window.innerWidth,
        height: window.innerHeight,
      });
    };

    updateSize();
    window.addEventListener('resize', updateSize);
    return () => window.removeEventListener('resize', updateSize);
  }, []);

  return (
    <div className="fixed inset-0 overflow-hidden bg-background">
      {/* 动画网格背景 */}
      <AnimatedGridBackground className="opacity-60" />

      {/* 浮动工具栏 */}
      <FloatingToolbar
        onOpenWords={() => setIsWordsDrawerOpen(true)}
        onOpenSearch={() => setIsSearchDrawerOpen(true)}
        onOpenSettings={() => setIsSettingsDrawerOpen(true)}
      />

      {/* 浮动控制栏 */}
      <NetworkControls
        layout={networkGraph.layout}
        onLayoutChange={networkGraph.applyLayout}
        onReset={networkGraph.resetLayout}
      />

      {/* 网络画布 */}
      <div className="absolute inset-0 flex items-center justify-center">
        {wordsLoading ? (
          <div className="flex items-center justify-center">
            <p className="text-muted-foreground text-lg">加载中...</p>
          </div>
        ) : networkGraph.graph.nodes.length > 0 ? (
          <NetworkCanvas
            graph={networkGraph.graph}
            onNodeClick={handleNodeClick}
            onNodeDrag={networkGraph.updateNodePosition}
            width={viewportSize.width}
            height={viewportSize.height}
            className="w-full h-full"
          />
        ) : (
          <div className="flex flex-col items-center justify-center gap-4">
            <p className="text-muted-foreground text-lg">
              还没有单词，请先添加一些单词到您的词网
            </p>
            <Button onClick={() => setIsWordsDrawerOpen(true)}>
              添加单词
            </Button>
          </div>
        )}
      </div>

      {/* 节点详情浮动面板 */}
      {selectedNode && isNodeDetailOpen && (
        <div className="fixed bottom-2 right-2 sm:bottom-4 sm:right-4 z-30 w-[calc(100vw-1rem)] sm:w-80 max-w-[calc(100vw-2rem)]">
          <Card className="shadow-2xl border-border bg-card/95 backdrop-blur-sm animate-in slide-in-from-bottom-1">
            <div className="flex items-center justify-between p-4 border-b">
              <h3 className="font-semibold">节点详情</h3>
              <Button
                variant="ghost"
                size="icon"
                onClick={() => {
                  setIsNodeDetailOpen(false);
                  setSelectedNode(null);
                }}
                className="h-8 w-8"
              >
                <X className="size-4" />
              </Button>
            </div>
            <CardContent className="p-4">
              <NetworkNodeDetail node={selectedNode} />
            </CardContent>
          </Card>
        </div>
      )}

      {/* 单词管理抽屉 */}
      <WordManagementDrawer
        isOpen={isWordsDrawerOpen}
        onClose={() => setIsWordsDrawerOpen(false)}
      />

      {/* 搜索抽屉 */}
      <SearchDrawer
        isOpen={isSearchDrawerOpen}
        onClose={() => setIsSearchDrawerOpen(false)}
      />

      {/* 设置抽屉 */}
      <SettingsDrawer
        isOpen={isSettingsDrawerOpen}
        onClose={() => setIsSettingsDrawerOpen(false)}
      />
    </div>
  );
}

