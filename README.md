# Axum Backend Service

基于 Rust Axum 框架开发的后端服务，提供用户认证和管理功能。

## 功能特性

- 用户注册与邮箱验证
- 用户登录与 JWT 认证
- 密码重置
- 用户管理（仅管理员）
- 数据库迁移
- 异步邮件发送

## 技术栈

- **Web 框架**: Axum
- **数据库**: PostgreSQL
- **ORM**: SQLx
- **认证**: JWT (JSON Web Tokens)
- **邮件服务**: SMTP
- **配置管理**: dotenv
- **日志**: tracing

## 环境要求

- Rust 1.75+
- PostgreSQL 17+
- Docker & Docker Compose

## 快速开始

1. 克隆项目

```bash
git clone <repository-url>
cd axum_back
```

2. 配置环境变量

```bash
cp .env.example .env
```

编辑 `.env` 文件，设置以下环境变量：

SMTP 邮箱配置参考：[SMTP 邮箱配置](https://www.cnblogs.com/jiyuwu/p/16313476.html)

```env
DATABASE_URL=postgresql://postgres:postgres@localhost:5432/axum_db
JWT_SECRET_KEY=your-secret-key
JWT_MAXAGE=60
SERVER_PORT=8000
SMTP_USERNAME=your-email@example.com
SMTP_PASSWORD=your-email-password
SMTP_SERVER=smtp.example.com
SMTP_PORT=587
```

3. 启动数据库

```bash
docker compose up -d
```

1. 运行数据库迁移

```bash
sqlx database create
sqlx migrate run
```

1. 启动服务

```bash
cargo run
```

## API 接口

### 认证相关

#### 用户注册

- 路径: `POST /api/auth/register`
- 请求体:

```json
{
    "name": "用户名",
    "email": "email@example.com",
    "password": "密码",
    "password_confirm": "确认密码"
}
```

#### 用户登录

- 路径: `POST /api/auth/login`
- 请求体:

```json
{
    "email": "email@example.com",
    "password": "密码"
}
```

#### 邮箱验证

- 路径: `GET /api/auth/verify?token=verification_token`

#### 忘记密码

- 路径: `POST /api/auth/forgot-password`
- 请求体:

```json
{
    "email": "email@example.com"
}
```

#### 重置密码

- 路径: `POST /api/auth/reset-password`
- 请求体:

```json
{
    "token": "reset_token",
    "password": "新密码"
}
```

### 用户管理

#### 获取用户列表（需要管理员权限）

- 路径: `GET /api/users?page=1&limit=10`

#### 更新用户角色（需要管理员权限）

- 路径: `PATCH /api/users/{user_id}/role`
- 请求体:

```json
{
    "role": "admin"
}
```

## 开发指南

### 项目结构

```bash
src/
├── config/         -- 配置管理
├── db/            -- 数据库操作
├── dtos/          -- 数据传输对象
├── error/         -- 错误处理
├── handlers/      -- 请求处理器
├── mail/          -- 邮件服务
├── middleware/    -- 中间件
├── models/        -- 数据模型
├── routes/        -- 路由定义
└── utils/         -- 工具函数
```

### 添加新功能

1. 在 `migrations` 目录添加数据库迁移文件
2. 在 `models` 目录定义数据模型
3. 在 `db` 目录实现数据库操作
4. 在 `handlers` 目录实现业务逻辑
5. 在 `routes` 目录注册路由

## 测试

运行所有测试：

```bash
cargo test
```

## 部署

1. 构建发布版本：

```bash
cargo build --release
```

2. 使用 Docker 部署：

```bash
docker compose -f docker-compose.prod.yml up -d
```

## 贡献指南

1. Fork 项目
2. 创建特性分支
3. 提交改动
4. 推送到分支
5. 创建 Pull Request

## 许可证

[MIT License](LICENSE)
