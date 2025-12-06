/**
 * 设置抽屉
 * 包装设置功能
 */

'use client';

import { FloatingDrawer } from './FloatingDrawer';
import { ProfileCard } from '@/modules/auth';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';

interface SettingsDrawerProps {
  isOpen: boolean;
  onClose: () => void;
}

export function SettingsDrawer({ isOpen, onClose }: SettingsDrawerProps) {
  return (
    <FloatingDrawer
      isOpen={isOpen}
      onClose={onClose}
      direction="right"
      title="设置"
      width="w-full max-w-2xl"
    >
      <div className="space-y-6">
        <div className="grid gap-6 md:grid-cols-1">
          {/* 用户信息 */}
          <Card>
            <CardHeader>
              <CardTitle>账户信息</CardTitle>
              <CardDescription>查看和管理您的账户信息</CardDescription>
            </CardHeader>
            <CardContent>
              <ProfileCard />
            </CardContent>
          </Card>

          {/* 应用设置 */}
          <Card>
            <CardHeader>
              <CardTitle>应用设置</CardTitle>
              <CardDescription>自定义应用行为</CardDescription>
            </CardHeader>
            <CardContent>
              <div className="space-y-4">
                <p className="text-sm text-muted-foreground">
                  更多设置功能即将推出...
                </p>
              </div>
            </CardContent>
          </Card>
        </div>
      </div>
    </FloatingDrawer>
  );
}

