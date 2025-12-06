/**
 * 验证工具函数
 */

// 验证用户名格式
export function validateUsername(username: string): boolean {
  // 用户名长度必须在 3 到 32 之间
  return username.length >= 3 && username.length <= 32;
}

// 验证密码格式
export function validatePassword(password: string): boolean {
  // 密码长度至少 6 位
  return password.length >= 6;
}

// 验证邮箱格式（如果将来需要）
export function validateEmail(email: string): boolean {
  const emailRegex = /^[^\s@]+@[^\s@]+\.[^\s@]+$/;
  return emailRegex.test(email);
}

// 验证单词文本格式
export function validateWordText(text: string): boolean {
  // 单词文本不能为空，且长度不超过 255
  return text.trim().length > 0 && text.length <= 255;
}

// 验证义项定义格式
export function validateSenseDefinition(definition: string): boolean {
  // 义项定义不能为空
  return definition.trim().length > 0;
}

