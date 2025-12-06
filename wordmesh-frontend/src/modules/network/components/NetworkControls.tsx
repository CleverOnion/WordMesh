/**
 * 网络图控制组件
 */

'use client';

import { ZoomIn, ZoomOut, RotateCcw, Maximize2, Minimize2 } from 'lucide-react';
import { Button } from '@/components/ui/button';
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select';
import type { LayoutAlgorithm } from '../types/network.types';

interface NetworkControlsProps {
  layout: LayoutAlgorithm;
  onLayoutChange: (layout: LayoutAlgorithm) => void;
  onZoomIn?: () => void;
  onZoomOut?: () => void;
  onReset?: () => void;
  onFitToScreen?: () => void;
  className?: string;
}

export function NetworkControls({
  layout,
  onLayoutChange,
  onZoomIn,
  onZoomOut,
  onReset,
  onFitToScreen,
  className,
}: NetworkControlsProps) {
  return (
    <div className={`flex items-center gap-2 p-4 border-b bg-card ${className}`}>
      <Select value={layout} onValueChange={(value) => onLayoutChange(value as LayoutAlgorithm)}>
        <SelectTrigger className="w-[140px]">
          <SelectValue />
        </SelectTrigger>
        <SelectContent>
          <SelectItem value="force">力导向布局</SelectItem>
          <SelectItem value="hierarchical">层次布局</SelectItem>
          <SelectItem value="circular">圆形布局</SelectItem>
        </SelectContent>
      </Select>

      <div className="flex gap-1 ml-auto">
        {onZoomIn && (
          <Button variant="outline" size="icon" onClick={onZoomIn} title="放大">
            <ZoomIn className="size-4" />
          </Button>
        )}
        {onZoomOut && (
          <Button variant="outline" size="icon" onClick={onZoomOut} title="缩小">
            <ZoomOut className="size-4" />
          </Button>
        )}
        {onFitToScreen && (
          <Button variant="outline" size="icon" onClick={onFitToScreen} title="适应屏幕">
            <Maximize2 className="size-4" />
          </Button>
        )}
        {onReset && (
          <Button variant="outline" size="icon" onClick={onReset} title="重置布局">
            <RotateCcw className="size-4" />
          </Button>
        )}
      </div>
    </div>
  );
}

