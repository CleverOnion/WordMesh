/**
 * 仪表板页面（登录后的主页面）
 */

import { ProfileCard } from '@/modules/auth';

export default function DashboardPage() {
  return (
    <div className="container mx-auto py-8">
      <div className="space-y-6">
        <div>
          <h1 className="text-3xl font-bold">欢迎使用 WordMesh</h1>
          <p className="text-muted-foreground">
            开始构建您的单词知识网络
          </p>
        </div>

        <div className="grid gap-6 md:grid-cols-2 lg:grid-cols-3">
          <ProfileCard />
        </div>
      </div>
    </div>
  );
}


