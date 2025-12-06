/**
 * 设置页面
 */

'use client';

import { ProfileCard } from '@/modules/auth';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Settings } from 'lucide-react';

export default function SettingsPage() {
  return (
    <div className="space-y-6">
      {/* 页面标题 */}
      <div>
        <h1 className="text-3xl font-bold flex items-center gap-2">
          <Settings className="size-8" />
          设置
        </h1>
        <p className="text-muted-foreground mt-2">
          管理您的账户和偏好设置
        </p>
      </div>

      <div className="grid gap-6 md:grid-cols-2">
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
  );
}



