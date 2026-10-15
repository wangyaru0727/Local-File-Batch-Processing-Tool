use std::env; // 引入标准库的 env 模块，用于读取命令行参数。
use std::fs; // 引入标准库的 fs 模块，用于读取文件和目录。
use std::io; // 引入标准库的 io 模块，用于表示文件系统操作的成功或失败。
use std::path::Path; // 引入 Path 类型，用于安全地表示文件或目录路径。
use std::process::ExitCode; // 引入 ExitCode 类型，用于向操作系统返回程序退出状态。

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
                Ok(()) => {
                    // Ok 表示目录扫描成功。
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

fn scan_directory(path: &Path) -> io::Result<()> {
    // 定义目录扫描函数；成功返回 Ok(())，失败返回 io::Error。
    if !path.is_dir() {
        // 如果路径不存在，或者它不是一个目录，就进入这个分支。
        let message = format!("`{}` 不是有效目录", path.display()); // 创建包含具体路径的错误信息。
        let error = io::Error::new(io::ErrorKind::InvalidInput, message); // 创建“输入参数无效”类型的错误。
        return Err(error); // 将错误返回给调用 scan_directory 的代码。
    } // 路径有效性检查到这里结束。

    let mut file_count: u64 = 0; // 创建可变计数器，用来记录普通文件数量。
    let mut directory_count: u64 = 0; // 创建可变计数器，用来记录子目录数量。
    let mut other_count: u64 = 0; // 创建可变计数器，用来记录符号链接等其他项目。

    let entries = fs::read_dir(path)?; // 读取目录内容；如果读取失败，? 会立即返回错误。

    for entry_result in entries {
        // 使用 for 循环，逐个处理目录第一层中的项目。
        let entry = entry_result?; // 取得一个目录项目；如果取得失败，? 会立即返回错误。
        let file_type = entry.file_type()?; // 读取项目类型；如果读取失败，? 会立即返回错误。

        if file_type.is_file() {
            // 如果当前项目是普通文件，就进入这个分支。
            file_count += 1; // 将普通文件数量增加 1。
        } else if file_type.is_dir() {
            // 如果当前项目是子目录，就进入这个分支。
            directory_count += 1; // 将子目录数量增加 1。
        } else {
            // 如果既不是普通文件，也不是目录，就进入这个分支。
            other_count += 1; // 将其他类型项目的数量增加 1。
        } // 当前项目的类型判断到这里结束。
    } // 所有第一层目录项目处理完毕后，循环结束。

    println!("扫描目录：{}", path.display()); // 输出本次扫描的目录路径。
    println!("文件数量：{file_count}"); // 输出统计到的普通文件数量。
    println!("子目录数量：{directory_count}"); // 输出统计到的子目录数量。
    println!("其他项目数量：{other_count}"); // 输出统计到的其他类型项目数量。

    Ok(()) // 表示目录扫描成功完成，没有需要返回的具体数据。
} // scan_directory 函数到这里结束。

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
