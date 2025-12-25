use rayon::prelude::*;
use secp256k1::{PublicKey, Secp256k1, SecretKey};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use clap::Parser;
use rand::RngCore;
use sha3::{Digest, Keccak256};
use std::fs::OpenOptions;
use std::io::Write;

#[derive(Parser)]
#[command(name = "eth-vanity")]
#[command(about = "极限性能以太坊靓号生成器")]
struct Args {
    /// 最小重复字符位数（默认8位）
    #[arg(short = 'c', long, default_value = "8")]
    min_repeats: usize,
    
    /// 线程数量（默认为CPU核心数）
    #[arg(short = 't', long)]
    threads: Option<usize>,
    
    /// 批处理大小（每次检查多少个地址）
    #[arg(short = 'b', long, default_value = "1000")]
    batch_size: usize,
    
    /// 显示统计信息的间隔（秒）
    #[arg(long, default_value = "3")]
    stats_interval: u64,
    
    /// 输出文件路径（结果将保存到此文件）
    #[arg(short = 'o', long, default_value = "eth_address.txt")]
    output: String,
    
    /// 要生成的靓号数量（0或不指定表示无限制）
    #[arg(short = 'l', long)]
    count: Option<usize>,
}

/// 优化的重复字符检查函数
#[inline(always)]
fn has_repeating_suffix_optimized(address: &[u8; 40], min_repeats: usize) -> Option<(u8, usize)> {
    let len = 40;
    
    // 从末尾开始检查
    for start in (0..len).rev() {
        let current_char = address[start];
        
        // 检查是否是有效的十六进制字符 (0-9, a-f)
        if !(current_char >= b'0' && current_char <= b'9') && !(current_char >= b'a' && current_char <= b'f') {
            continue;
        }
        
        let mut count = 1;
        let mut pos = start + 1;
        
        // 向后计算连续相同字符的数量
        while pos < len && address[pos] == current_char {
            count += 1;
            pos += 1;
        }
        
        // 如果连续字符达到要求且位于地址末尾
        if count >= min_repeats && pos == len {
            return Some((current_char, count));
        }
    }
    
    None
}

/// 优化的地址生成函数
#[inline(always)]
fn generate_address_batch(batch_size: usize) -> Vec<(SecretKey, [u8; 40])> {
    let secp = Secp256k1::new();
    let mut rng = rand::thread_rng();
    let mut results = Vec::with_capacity(batch_size);
    
    for _ in 0..batch_size {
        // 生成32字节随机数
        let mut secret_bytes = [0u8; 32];
        rng.fill_bytes(&mut secret_bytes);
        
        let secret_key = match SecretKey::from_slice(&secret_bytes) {
            Ok(key) => key,
            Err(_) => continue, // 跳过无效的密钥
        };
        
        let public_key = PublicKey::from_secret_key(&secp, &secret_key);
        let public_key_bytes = public_key.serialize_uncompressed();
        
        // 计算Keccak256哈希
        let mut hasher = Keccak256::new();
        hasher.update(&public_key_bytes[1..]);
        let hash = hasher.finalize();
        
        // 转换为十六进制字符串（直接操作字节）
        let mut address = [0u8; 40];
        for (i, &byte) in hash[12..].iter().enumerate() {
            let high = (byte >> 4) & 0x0f;
            let low = byte & 0x0f;
            address[i * 2] = if high < 10 { b'0' + high } else { b'a' + high - 10 };
            address[i * 2 + 1] = if low < 10 { b'0' + low } else { b'a' + low - 10 };
        }
        
        results.push((secret_key, address));
    }
    
    results
}

fn worker_optimized(
    found: Arc<AtomicBool>,
    counter: Arc<AtomicU64>,
    min_repeats: usize,
    batch_size: usize,
) -> Option<(String, String, u8, usize)> {
    while !found.load(Ordering::Relaxed) {
        let batch = generate_address_batch(batch_size);
        counter.fetch_add(batch.len() as u64, Ordering::Relaxed);
        
        for (secret_key, address) in batch {
            if let Some((char, count)) = has_repeating_suffix_optimized(&address, min_repeats) {
                found.store(true, Ordering::Relaxed);
                let private_key = hex::encode(secret_key.secret_bytes());
                let address_str = String::from_utf8_lossy(&address).to_string();
                return Some((private_key, address_str, char, count));
            }
        }
    }
    None
}

fn format_number(n: u64) -> String {
    let s = n.to_string();
    let mut result = String::new();
    let chars: Vec<char> = s.chars().rev().collect();
    
    for (i, c) in chars.iter().enumerate() {
        if i > 0 && i % 3 == 0 {
            result.push(',');
        }
        result.push(*c);
    }
    
    result.chars().rev().collect()
}

/// 保存结果到文件
fn save_to_file(filename: &str, address: &str, private_key: &str, character: u8, count: usize, attempts: u64, elapsed: f64) -> Result<(), std::io::Error> {
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(filename)?;
    
    let timestamp = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC");
    
    writeln!(file, "=== 以太坊靓号地址 ===")?;
    writeln!(file, "时间: {}", timestamp)?;
    writeln!(file, "地址: 0x{}", address)?;
    writeln!(file, "私钥: {}", private_key)?;
    writeln!(file, "重复字符: '{}' 连续 {} 位", character as char, count)?;
    writeln!(file, "尝试次数: {}", format_number(attempts))?;
    writeln!(file, "用时: {:.2} 秒", elapsed)?;
    writeln!(file, "平均速度: {:.0} 次/秒", attempts as f64 / elapsed)?;
    writeln!(file, "")?;
    
    Ok(())
}

fn main() {
    let args = Args::parse();
    
    let min_repeats = args.min_repeats;
    // 使用CPU核心数作为默认线程数
    let thread_count = args.threads.unwrap_or_else(|| num_cpus::get());
    let batch_size = args.batch_size;
    let output_file = args.output;
    let target_count = args.count.unwrap_or(0);
    let target_count = if target_count == 0 { usize::MAX } else { target_count };
    
    // 验证参数
    if min_repeats < 3 {
        eprintln!("错误：最小重复位数不能少于3位");
        std::process::exit(1);
    }
    
    if min_repeats > 15 {
        eprintln!("警告：{}位重复字符极难找到，可能需要非常长的时间", min_repeats);
    }
    
    println!("🚀 极限性能以太坊靓号生成器");
    println!("📋 搜索条件: 后缀连续重复字符 >= {} 位", min_repeats);
    println!("🧵 线程数: {}", thread_count);
    println!("📦 批处理大小: {}", batch_size);
    println!("📁 输出文件: {}", output_file);
    if target_count == usize::MAX {
        println!("🔄 生成模式: 无限制");
    } else {
        println!("🔄 生成模式: {} 个靓号", target_count);
    }
    println!();
    
    let mut found_count = 0;
    let global_start_time = Instant::now();
    
    while found_count < target_count {
        let found = Arc::new(AtomicBool::new(false));
        let counter = Arc::new(AtomicU64::new(0));
        let start_time = Instant::now();
        
        // 启动统计线程
        let stats_counter = counter.clone();
        let stats_found = found.clone();
        let stats_start_time = start_time;
        let stats_interval = args.stats_interval;
        let stats_handle = std::thread::spawn(move || {
            while !stats_found.load(Ordering::Relaxed) {
                std::thread::sleep(Duration::from_secs(stats_interval));
                let count = stats_counter.load(Ordering::Relaxed);
                let elapsed = stats_start_time.elapsed().as_secs_f64();
                let rate = count as f64 / elapsed;
                println!("📊 已尝试: {} 次 | 速度: {:.0} 次/秒", format_number(count), rate);
            }
        });
        
        // 使用rayon并行处理
        let result = (0..thread_count)
            .into_par_iter()
            .map(|_| worker_optimized(found.clone(), counter.clone(), min_repeats, batch_size))
            .find_any(|result| result.is_some())
            .flatten();
        
        // 等待统计线程结束
        let _ = stats_handle.join();
        
        let elapsed = start_time.elapsed();
        let total_attempts = counter.load(Ordering::Relaxed);
        
        match result {
            Some((private_key, address, digit, count)) => {
                found_count += 1;
                
                println!();
                println!("🎉 找到第 {} 个靓号！", found_count);
                println!("📍 地址: 0x{}", address);
                println!("🔢 重复数字: '{}' 连续 {} 位", digit as char, count);
                println!("🔑 私钥: {}", private_key);
                println!("⏱️  用时: {:.2} 秒", elapsed.as_secs_f64());
                println!("🔢 尝试次数: {}", format_number(total_attempts));
                println!("⚡ 平均速度: {:.0} 次/秒", total_attempts as f64 / elapsed.as_secs_f64());
                
                // 保存到文件
                match save_to_file(&output_file, &address, &private_key, digit, count, total_attempts, elapsed.as_secs_f64()) {
                    Ok(_) => println!("💾 结果已保存到: {}", output_file),
                    Err(e) => eprintln!("❌ 保存文件失败: {}", e),
                }
                
                println!();
                println!("⚠️  请妥善保管私钥，不要泄露给任何人！");
                
                if found_count < target_count {
                    println!();
                    println!("🔄 继续查找下一个靓号... ({}/{})", found_count, if target_count == usize::MAX { "∞".to_string() } else { target_count.to_string() });
                    println!();
                }
            }
            None => {
                println!("❌ 未找到符合条件的地址");
                break;
            }
        }
    }
    
    if found_count > 0 {
        let total_elapsed = global_start_time.elapsed();
        println!();
        println!("🏁 生成完成！");
        println!("📊 总计生成: {} 个靓号", found_count);
        println!("⏱️  总用时: {:.2} 秒", total_elapsed.as_secs_f64());
        println!("📁 所有结果已保存到: {}", output_file);
    }
}
