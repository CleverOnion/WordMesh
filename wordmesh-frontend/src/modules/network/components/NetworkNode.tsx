/**
 * 网络节点组件（用于详情展示）
 */

'use client';

import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import type { NetworkNode } from '../types/network.types';

interface NetworkNodeProps {
  node: NetworkNode;
  className?: string;
}

export function NetworkNodeDetail({ node, className }: NetworkNodeProps) {
  return (
    <Card className={className}>
      <CardHeader>
        <div className="flex items-center justify-between">
          <CardTitle className="text-lg">{node.label}</CardTitle>
          <Badge variant={node.type === 'word' ? 'default' : 'secondary'}>
            {node.type === 'word' ? '单词' : '义项'}
          </Badge>
        </div>
        <CardDescription>节点 ID: {node.id}</CardDescription>
      </CardHeader>
      <CardContent>
        <div className="space-y-2 text-sm">
          {node.wordId && <div>单词 ID: {node.wordId}</div>}
          {node.senseId && <div>义项 ID: {node.senseId}</div>}
          {node.userWordId && <div>用户词项 ID: {node.userWordId}</div>}
        </div>
      </CardContent>
    </Card>
  );
}

