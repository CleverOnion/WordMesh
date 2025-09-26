# WordMesh API 文档（鉴权与健康检查）

本项目采用统一响应结构，所有接口均返回 HTTP 200 状态码，通过业务码 `code` 标识成功或错误。

- 成功：`code = 2000`，`data` 为具体数据
- 失败：`code != 2000`，`message` 为错误文案，`data` 为空或包含字段级错误

统一响应字段：`{ code, message, data, traceId, timestamp }`

## 认证相关

### 注册

- 方法：POST
- 路径：`/api/v1/auth/register`
- 请求体：

```json
{
  "username": "user_123",
  "password": "password123"
}
```

- 成功响应：

```json
{
  "code": 2000,
  "message": "OK",
  "data": {
    "id": 1,
    "username": "user_123",
    "created_at": "2025-01-04T12:34:56Z"
  },
  "traceId": "...",
  "timestamp": 1735970000000
}
```

- 失败示例（参数校验失败，字段级错误在 data 中返回）：

```json
{
  "code": 4001,
  "message": "参数校验失败",
  "data": [{ "field": "username", "message": "用户名长度必须在 3 到 32 之间" }],
  "traceId": "...",
  "timestamp": 1735970000000
}
```

示例 cURL：

```bash
curl -sS http://127.0.0.1:8080/api/v1/auth/register \
  -H 'Content-Type: application/json' \
  -d '{"username":"user_123","password":"password123"}'
```

### 登录

- 方法：POST
- 路径：`/api/v1/auth/login`
- 请求体：

```json
{ "username": "user_123", "password": "password123" }
```

- 成功响应：

```json
{
  "code": 2000,
  "message": "OK",
  "data": { "access_token": "<JWT>", "refresh_token": "<JWT or null>" },
  "traceId": "...",
  "timestamp": 1735970000000
}
```

- 失败示例（凭证无效）：

```json
{
  "code": 4011,
  "message": "Invalid credentials",
  "data": null,
  "traceId": "...",
  "timestamp": 1735970000000
}
```

示例 cURL：

```bash
curl -sS http://127.0.0.1:8080/api/v1/auth/login \
  -H 'Content-Type: application/json' \
  -d '{"username":"user_123","password":"password123"}'
```

### 刷新令牌

- 方法：POST
- 路径：`/api/v1/auth/refresh`
- 请求体：

```json
{ "refresh_token": "<JWT>" }
```

- 成功响应：与登录相同结构，返回新的 `access_token`，可能返回新的 `refresh_token`（视配置而定）。

示例 cURL：

```bash
curl -sS http://127.0.0.1:8080/api/v1/auth/refresh \
  -H 'Content-Type: application/json' \
  -d '{"refresh_token":"<JWT>"}'
```

### 获取当前用户资料

- 方法：GET
- 路径：`/api/v1/auth/profile`
- 鉴权：需要在请求头携带 `Authorization: Bearer <access_token>`
- 成功响应：同“注册”接口的 Profile 数据结构。

示例 cURL：

```bash
curl -sS http://127.0.0.1:8080/api/v1/auth/profile \
  -H 'Authorization: Bearer <ACCESS_TOKEN>'
```

## 健康检查

- 方法：GET
- 路径：`/api/v1/health`
- 成功响应：

```json
{
  "code": 2000,
  "message": "OK",
  "data": {
    "status": "healthy",
    "service": "WordMesh Backend",
    "version": "0.1.0"
  },
  "traceId": "...",
  "timestamp": 1735970000000
}
```

## 错误码对照

| 代码 | 说明                                     |
| ---- | ---------------------------------------- |
| 2000 | 成功                                     |
| 4000 | 业务错误（通用）                         |
| 4001 | 参数校验失败（字段级错误在 data 数组中） |
| 4010 | 鉴权错误（通用）                         |
| 4011 | 无效凭证（用户名不存在或密码错误）       |
| 4012 | 访问令牌过期                             |
| 4013 | 令牌无效                                 |
| 4014 | 刷新功能被禁用                           |
| 5000 | 内部服务错误                             |

备注：服务统一返回 HTTP 200，请以 `code` 判定业务成功与否。
