use std::env; // 引入标准库的 env 模块，用于读取命令行参数。
use std::ffi::OsStr;
use std::fs; // 引入标准库的 fs 模块，用于读取文件和目录。
use std::io; // 引入标准库的 io 模块，用于表示文件系统操作的成功或失败。
use std::path::Path; // 引入 Path 类型，用于安全地表示文件或目录路径。
use std::process::ExitCode; // 引入 ExitCode 类型，用于向操作系统返回程序退出状态。 // 引入 OsStr，用来直接比较操作系统中的文件名，不要求文件名必须是 UTF-8。

const IGNORED_DIRECTORY_NAMES: [&str; 2] = [".git", "target"]; // 定义默认忽略的目录名称；数组长度是 2。

#[derive(Default)] // 自动为结构体生成 Default，使所有 u64 字段默认初始化为 0。
struct ScanSummary {
    // 定义一个结构体，把相关的扫描统计数据组合到一起。
    file_count: u64,              // 保存扫描到的普通文件总数。
    directory_count: u64,         // 保存扫描到的子目录总数，不包含用户传入的根目录。
    other_count: u64,             // 保存符号链接等其他类型项目的总数。
    total_bytes: u64,             // 保存所有普通文件的总字节数。
    ignored_directory_count: u64, // 保存因为命中过滤规则而没有进入扫描的目录数量。
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
                    // Ok 中现在包含一个 ScanSummary 类型的扫描结果。
                    println!("扫描目录：{}", path.display()); // 输出本次扫描的根目录。
                    println!("文件数量：{}", summary.file_count); // 输出所有层级的文件总数。
                    println!("子目录数量：{}", summary.directory_count); // 输出所有层级的子目录总数。
                    println!(
                        // 使用多行形式调用 println!，避免这一行代码过长。
                        "忽略目录数量：{}", // 定义需要输出的文字，并用 {} 接收后面的数值。
                        summary.ignored_directory_count  // 把忽略目录数量填入前面的 {}。
                    ); // 输出语句结束。
                    println!("其他项目数量：{}", summary.other_count); // 输出符号链接等其他项目数量。
                    println!("文件总字节数：{}", summary.total_bytes); // 输出全部普通文件占用的总字节数。
                    ExitCode::SUCCESS // 扫描成功，向操作系统返回退出码 0。
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
    visit_directory(path, &mut summary)?; // 递归访问根目录，并允许函数修改统计结果。
    Ok(summary) // 扫描完成，把最终统计结果返回给 main 函数。
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
            let metadata = entry.metadata()?; // 读取文件元数据，其中包含文件大小。
            summary.file_count += 1; // 普通文件总数增加 1。
            summary.total_bytes += metadata.len(); // 把当前文件的字节数加入总字节数。
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
