/**
 * 用户资料卡片组件
 */

'use client';

import { Button } from '@/components/ui/button';
import { useAuth } from '../hooks/useAuth';
import { formatDateTime } from '@/shared/utils/format';

export function ProfileCard() {
  const { user, logout, isLoading } = useAuth();

  if (isLoading || !user) {
    return (
      <div className="rounded-lg border p-4">
        <div className="text-sm text-muted-foreground">加载中...</div>
      </div>
    );
  }

  return (
    <div className="rounded-lg border p-4 space-y-4">
      <div>
        <h3 className="text-lg font-semibold">用户资料</h3>
      </div>

      <div className="space-y-2">
        <div>
          <div className="text-sm text-muted-foreground">用户名</div>
          <div className="text-base font-medium">{user.username}</div>
        </div>

        <div>
          <div className="text-sm text-muted-foreground">用户 ID</div>
          <div className="text-base font-medium">{user.id}</div>
        </div>

        <div>
          <div className="text-sm text-muted-foreground">注册时间</div>
          <div className="text-base font-medium">
            {formatDateTime(user.created_at)}
          </div>
        </div>
      </div>

      <Button
        variant="destructive"
        onClick={logout}
        className="w-full"
        disabled={isLoading}
      >
        退出登录
      </Button>
    </div>
  );
}

