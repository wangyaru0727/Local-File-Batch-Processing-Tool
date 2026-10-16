use local_file_batch_tool::directory_watcher::watch_directory;
// 从项目的库层引入目录监控函数。

use local_file_batch_tool::duplicates::{find_duplicate_groups, group_files_by_size};
// 从项目的库层引入重复文件检测函数。

use local_file_batch_tool::image_compression::{
    // 从项目的库层引入图片压缩功能。
    compress_jpeg,
    // 引入单张 JPEG 压缩函数。
    compress_jpeg_directory,
    // 引入并发目录批量压缩函数。
};

use local_file_batch_tool::scanner::{FileInfo, ScanSummary, scan_directory};
// 从项目的库层引入扫描函数和扫描结果类型。
use std::env; // 引入命令行参数功能。
use std::io; // 引入 I/O 结果类型。
use std::path::Path; // 引入借用路径类型。
use std::process::ExitCode; // 引入程序退出码。 // 引入单张 JPEG 压缩函数。
// 引入持续目录监控函数。

const DISPLAY_FILE_LIMIT: usize = 10; // 最多在终端中显示 10 个文件，避免大量输出刷屏。

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

        Some(command @ ("scan" | "duplicates")) => {
            // scan 和 duplicates 都需要目录参数，因此共用这个命令分支。
            let Some(path) = args.next() else {
                // 尝试取得命令后面的目录参数。
                eprintln!("错误：{command} 命令缺少目录参数");
                // 输出具体是哪个命令缺少参数。
                eprintln!("用法：{program_name} {command} <目录>");
                // 根据当前命令输出正确用法。
                return ExitCode::from(2);
                // 参数使用错误，返回退出码 2。
            }; // let-else 语句结束。

            let path = Path::new(&path);
            // 将 String 路径转换成文件系统使用的 Path 引用。
            let should_find_duplicates = command == "duplicates";
            // duplicates 命令需要计算哈希，scan 命令不需要。

            match scan_directory(path) {
                // 两个命令都先扫描目录，取得文件清单。
                Ok(summary) => {
                    // 目录扫描成功，取得完整的 ScanSummary。
                    print_scan_summary(path, &summary);
                    // 输出扫描统计、文件清单和大小相同的候选文件。

                    if !should_find_duplicates {
                        // 如果当前执行的是 scan，就不读取文件内容、不计算哈希。
                        return ExitCode::SUCCESS;
                        // 快速扫描已经完成，返回成功退出码 0。
                    } // scan 快速结束判断结束。

                    match print_confirmed_duplicates(&summary.files) {
                        // duplicates 命令继续计算候选文件的 SHA-256。
                        Ok(()) => {
                            // 所有候选文件均成功读取并完成哈希计算。
                            ExitCode::SUCCESS // 返回成功退出码 0。
                        }

                        Err(error) => {
                            // 某个候选文件打开或读取失败时进入这个分支。
                            eprintln!("重复文件检测失败：{error}");
                            // 输出文件读取或哈希计算错误。
                            ExitCode::from(1) // 返回运行错误退出码 1。
                        }
                    } // SHA-256 检测结果处理结束。
                }

                Err(error) => {
                    // 目录不存在、无法读取或者扫描失败时进入这个分支。
                    eprintln!("扫描失败：{error}");
                    // 输出具体的目录扫描错误。
                    ExitCode::from(1) // 返回运行错误退出码 1。
                }
            } // 目录扫描结果处理结束。
        }
        Some("compress") => {
            // 用户输入 compress 时进入单张 JPEG 压缩分支。
            let Some(input_text) = args.next() else {
                // 尝试取得输入 JPEG 路径。
                eprintln!("错误：compress 命令缺少输入文件");
                // 输出缺少输入路径的错误。
                eprintln!("用法：{program_name} compress <输入.jpg> <输出.jpg> [质量]");
                // 输出完整命令格式。
                return ExitCode::from(2);
                // 参数使用错误，返回退出码 2。
            }; // 输入路径检查结束。

            let Some(output_text) = args.next() else {
                // 尝试取得输出 JPEG 路径。
                eprintln!("错误：compress 命令缺少输出文件");
                // 输出缺少输出路径的错误。
                eprintln!("用法：{program_name} compress <输入.jpg> <输出.jpg> [质量]");
                // 输出完整命令格式。
                return ExitCode::from(2);
                // 参数使用错误，返回退出码 2。
            }; // 输出路径检查结束。

            let quality = match args.next() {
                // 尝试取得可选的质量参数。
                Some(quality_text) => {
                    // 用户明确提供了质量参数。
                    match quality_text.parse::<u8>() {
                        // 尝试把字符串转换成 u8。
                        Ok(value) if (1..=100).contains(&value) => value,
                        // 转换成功且位于合法范围时使用该值。
                        _ => {
                            // 转换失败或数值超出范围时进入这里。
                            eprintln!("错误：JPEG 质量必须是 1 到 100 的整数");
                            // 输出质量参数错误。
                            return ExitCode::from(2);
                            // 参数使用错误，返回退出码 2。
                        }
                    } // 质量字符串转换结束。
                }

                None => 80,
                // 没有提供质量参数时使用默认质量 80。
            }; // 质量参数处理结束。

            let input = Path::new(&input_text);
            // 将输入路径字符串转换为 Path。
            let output = Path::new(&output_text);
            // 将输出路径字符串转换为 Path。

            match compress_jpeg(input, output, quality) {
                // 调用 JPEG 压缩函数并处理结果。
                Ok(result) => {
                    // 图片成功解码、编码并写入输出文件。
                    println!("JPEG 压缩完成");
                    // 输出成功提示。
                    println!("输入文件：{}", input.display());
                    // 输出输入路径。
                    println!("输出文件：{}", output.display());
                    // 输出输出路径。
                    println!("JPEG 质量：{quality}");
                    // 输出实际使用的质量。
                    println!("原始大小：{} 字节", result.original_bytes);
                    // 输出原始文件大小。
                    println!("压缩后大小：{} 字节", result.compressed_bytes);
                    // 输出压缩后文件大小。

                    if result.compressed_bytes < result.original_bytes {
                        // 如果输出文件确实比原文件小，就进入这个分支。
                        let saved_bytes = result.original_bytes - result.compressed_bytes;
                        // 计算实际减少的字节数。
                        println!("减少空间：{saved_bytes} 字节");
                        // 输出节省的空间。
                    } else {
                        // 如果重新编码后没有变小，就进入这个分支。
                        let increased_bytes = result.compressed_bytes - result.original_bytes;
                        // 计算文件增加的字节数。
                        println!("文件没有变小，反而增加了 {increased_bytes} 字节");
                        // 明确告诉用户本次压缩没有收益。
                    } // 压缩收益判断结束。

                    ExitCode::SUCCESS
                    // 压缩成功，返回退出码 0。
                }

                Err(error) => {
                    // 解码、编码或写入失败时进入这个分支。
                    eprintln!("JPEG 压缩失败：{error}");
                    // 输出具体错误。
                    ExitCode::from(1)
                    // 返回运行错误退出码 1。
                }
            } // 图片压缩结果处理结束。
        }
        Some("compress-dir") => {
            // 用户输入 compress-dir 时进入目录批量压缩分支。
            let Some(input_text) = args.next() else {
                // 尝试取得输入目录。
                eprintln!("错误：compress-dir 缺少输入目录");
                // 输出参数错误。
                eprintln!("用法：{program_name} compress-dir <输入目录> <输出目录>");
                // 输出正确用法。
                return ExitCode::from(2);
                // 返回参数错误退出码。
            }; // 输入目录检查结束。

            let Some(output_text) = args.next() else {
                // 尝试取得输出目录。
                eprintln!("错误：compress-dir 缺少输出目录");
                // 输出参数错误。
                eprintln!("用法：{program_name} compress-dir <输入目录> <输出目录>");
                // 输出正确用法。
                return ExitCode::from(2);
                // 返回参数错误退出码。
            }; // 输出目录检查结束。

            let input_directory = Path::new(&input_text);
            // 将输入目录字符串转换为 Path。
            let output_directory = Path::new(&output_text);
            // 将输出目录字符串转换为 Path。
            let quality = 80;
            // 当前阶段固定使用 JPEG 质量 80。
            let default_workers = std::thread::available_parallelism()
                // 查询操作系统提供的逻辑处理器数量。
                .map(|count| count.get())
                // 将 NonZeroUsize 转换成普通 usize。
                .unwrap_or(1)
                // 无法取得处理器数量时退回单线程。
                .min(8);
            // 默认最多使用 8 个线程，避免图片解码占用过多内存。

            let worker_count = match args.next() {
                // 尝试读取可选的线程数参数。
                Some(worker_text) => {
                    // 用户提供了线程数。
                    match worker_text.parse::<usize>() {
                        // 尝试把字符串转换为 usize。
                        Ok(value) if (1..=64).contains(&value) => value,
                        // 转换成功且范围合法时使用该值。
                        _ => {
                            // 转换失败或范围非法时进入这里。
                            eprintln!("错误：线程数必须是 1 到 64 的整数");
                            // 输出线程数错误。
                            return ExitCode::from(2);
                            // 返回参数错误退出码。
                        }
                    } // 线程数字符串转换结束。
                }

                None => default_workers,
                // 没有提供线程数时使用自动计算的默认值。
            }; // 线程数参数处理结束。
            match compress_jpeg_directory(
                // 调用并发批量压缩函数。
                input_directory,
                // 传入输入目录。
                output_directory,
                // 传入输出目录。
                quality,
                // 传入固定质量 80。
                worker_count,
                // 传入实际工作线程数量。
            ) {
                // 执行单线程目录批量压缩。
                Ok(result) => {
                    // 批量任务成功运行到结束。
                    println!("JPEG 批量压缩完成");
                    println!("工作线程：{worker_count}");
                    // 输出本次批量任务实际使用的线程数量。
                    // 输出完成提示。
                    println!("发现文件：{}", result.discovered_count);
                    // 输出发现的 JPEG 数量。
                    println!("成功文件：{}", result.successful_count);
                    // 输出成功压缩数量。
                    println!("失败文件：{}", result.failures.len());
                    // 输出失败数量。
                    println!("原始总大小：{} 字节", result.original_bytes);
                    // 输出成功文件原始总大小。
                    println!("压缩后总大小：{} 字节", result.compressed_bytes);
                    // 输出成功文件输出总大小。

                    if result.compressed_bytes < result.original_bytes {
                        // 如果批量输出总大小更小，就进入这里。
                        let saved_bytes = result.original_bytes - result.compressed_bytes;
                        // 计算总计减少的字节数。
                        println!("总计减少：{saved_bytes} 字节");
                        // 输出总计节省空间。
                    } // 批量压缩收益判断结束。

                    for failure in result.failures {
                        // 逐个输出失败文件，不隐藏局部错误。
                        eprintln!(
                            // 使用标准错误流输出失败记录。
                            "失败：{}：{}",
                            // 定义失败输出格式。
                            failure.input_path.display(),
                            // 输出失败文件路径。
                            failure.message // 输出具体错误信息。
                        ); // 当前失败记录输出结束。
                    } // 失败记录输出循环结束。

                    ExitCode::SUCCESS
                    // 批量任务完成，返回成功退出码。
                }

                Err(error) => {
                    // 输入目录、输出目录或目录读取发生严重错误。
                    eprintln!("JPEG 批量压缩失败：{error}");
                    // 输出严重错误。
                    ExitCode::from(1)
                    // 返回运行错误退出码。
                }
            } // 批量压缩结果处理结束。
        }
        Some("watch") => {
            // 用户输入 watch 时进入目录监控分支。
            let Some(path_text) = args.next() else {
                // 尝试取得需要监控的目录路径。
                eprintln!("错误：watch 命令缺少目录参数");
                // 输出参数错误。
                eprintln!("用法：{program_name} watch <目录>");
                // 输出正确命令格式。
                return ExitCode::from(2);
                // 返回参数错误退出码。
            }; // 目录参数检查结束。

            let path = Path::new(&path_text);
            // 将路径字符串转换成 Path 引用。

            match watch_directory(path) {
                // 启动持续目录监控。
                Ok(()) => {
                    // 监控通道正常结束时进入这里。
                    ExitCode::SUCCESS
                    // 返回成功退出码。
                }

                Err(error) => {
                    // 监控器创建或目录注册失败时进入这里。
                    eprintln!("目录监控失败：{error}");
                    // 输出具体错误。
                    ExitCode::from(1)
                    // 返回运行错误退出码。
                }
            } // 目录监控结果处理结束。
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
    println!(); // 输出空行，将统计数据和文件清单分开。

    let displayed_count = summary.files.len().min(DISPLAY_FILE_LIMIT);
    // 取实际文件数量与显示上限中的较小值。
    println!("前 {displayed_count} 个文件："); // 输出即将显示的文件数量。

    for file in summary.files.iter().take(DISPLAY_FILE_LIMIT) {
        // 逐个借用文件信息，并且最多遍历 DISPLAY_FILE_LIMIT 个文件。
        println!("  {}（{} 字节）", file.path.display(), file.size);
        // 输出当前文件的路径和大小。
    } // 文件清单输出循环结束。

    if summary.files.len() > DISPLAY_FILE_LIMIT {
        // 如果实际文件数量超过显示上限，就进入这个分支。
        let hidden_count = summary.files.len() - DISPLAY_FILE_LIMIT;
        // 计算没有显示的文件数量。
        println!("  还有 {hidden_count} 个文件未显示");
        // 输出没有显示的文件数量。
    } // 文件数量上限判断结束。

    print_size_candidates(&summary.files);
    // 只在这里调用一次候选分组输出函数。
} // print_scan_summary 函数结束。

fn print_size_candidates(files: &[FileInfo]) {
    // 接收完整文件清单，输出大小相同的重复候选。
    let candidate_groups = group_files_by_size(files);
    // 调用分组函数，取得只包含重复候选的分组。

    println!(); // 输出空行，将文件清单和候选分组分开。
    println!("按文件大小筛选重复候选："); // 输出候选结果标题。

    if candidate_groups.is_empty() {
        // 如果没有任何至少包含两个文件的大小分组，就进入这个分支。
        println!("  没有发现大小相同的文件"); // 告诉用户当前没有重复候选。
        return; // 提前结束函数，避免继续执行后面的输出逻辑。
    } // 空候选判断结束。

    let candidate_file_count: usize = candidate_groups
        // 访问所有候选分组。
        .values()
        // 创建只遍历每个分组值的迭代器。
        .map(Vec::len)
        // 将每个 Vec 转换成其中的文件数量。
        .sum();
    // 把所有分组的文件数量相加。

    println!("  候选分组数量：{}", candidate_groups.len());
    // 输出包含相同文件大小的分组数量。
    println!("  候选文件数量：{candidate_file_count}");
    // 输出所有候选分组中的文件总数。

    for (file_size, same_size_files) in candidate_groups {
        // 逐个取得文件大小，以及具有该大小的文件清单。
        println!(); // 在每个候选分组前输出空行。
        println!(
            // 输出当前分组的基本信息。
            "  {file_size} 字节，共 {} 个文件：",
            // file_size 是这个分组共同的文件大小。
            same_size_files.len() // 输出当前分组中的文件数量。
        ); // 分组标题输出结束。

        for file in same_size_files {
            // 逐个借用当前大小分组中的文件信息。
            println!("    {}", file.path.display());
            // 输出候选文件的路径。
        } // 当前候选分组的文件路径输出结束。
    } // 所有候选分组输出结束。
} // print_size_candidates 函数结束。

fn print_confirmed_duplicates(files: &[FileInfo]) -> io::Result<()> {
    // 计算并输出 SHA-256 相同的重复文件。
    let duplicate_groups = find_duplicate_groups(files)?;
    // 取得确认后的重复文件分组；读取失败时立即返回错误。

    println!(); // 输出空行，分隔大小候选结果和最终结果。
    println!("SHA-256 确认结果："); // 输出最终重复检测标题。

    if duplicate_groups.is_empty() {
        // 如果没有任何至少包含两个文件的哈希分组，就进入这里。
        println!("  没有发现内容相同的重复文件"); // 输出没有重复文件的提示。
        return Ok(()); // 正常结束函数。
    } // 空结果判断结束。

    println!("  重复分组数量：{}", duplicate_groups.len());
    // 输出真正重复的文件分组数量。
    let redundant_file_count: usize = duplicate_groups
        // 借用所有重复文件分组。
        .values()
        // 只遍历每组中的文件清单。
        .map(|duplicate_files| duplicate_files.len().saturating_sub(1))
        // 每组保留一个文件，其余文件计为多余副本。
        .sum();
    // 将每组的多余副本数量相加。

    let reclaimable_bytes: u64 = duplicate_groups
        // 再次借用所有重复文件分组。
        .iter()
        // 同时遍历分组键和文件清单。
        .map(|((file_size, _hash), duplicate_files)| {
            // 取得当前组的文件大小和文件清单；哈希在此计算中不需要使用。
            let redundant_copies = duplicate_files.len().saturating_sub(1) as u64;
            // 每组保留一个文件，并把多余副本数量转换成 u64。
            (*file_size).saturating_mul(redundant_copies)
            // 单个文件大小乘以多余副本数量，得到这一组可释放的字节数。
        })
        // 每个重复分组都转换成可释放字节数。
        .sum();
    // 将所有分组的可释放字节数相加。

    println!("  多余副本数量：{redundant_file_count}");
    // 输出除每组保留文件之外的副本总数。
    println!("  预计可释放字节数：{reclaimable_bytes}");
    // 输出删除多余副本后理论上可以释放的空间。
    for ((file_size, hash), duplicate_files) in duplicate_groups {
        // 逐个取得文件大小、SHA-256 和对应的重复文件清单。
        println!(); // 在每个重复分组前输出空行。
        println!(
            // 输出当前重复分组的大小和文件数量。
            "  {file_size} 字节，共 {} 个相同文件：",
            // 当前分组中所有文件大小都相同。
            duplicate_files.len() // 输出当前重复分组中的文件数量。
        ); // 重复分组标题输出结束。
        println!("  SHA-256：{hash}"); // 输出当前分组共同的 SHA-256。

        for file in duplicate_files {
            // 逐个遍历当前重复分组中的文件。
            println!("    {}", file.path.display());
            // 输出重复文件的路径。
        } // 当前重复分组的路径输出结束。
    } // 所有重复分组输出结束。

    Ok(()) // 表示哈希计算和结果输出成功。
} // print_confirmed_duplicates 函数结束。

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
    println!("  duplicates <目录>    查找内容相同的重复文件"); // duplicates 会计算 SHA-256。
    println!(); // 输出一个空行。
    println!("选项："); // 输出选项列表标题。
    println!("  -h, --help     显示帮助"); // 说明两个帮助选项。
    println!("  compress <输入> <输出> [质量]    压缩单张 JPEG 图片");
    // 说明 compress 命令和可选质量参数。
    println!("  compress-dir <输入目录> <输出目录> [线程数]    并发批量压缩 JPEG");
    // 说明可选的工作线程数量。
    // 说明目录批量压缩命令。
    println!("  watch <目录>    实时监控目录变化");
    // 说明目录监控命令。
} // print_help 函数到这里结束。
