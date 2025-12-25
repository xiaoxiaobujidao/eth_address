# Docker 镜像构建配置说明

本项目使用 GitHub Actions 自动构建 Docker 镜像并推送到 GitHub Container Registry (ghcr.io)。

## 🎯 优势

使用 GitHub Container Registry 的优势：
- ✅ **无需额外配置**：使用 GitHub 自带的 `GITHUB_TOKEN`，无需创建额外的 secrets
- ✅ **权限自动管理**：与 GitHub 仓库权限集成
- ✅ **无限制的公开镜像**：公开镜像完全免费
- ✅ **与代码同步**：镜像和代码在同一个平台管理

## 🚀 使用步骤

### 1. 推送代码触发构建

无需任何额外配置！只需要推送代码到 GitHub：

```bash
git add .
git commit -m "Add Docker support"
git push origin main
```

GitHub Actions 会自动：
1. 构建多架构 Docker 镜像（amd64, arm64）
2. 推送到 GitHub Container Registry
3. 自动打标签

### 2. 发布版本

创建标签可以生成版本化的镜像：

```bash
# 创建版本标签
git tag -a v1.0.0 -m "Release version 1.0.0"
git push origin v1.0.0
```

这会生成以下镜像标签：
- `ghcr.io/xiaoxiaobujidao/eth_address:latest`
- `ghcr.io/xiaoxiaobujidao/eth_address:v1.0.0`
- `ghcr.io/xiaoxiaobujidao/eth_address:v1.0`
- `ghcr.io/xiaoxiaobujidao/eth_address:v1`

### 3. 设置镜像可见性（可选）

默认情况下，镜像可能是私有的。要设置为公开：

1. 进入 GitHub 仓库页面
2. 点击右侧的 "Packages" 链接
3. 选择你的镜像包
4. 点击 "Package settings"
5. 滚动到底部，选择 "Change visibility" → "Public"

或者通过命令行设置：
```bash
# 使用 GitHub CLI
gh api \
  --method PATCH \
  -H "Accept: application/vnd.github+json" \
  /user/packages/container/eth_address/visibility \
  -f visibility='public'
```

## 📦 使用镜像

### 拉取镜像

对于公开镜像，无需登录即可拉取：

```bash
docker pull ghcr.io/xiaoxiaobujidao/eth_address:latest
```

对于私有镜像，需要先登录：

```bash
# 使用 Personal Access Token 登录
echo $GITHUB_TOKEN | docker login ghcr.io -u xiaoxiaobujidao --password-stdin

# 或使用 GitHub CLI
gh auth token | docker login ghcr.io -u xiaoxiaobujidao --password-stdin
```

### 运行容器

```bash
# 基本使用
docker run -v $(pwd):/app/output ghcr.io/xiaoxiaobujidao/eth_address:latest \
  --min-repeats 6 --threads 16

# 持续生成模式
docker run -v $(pwd):/app/output ghcr.io/xiaoxiaobujidao/eth_address:latest \
  --min-repeats 6 --continuous --count 5

# 使用特定版本
docker run -v $(pwd):/app/output ghcr.io/xiaoxiaobujidao/eth_address:v1.0.0 \
  --min-repeats 6
```

## 🔧 高级配置

### 手动触发构建

1. 进入 GitHub 仓库的 `Actions` 页面
2. 选择 "Build and Push Docker Image" workflow
3. 点击 "Run workflow" 按钮
4. 选择分支并运行

### 查看构建日志

1. 进入 GitHub 仓库的 `Actions` 页面
2. 点击具体的 workflow 运行记录
3. 查看详细的构建日志

### 查看所有镜像版本

```bash
# 使用 GitHub API
curl -H "Authorization: token $GITHUB_TOKEN" \
  https://api.github.com/users/xiaoxiaobujidao/packages/container/eth_address/versions

# 或在 GitHub 网页查看
# https://github.com/xiaoxiaobujidao/eth_address/pkgs/container/eth_address
```

## 🏗️ Workflow 说明

### 触发条件

- 推送到 `main` 或 `master` 分支
- 创建以 `v` 开头的标签（如 `v1.0.0`）
- 提交 Pull Request
- 手动触发

### 构建平台

支持以下平台：
- `linux/amd64` - Intel/AMD 64位
- `linux/arm64` - ARM 64位（Apple Silicon, Raspberry Pi 等）

### 构建缓存

使用 GitHub Actions 缓存来加速构建：
- Rust 依赖缓存
- Docker 层缓存

## 📊 镜像标签策略

| 触发方式 | 生成的标签 | 示例 |
|---------|-----------|------|
| 推送到 main | `latest`, `main` | `ghcr.io/user/repo:latest` |
| 推送到分支 | 分支名 | `ghcr.io/user/repo:dev` |
| 创建标签 v1.2.3 | `v1.2.3`, `v1.2`, `v1` | `ghcr.io/user/repo:v1.2.3` |
| 任何提交 | `分支名-SHA` | `ghcr.io/user/repo:main-abc1234` |
| Pull Request | `pr-123` | `ghcr.io/user/repo:pr-123` |

## 🔒 安全说明

- `GITHUB_TOKEN` 由 GitHub Actions 自动提供，具有临时性和作用域限制
- 镜像推送权限由仓库的 `packages: write` 权限控制
- 建议将敏感镜像设置为私有

## 🐛 故障排除

### 问题：推送镜像失败

**解决方案**：
1. 检查仓库的 Actions 权限设置
2. 进入 `Settings` > `Actions` > `General`
3. 确保 "Workflow permissions" 设置为 "Read and write permissions"

### 问题：无法拉取镜像

**解决方案**：
1. 确认镜像是公开的
2. 如果是私有镜像，使用正确的认证方式
3. 检查镜像名称是否正确（必须全部小写）

### 问题：构建超时

**解决方案**：
1. 优化 Dockerfile，利用好构建缓存
2. 考虑减少构建平台数量
3. 如果是免费账户，注意构建时间限制

## 📚 参考资料

- [GitHub Container Registry 文档](https://docs.github.com/en/packages/working-with-a-github-packages-registry/working-with-the-container-registry)
- [GitHub Actions 文档](https://docs.github.com/en/actions)
- [Docker Build Push Action](https://github.com/docker/build-push-action)

