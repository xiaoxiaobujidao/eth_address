# 以太坊靓号地址生成器

极限性能的以太坊靓号地址生成器，使用 Rust 编写，支持多线程并行生成。

## 功能特性

- 🚀 极高性能：多线程并行计算，充分利用 CPU
- 🎯 自定义要求：可指定最小重复字符位数
- 💾 自动保存：找到的靓号自动保存到文件
- 🔄 持续模式：支持连续生成多个靓号
- 📊 实时统计：显示尝试次数和生成速度

## 使用方法

### 本地构建运行

```bash
# 构建
cargo build --release

# 运行（默认8位重复字符）
./target/release/eth_address

# 指定参数
./target/release/eth_address -c 6 -t 16

# 生成5个靓号
./target/release/eth_address -c 6 -l 5

# 无限制生成（默认）
./target/release/eth_address -c 6
```

### Docker 使用

```bash
# 拉取镜像（从 GitHub Container Registry）
docker pull ghcr.io/xiaoxiaobujidao/eth_address:latest

# 运行（结果将保存在当前目录）
docker run -v $(pwd):/app/output ghcr.io/xiaoxiaobujidao/eth_address:latest -c 6

# 生成5个靓号
docker run -v $(pwd):/app/output ghcr.io/xiaoxiaobujidao/eth_address:latest \
  -c 6 -l 5

# 指定线程数和输出文件
docker run -v $(pwd):/app/output ghcr.io/xiaoxiaobujidao/eth_address:latest \
  -c 6 -t 16 -o my_addresses.txt
```

## 参数说明

- `-c, --min-repeats <NUM>`: 最小重复字符位数（默认8位）
- `-t, --threads <NUM>`: 线程数量（默认为CPU核心数）
- `-l, --count <NUM>`: 生成靓号数量（默认不限制，0表示无限制）
- `-o, --output <FILE>`: 输出文件路径（默认 eth_address.txt）
- `-b, --batch-size <NUM>`: 批处理大小（默认1000）
- `--stats-interval <SECS>`: 统计信息显示间隔（秒）

## 示例

生成6位重复字符的靓号：
```bash
docker run -v $(pwd):/app/output ghcr.io/xiaoxiaobujidao/eth_address:latest \
  -c 6 -t 16
```

生成10个8位重复字符的靓号：
```bash
docker run -v $(pwd):/app/output ghcr.io/xiaoxiaobujidao/eth_address:latest \
  -c 8 -l 10
```

## 安全警告

⚠️ **请妥善保管生成的私钥，不要泄露给任何人！**

生成的私钥具有完全的资产控制权，一旦泄露将导致资产损失。

## License

MIT

