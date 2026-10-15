use std::env; // 引入标准库的 env 模块，用于读取命令行参数。
use std::ffi::OsStr; // 引入 OsStr，用于直接比较操作系统中的文件名。
use std::fs; // 引入标准库的 fs 模块，用于读取文件和目录。
use std::io; // 引入标准库的 io 模块，用于表示文件系统操作的成功或失败。
use std::path::{Path, PathBuf}; // 引入借用路径 Path，以及拥有路径所有权的 PathBuf。
use std::process::ExitCode; // 引入 ExitCode 类型，用于向操作系统返回程序退出状态。

const IGNORED_DIRECTORY_NAMES: [&str; 2] = [".git", "target"]; // 定义默认忽略的目录名称；数组长度是 2。
const DISPLAY_FILE_LIMIT: usize = 10; // 最多在终端中显示 10 个文件，避免大量输出刷屏。

#[derive(Debug)] // 自动生成调试输出能力，方便以后检查结构体内容。
struct FileInfo {
    // 表示扫描到的一个普通文件。
    path: PathBuf, // 保存文件完整路径；PathBuf 拥有路径数据的所有权。
    size: u64,     // 保存文件大小，单位是字节。
} // FileInfo 结构体定义结束。

#[derive(Default)] // 自动生成默认值；数字是 0，Vec 是空数组。
struct ScanSummary {
    // 保存一次目录扫描产生的所有结果。
    files: Vec<FileInfo>,         // 保存扫描到的全部普通文件信息。
    file_count: u64,              // 保存扫描到的普通文件总数。
    directory_count: u64,         // 保存发现的子目录总数。
    ignored_directory_count: u64, // 保存没有进入扫描的目录数量。
    other_count: u64,             // 保存符号链接等其他项目数量。
    total_bytes: u64,             // 保存所有普通文件的总字节数。
} // ScanSummary 结构体定义结束。

fn main() -> ExitCode {
    // main 是程序入口，并返回一个退出码。
    let mut args = env::args(); //env::args() 返回的不是普通数组，而是一个迭代器。迭代器可以使用 next() 依次取出内容。

    let program_name = args // 开始从命令行参数迭代器中取得程序名称。
        .next() //从参数迭代器中取出第一个参数。通常是程序自身的路径。
        //next() 的返回类型不是直接的 String，而是：Option<String>
        //因为迭代器可能有下一个值，也可能没有，所以 Rust 用 Option 表达这两种情况：
        //Some(字符串) // 成功取得参数
        //None         // 没有取得参数
        .unwrap_or_else(|| String::from("local-file-batch-tool")); // 如果没有取得程序名称，就使用默认名称。
    //如果有值就取出它；如果没有值，就执行括号中的函数产生默认值。

    match args.next().as_deref() {
        // 从参数中取得用户输入的命令，并根据命令进入不同分支。
        //.as_deref()，把：Option<String>临时转换为：Option<&str>
        None | Some("help") | Some("-h") | Some("--help") => {
            // 没有输入命令，或者输入了帮助参数时，进入这个分支。
            print_help(&program_name); //调用后面定义的 print_help 函数。
            ExitCode::SUCCESS //返回成功退出码。
        }

        Some("scan") => {
            // 用户输入 scan 命令时，进入目录扫描分支。
            let Some(path) = args.next() else {
                // 尝试取得 scan 命令后面的目录参数。
                eprintln!("错误：scan 命令缺少目录参数"); // 把参数错误输出到标准错误流。
                eprintln!("用法：{program_name} scan <目录>"); // 告诉用户 scan 命令的正确写法。
                return ExitCode::from(2); // 参数使用错误，返回退出码 2，并立即结束程序。
            }; // let-else 语句到这里结束。

            let path = Path::new(&path); // 把 String 形式的路径转换为 Path 引用。

            match scan_directory(path) {
                // 调用目录扫描函数，并处理函数返回的成功或失败结果。
                Ok(summary) => {
                    // Ok 中包含完整的 ScanSummary 扫描结果。
                    print_scan_summary(path, &summary); // 借用路径和扫描结果，并统一负责输出。
                    ExitCode::SUCCESS // 扫描成功，返回退出码 0。
                }

                Err(error) => {
                    // Err 表示目录扫描过程中发生了错误。
                    eprintln!("扫描失败：{error}"); // 将具体错误信息输出到标准错误流。
                    ExitCode::from(1) // 运行过程中失败，向操作系统返回退出码 1。
                }
            } // 扫描结果的 match 表达式到这里结束。
        }

        Some(command) => {
            // 如果参数不是已支持的命令，就会进入这个分支。
            eprintln!("错误：未知命令 `{command}`"); // 输出用户输入的未知命令。
            print_help(&program_name); // 输出帮助信息，让用户看到可用命令。
            ExitCode::from(2) // 命令使用错误，返回退出码 2。
        } //如果参数不是：
          //- 没有参数；
          //- help；
          //- -h；
          //- --help；
          //- scan；
          //就会进入这个分支
    } // 最外层的命令匹配到这里结束。
} // main 函数到这里结束。

fn scan_directory(path: &Path) -> io::Result<ScanSummary> {
    // 扫描成功时返回 ScanSummary，失败时返回 io::Error。
    if !path.is_dir() {
        // 如果路径不存在或者不是目录，就进入这个分支。
        let message = format!("`{}` 不是有效目录", path.display()); // 生成包含具体路径的错误信息。
        let error = io::Error::new(io::ErrorKind::InvalidInput, message); // 创建输入无效类型的错误。
        return Err(error); // 立即把错误返回给 main 函数。
    } // 路径检查结束。

    let mut summary = ScanSummary::default(); // 创建统计结果，所有计数初始值都是 0。
    visit_directory(path, &mut summary)?; // 递归扫描根目录，并填写统计结果。
    summary
        // 访问扫描结果中的文件清单。
        .files
        // 取得可变的 Vec<FileInfo>。
        .sort_by(|left, right| left.path.cmp(&right.path)); // 按文件路径排序，使每次输出顺序稳定。
    Ok(summary) // 把排序后的完整扫描结果返回给 main。
} // scan_directory 函数结束。

fn visit_directory(path: &Path, summary: &mut ScanSummary) -> io::Result<()> {
    // 递归访问一个目录；&mut 表示允许修改传入的统计结果。
    let entries = fs::read_dir(path)?; // 读取当前目录；失败时通过 ? 立即返回错误。

    for entry_result in entries {
        // 逐个处理当前目录中的项目。
        let entry = entry_result?; // 取得目录项目；读取失败时立即返回错误。
        let file_type = entry.file_type()?; // 取得项目类型；读取失败时立即返回错误。

        if file_type.is_file() {
            // 如果当前项目是普通文件，就进入这个分支。
            let metadata = entry.metadata()?; // 读取文件元数据；失败时通过 ? 返回错误。
            let file_size = metadata.len(); // 从元数据中取得文件大小，单位是字节。
            let file_path = entry.path(); // 取得文件完整路径，并获得这个 PathBuf 的所有权。

            let file_info = FileInfo {
                // 创建一个 FileInfo，把同一文件的路径和大小组合起来。
                path: file_path, // 把文件路径移动到 FileInfo 中。
                size: file_size, // 把文件大小保存到 FileInfo 中。
            }; // FileInfo 创建结束。

            summary.file_count += 1; // 普通文件总数增加 1。
            summary.total_bytes += file_size; // 把当前文件大小加入总字节数。
            summary.files.push(file_info); // 把当前文件信息追加到文件清单末尾。
        } else if file_type.is_dir() {
            // 如果当前项目是子目录，就进入这个分支。
            summary.directory_count += 1; // 已发现的子目录总数增加 1。
            let child_path = entry.path(); // 取得当前子目录的完整路径。

            if should_ignore_directory(&child_path) {
                // 判断当前子目录名称是否命中了默认忽略规则。
                summary.ignored_directory_count += 1; // 被忽略的目录总数增加 1。
                continue; // 跳过本轮循环，不进入这个目录，继续处理下一个项目。
            } // 忽略规则判断结束。

            visit_directory(&child_path, summary)?; // 没有被忽略，因此递归扫描这个子目录。
        } else {
            // 如果不是普通文件也不是目录，就进入这个分支。
            summary.other_count += 1; // 其他类型项目总数增加 1。
        } // 当前项目的类型判断结束。
    } // 当前目录的所有项目处理结束。

    Ok(()) // 表示当前目录及其所有子目录均扫描成功。
} // visit_directory 函数结束。

fn should_ignore_directory(path: &Path) -> bool {
    // 判断给定路径的最后一部分是否属于默认忽略目录。
    let Some(directory_name) = path.file_name() else {
        // file_name() 返回 Option，因为某些特殊路径可能没有文件名部分。
        return false; // 没有文件名时不执行忽略，返回 false。
    }; // let-else 语句结束。

    IGNORED_DIRECTORY_NAMES
        // 取得默认忽略目录名称数组。
        .iter()
        // 创建数组迭代器，逐个检查忽略名称。
        .any(|ignored_name| directory_name == OsStr::new(*ignored_name))
    // 只要存在一个相同名称就返回 true，否则返回 false。
} // should_ignore_directory 函数结束。

fn print_scan_summary(root: &Path, summary: &ScanSummary) {
    // 只借用扫描结果，不取得所有权，也不修改扫描结果。
    println!("扫描目录：{}", root.display()); // 输出本次扫描的根目录。
    println!("文件数量：{}", summary.file_count); // 输出普通文件总数。
    println!("子目录数量：{}", summary.directory_count); // 输出发现的子目录总数。
    println!(
        // 使用多行形式输出忽略目录数量。
        "忽略目录数量：{}",              // {} 用来接收后面的数值。
        summary.ignored_directory_count  // 提供被忽略的目录数量。
    ); // 忽略目录数量输出结束。
    println!("其他项目数量：{}", summary.other_count); // 输出其他类型项目数量。
    println!("文件总字节数：{}", summary.total_bytes); // 输出普通文件的总字节数。
    println!(); // 输出一个空行，将统计数据和文件清单分开。

    let displayed_count = summary.files.len().min(DISPLAY_FILE_LIMIT); // 取文件数量和显示上限中的较小值。
    println!("前 {displayed_count} 个文件："); // 告诉用户接下来会显示多少个文件。

    for file in summary.files.iter().take(DISPLAY_FILE_LIMIT) {
        // 借用文件清单，并最多遍历 DISPLAY_FILE_LIMIT 个元素。
        println!("  {}（{} 字节）", file.path.display(), file.size); // 输出一个文件的路径和大小。
    } // 文件清单输出循环结束。

    if summary.files.len() > DISPLAY_FILE_LIMIT {
        // 如果实际文件数量超过显示上限，就进入这个分支。
        let hidden_count = summary.files.len() - DISPLAY_FILE_LIMIT; // 计算没有显示的文件数量。
        println!("  还有 {hidden_count} 个文件未显示"); // 告诉用户还有多少文件被省略。
    } // 文件数量上限判断结束。
} // print_scan_summary 函数结束。

fn print_help(program_name: &str) {
    // 定义帮助函数；&str 表示借用一段字符串，不取得字符串所有权。
    println!("本地文件批处理工具"); // 输出工具名称。
    println!(); // 输出一个空行，使帮助内容更容易阅读。
    println!("用法："); // 输出用法标题。
    println!("  {program_name} <命令>"); // 输出程序的基本命令格式。
    println!(); // 输出一个空行。
    println!("命令："); // 输出命令列表标题。
    println!("  help           显示帮助"); // 说明 help 命令的作用。
    println!("  scan <目录>    扫描指定目录"); // 说明 scan 命令及其目录参数。
    println!(); // 输出一个空行。
    println!("选项："); // 输出选项列表标题。
    println!("  -h, --help     显示帮助"); // 说明两个帮助选项。
} // print_help 函数到这里结束。
