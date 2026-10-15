use crate::scanner::FileInfo; // 从 scanner 模块引入文件信息类型。
use sha2::{Digest, Sha256}; // 引入 SHA-256 算法和通用哈希接口。
use std::collections::BTreeMap; // 引入有序映射，用于文件分组。
use std::fs::File; // 引入 File，用于打开候选文件。
use std::io::{self, BufReader, Read}; // 引入错误类型、缓冲读取器和读取接口。
use std::path::Path; // 引入借用路径类型。

pub fn group_files_by_size(files: &[FileInfo]) -> BTreeMap<u64, Vec<&FileInfo>> {
    // 按文件大小分组，只返回至少有两个文件的候选组。
    let mut groups: BTreeMap<u64, Vec<&FileInfo>> = BTreeMap::new();
    // 创建“文件大小 → 文件清单”的空映射。

    for file in files {
        // 逐个借用文件信息。
        groups
            // 访问大小分组映射。
            .entry(file.size)
            // 查找当前文件大小对应的分组。
            .or_default()
            // 分组不存在时创建空 Vec。
            .push(file);
        // 将当前文件加入对应的大小分组。
    } // 文件大小分组循环结束。

    groups.retain(|_size, same_size_files| same_size_files.len() > 1);
    // 删除只有一个文件的分组，因为它不可能存在重复副本。

    groups
    // 返回只包含重复候选的大小分组。
} // group_files_by_size 函数结束。

fn calculate_sha256(path: &Path) -> io::Result<String> {
    // 打开指定文件并计算 SHA-256。
    let file = File::open(path)?;
    // 打开磁盘文件；失败时立即返回错误。
    let reader = BufReader::new(file);
    // 使用缓冲读取器包装文件。
    calculate_sha256_from_reader(reader)
    // 调用通用读取器哈希函数。
} // calculate_sha256 函数结束。

fn calculate_sha256_from_reader(mut reader: impl Read) -> io::Result<String> {
    // 接收任何实现 Read 接口的数据来源并计算 SHA-256。
    let mut hasher = Sha256::new();
    // 创建 SHA-256 计算器。
    let mut buffer = [0_u8; 64 * 1024];
    // 创建 64 KiB 的固定大小缓冲区。

    loop {
        // 循环分块读取数据。
        let bytes_read = reader.read(&mut buffer)?;
        // 读取一块数据，并取得实际读取字节数。

        if bytes_read == 0 {
            // 读取到 0 字节表示到达末尾。
            break;
            // 结束读取循环。
        } // 数据末尾判断结束。

        hasher.update(&buffer[..bytes_read]);
        // 将本次实际读取的字节加入哈希计算。
    } // 数据读取循环结束。

    let digest = hasher.finalize();
    // 完成哈希计算，得到 32 字节结果。
    let mut hash_text = String::with_capacity(64);
    // 为 64 个十六进制字符预先分配空间。

    for byte in digest {
        // 逐个处理哈希字节。
        let byte_text = format!("{byte:02x}");
        // 将字节转换为两位小写十六进制文本。
        hash_text.push_str(&byte_text);
        // 将文本追加到最终哈希字符串。
    } // 十六进制转换循环结束。

    Ok(hash_text)
    // 返回 SHA-256 十六进制字符串。
} // calculate_sha256_from_reader 函数结束。

pub fn find_duplicate_groups(
    // 定义可以由其他模块调用的重复文件查找函数。
    files: &[FileInfo],
    // 借用扫描得到的文件清单。
) -> io::Result<BTreeMap<(u64, String), Vec<&FileInfo>>> {
    // 返回“文件大小和哈希 → 重复文件清单”的映射。
    let size_candidates = group_files_by_size(files);
    // 先筛选大小相同的候选文件。

    let mut hash_groups: BTreeMap<(u64, String), Vec<&FileInfo>> = BTreeMap::new();
    // 创建“文件大小和哈希 → 文件清单”的空映射。

    for (file_size, same_size_files) in size_candidates {
        // 逐个处理大小相同的候选组。
        for file in same_size_files {
            // 逐个处理当前候选组中的文件。
            let hash = calculate_sha256(&file.path)?;
            // 读取文件内容并计算 SHA-256。

            hash_groups
                // 访问哈希分组映射。
                .entry((file_size, hash))
                // 查找当前大小和哈希对应的分组。
                .or_default()
                // 分组不存在时创建空 Vec。
                .push(file);
            // 将当前文件加入对应的哈希分组。
        } // 当前大小候选组处理结束。
    } // 所有大小候选组处理结束。

    hash_groups.retain(|_key, duplicate_files| duplicate_files.len() > 1);
    // 删除只有一个文件的哈希分组。

    Ok(hash_groups)
    // 返回确认后的重复文件分组。
} // find_duplicate_groups 函数结束。

#[cfg(test)] // 只有执行 cargo test 时才编译测试模块。
mod tests {
    // 定义 duplicates 模块的单元测试。
    use super::*; // 引入 duplicates 模块中的函数和类型。
    use std::path::PathBuf; // 引入测试数据所需的拥有型路径。

    #[test] // 标记为自动化测试。
    fn groups_files_with_the_same_size() {
        // 验证只有大小重复的文件会被保留。
        let files = vec![
            // 创建内存中的模拟文件清单。
            FileInfo {
                // 创建第一个模拟文件。
                path: PathBuf::from("a.txt"), // 设置文件路径。
                size: 12,                     // 设置文件大小。
            },
            FileInfo {
                // 创建第二个模拟文件。
                path: PathBuf::from("b.txt"), // 设置文件路径。
                size: 12,                     // 设置相同的文件大小。
            },
            FileInfo {
                // 创建第三个模拟文件。
                path: PathBuf::from("c.txt"), // 设置文件路径。
                size: 28,                     // 设置不同的文件大小。
            },
        ]; // 测试文件清单创建结束。

        let groups = group_files_by_size(&files);
        // 调用实际的大小分组函数。

        assert_eq!(groups.len(), 1);
        // 应该只保留一个重复大小分组。

        let twelve_byte_group = groups
            // 访问分组结果。
            .get(&12)
            // 查找 12 字节分组。
            .expect("应该存在 12 字节的候选分组");
        // 分组不存在时让测试失败。

        assert_eq!(twelve_byte_group.len(), 2);
        // 12 字节分组应该包含两个文件。

        assert_eq!(twelve_byte_group[0].path.as_path(), Path::new("a.txt"));
        // 验证第一个文件路径。
        assert_eq!(twelve_byte_group[1].path.as_path(), Path::new("b.txt"));
        // 验证第二个文件路径。
        assert!(!groups.contains_key(&28));
        // 单独存在的 28 字节文件不应该保留。
    } // 大小分组测试结束。

    #[test] // 标记为自动化测试。
    fn calculates_known_sha256() {
        // 验证已知文本的 SHA-256 结果。
        let input = io::Cursor::new(b"hello world");
        // 将内存字节包装成实现 Read 的读取器。

        let hash = calculate_sha256_from_reader(input)
            // 调用真实的哈希核心逻辑。
            .expect("内存数据的 SHA-256 计算应该成功");
        // 哈希失败时让测试失败。

        assert_eq!(
            // 比较实际结果和标准结果。
            hash,
            // hello world 对应的已知 SHA-256。
            "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9"
        );
        // 两个哈希不相等时测试失败。
    } // SHA-256 测试结束。
} // tests 模块结束。
