/**
 * 动画网格背景组件
 * 提供动态网格背景效果，支持主题切换
 */

'use client';

import { useEffect, useRef } from 'react';
import { cn } from '@/lib/utils';

interface AnimatedGridBackgroundProps {
  className?: string;
  gridSize?: number;
  lineColor?: string;
  animationSpeed?: number;
}

export function AnimatedGridBackground({
  className,
  gridSize = 50,
  lineColor,
  animationSpeed = 20,
}: AnimatedGridBackgroundProps) {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const animationFrameRef = useRef<number>();
  const offsetRef = useRef({ x: 0, y: 0 });

  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;

    const ctx = canvas.getContext('2d');
    if (!ctx) return;

    const resizeCanvas = () => {
      canvas.width = window.innerWidth;
      canvas.height = window.innerHeight;
    };

    resizeCanvas();
    window.addEventListener('resize', resizeCanvas);

    const drawGrid = () => {
      if (!ctx) return;

      // 清除画布
      ctx.clearRect(0, 0, canvas.width, canvas.height);

      // 获取计算后的颜色
      const computedStyle = getComputedStyle(document.documentElement);
      const bgColor = computedStyle.getPropertyValue('--background').trim();
      const borderColor = lineColor || computedStyle.getPropertyValue('--border').trim();
      
      // 解析 HSL 颜色
      const bgMatch = bgColor.match(/hsl\((\d+)\s+(\d+)%\s+(\d+)%\)/);
      const borderMatch = borderColor.match(/hsl\((\d+)\s+(\d+)%\s+(\d+)%\)/);
      
      // 设置网格线颜色（更明显）
      if (borderMatch) {
        const [, h, s, l] = borderMatch;
        // 根据主题调整透明度
        const isDark = document.documentElement.classList.contains('dark');
        const opacity = isDark ? 0.4 : 0.3;
        ctx.strokeStyle = `hsla(${h}, ${s}%, ${l}%, ${opacity})`;
      } else {
        ctx.strokeStyle = 'rgba(0, 0, 0, 0.25)';
      }

      ctx.lineWidth = 1.5;

      // 绘制垂直线
      for (let x = offsetRef.current.x % gridSize; x < canvas.width; x += gridSize) {
        ctx.beginPath();
        ctx.moveTo(x, 0);
        ctx.lineTo(x, canvas.height);
        ctx.stroke();
      }

      // 绘制水平线
      for (let y = offsetRef.current.y % gridSize; y < canvas.height; y += gridSize) {
        ctx.beginPath();
        ctx.moveTo(0, y);
        ctx.lineTo(canvas.width, y);
        ctx.stroke();
      }

      // 缓慢移动网格（创建动画效果）
      offsetRef.current.x += 0.05;
      offsetRef.current.y += 0.05;

      // 限制偏移值，避免数值过大
      if (offsetRef.current.x >= gridSize) offsetRef.current.x = 0;
      if (offsetRef.current.y >= gridSize) offsetRef.current.y = 0;
    };

    const animate = () => {
      drawGrid();
      animationFrameRef.current = requestAnimationFrame(animate);
    };

    animate();

    return () => {
      window.removeEventListener('resize', resizeCanvas);
      if (animationFrameRef.current) {
        cancelAnimationFrame(animationFrameRef.current);
      }
    };
  }, [gridSize, lineColor, animationSpeed]);

  return (
    <canvas
      ref={canvasRef}
      className={cn('fixed inset-0 pointer-events-none z-0', className)}
      style={{ 
        willChange: 'transform',
      }}
    />
  );
}

