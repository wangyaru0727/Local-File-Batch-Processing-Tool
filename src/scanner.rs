use std::ffi::OsStr; // 引入 OsStr，用于比较操作系统中的目录名称。
use std::fs; // 引入文件系统模块，用于读取目录和文件元数据。
use std::io; // 引入 I/O 模块，用于返回文件系统错误。
use std::path::{Path, PathBuf}; // 引入借用路径 Path 和拥有路径的 PathBuf。

const IGNORED_DIRECTORY_NAMES: [&str; 2] = [".git", "target"];
// 定义扫描时默认跳过的目录名称。

#[derive(Debug)] // 自动生成调试输出能力。
pub struct FileInfo {
    // 定义一个能够被其他模块使用的文件信息结构体。
    pub path: PathBuf, // 保存文件路径，并允许其他模块读取。
    pub size: u64,     // 保存文件大小，并允许其他模块读取。
} // FileInfo 定义结束。

#[derive(Default)] // 自动生成所有字段的默认值。
pub struct ScanSummary {
    // 定义一个能够被其他模块使用的扫描结果。
    pub files: Vec<FileInfo>,         // 保存扫描到的所有普通文件。
    pub file_count: u64,              // 保存普通文件总数。
    pub directory_count: u64,         // 保存发现的子目录总数。
    pub ignored_directory_count: u64, // 保存跳过的目录总数。
    pub other_count: u64,             // 保存其他类型项目总数。
    pub total_bytes: u64,             // 保存所有普通文件的总字节数。
} // ScanSummary 定义结束。

pub fn scan_directory(path: &Path) -> io::Result<ScanSummary> {
    // 扫描指定目录，并允许 main 模块调用这个函数。
    if !path.is_dir() {
        // 如果路径不存在或者不是目录，就进入这个分支。
        let message = format!("`{}` 不是有效目录", path.display());
        // 生成包含具体路径的错误信息。
        let error = io::Error::new(io::ErrorKind::InvalidInput, message);
        // 创建“输入无效”类型的 I/O 错误。
        return Err(error);
        // 立即将错误返回给调用者。
    } // 路径有效性判断结束。

    let mut summary = ScanSummary::default();
    // 创建所有计数均为零的扫描结果。
    visit_directory(path, &mut summary)?;
    // 递归扫描目录，并修改扫描结果。
    summary
        // 访问文件清单。
        .files
        // 取得可变的 Vec<FileInfo>。
        .sort_by(|left, right| left.path.cmp(&right.path));
    // 按路径排序，使每次扫描的输出顺序稳定。

    Ok(summary)
    // 返回完整的扫描结果。
} // scan_directory 函数结束。

fn visit_directory(path: &Path, summary: &mut ScanSummary) -> io::Result<()> {
    // 递归访问一个目录，并修改同一个扫描结果。
    let entries = fs::read_dir(path)?;
    // 读取当前目录；失败时立即返回错误。

    for entry_result in entries {
        // 逐个处理当前目录中的项目。
        let entry = entry_result?;
        // 取得一个目录项目；失败时立即返回错误。
        let file_type = entry.file_type()?;
        // 取得项目类型；失败时立即返回错误。

        if file_type.is_file() {
            // 如果当前项目是普通文件，就进入这个分支。
            let metadata = entry.metadata()?;
            // 读取文件元数据。
            let file_size = metadata.len();
            // 从元数据中取得文件大小。
            let file_path = entry.path();
            // 取得文件的完整路径。

            let file_info = FileInfo {
                // 将同一个文件的路径和大小组合起来。
                path: file_path, // 保存文件路径。
                size: file_size, // 保存文件大小。
            }; // FileInfo 创建结束。

            summary.file_count += 1;
            // 普通文件总数增加 1。
            summary.total_bytes += file_size;
            // 将文件大小加入总字节数。
            summary.files.push(file_info);
            // 将文件信息加入文件清单。
        } else if file_type.is_dir() {
            // 如果当前项目是目录，就进入这个分支。
            summary.directory_count += 1;
            // 发现的子目录数量增加 1。
            let child_path = entry.path();
            // 取得子目录完整路径。

            if should_ignore_directory(&child_path) {
                // 判断当前目录是否应该跳过。
                summary.ignored_directory_count += 1;
                // 跳过的目录数量增加 1。
                continue;
                // 不进入当前目录，继续处理下一个项目。
            } // 忽略目录判断结束。

            visit_directory(&child_path, summary)?;
            // 递归进入没有被忽略的子目录。
        } else {
            // 如果不是普通文件或目录，就进入这个分支。
            summary.other_count += 1;
            // 其他类型项目数量增加 1。
        } // 文件类型判断结束。
    } // 当前目录遍历结束。

    Ok(())
    // 表示当前目录及其子目录均扫描成功。
} // visit_directory 函数结束。

fn should_ignore_directory(path: &Path) -> bool {
    // 判断目录名称是否出现在默认忽略列表中。
    let Some(directory_name) = path.file_name() else {
        // 尝试取得路径最后一部分的目录名称。
        return false;
        // 没有目录名称时不跳过。
    }; // let-else 语句结束。

    IGNORED_DIRECTORY_NAMES
        // 访问默认忽略目录数组。
        .iter()
        // 逐个遍历忽略目录名称。
        .any(|ignored_name| directory_name == OsStr::new(*ignored_name))
    // 只要有一个名称相同就返回 true。
} // should_ignore_directory 函数结束。
