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
import { cn } from '@/lib/utils';
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
    <div
      className={cn(
        'fixed top-2 right-2 sm:top-4 sm:right-4 z-30 flex flex-col sm:flex-row items-stretch sm:items-center gap-2 p-2 sm:p-3 rounded-lg',
        'bg-card/90 backdrop-blur-sm border border-border shadow-lg',
        'transition-smooth gpu-accelerated',
        className
      )}
    >
      <Select value={layout} onValueChange={(value) => onLayoutChange(value as LayoutAlgorithm)}>
        <SelectTrigger className="w-full sm:w-[140px] h-9 text-xs sm:text-sm">
          <SelectValue />
        </SelectTrigger>
        <SelectContent>
          <SelectItem value="force">力导向布局</SelectItem>
          <SelectItem value="hierarchical">层次布局</SelectItem>
          <SelectItem value="circular">圆形布局</SelectItem>
        </SelectContent>
      </Select>

      <div className="flex gap-1">
        {onZoomIn && (
          <Button
            variant="outline"
            size="icon"
            onClick={onZoomIn}
            title="放大"
            className="h-9 w-9 transition-smooth hover:scale-110"
          >
            <ZoomIn className="size-4" />
          </Button>
        )}
        {onZoomOut && (
          <Button
            variant="outline"
            size="icon"
            onClick={onZoomOut}
            title="缩小"
            className="h-9 w-9 transition-smooth hover:scale-110"
          >
            <ZoomOut className="size-4" />
          </Button>
        )}
        {onFitToScreen && (
          <Button
            variant="outline"
            size="icon"
            onClick={onFitToScreen}
            title="适应屏幕"
            className="h-9 w-9 transition-smooth hover:scale-110"
          >
            <Maximize2 className="size-4" />
          </Button>
        )}
        {onReset && (
          <Button
            variant="outline"
            size="icon"
            onClick={onReset}
            title="重置布局"
            className="h-9 w-9 transition-smooth hover:scale-110"
          >
            <RotateCcw className="size-4" />
          </Button>
        )}
      </div>
    </div>
  );
}

