use image::{DynamicImage, ImageReader};
// 引入动态图片类型和图片读取器。
use image::codecs::jpeg::JpegEncoder; // 引入支持质量参数的 JPEG 编码器。
use rayon::ThreadPoolBuilder; // 引入线程池构建器，用于限制并发线程数量。
use rayon::prelude::*;
use std::error::Error; // 引入通用错误接口。
use std::fs::{self, OpenOptions}; // 引入文件元数据和安全的新建文件选项。
use std::io::{self, Write}; // 引入 I/O 错误和文件写入接口。
use std::path::{Path, PathBuf}; // 引入借用路径和拥有所有权的路径类型。 // 引入 par_iter 等并行迭代器接口。

#[derive(Debug)] // 自动生成调试输出能力。
pub struct CompressionResult {
    // 保存一次图片压缩产生的统计结果。
    pub original_bytes: u64,   // 保存原始文件大小。
    pub compressed_bytes: u64, // 保存输出文件大小。
} // CompressionResult 定义结束。

#[derive(Debug)] // 自动生成调试输出能力。
pub struct CompressionFailure {
    // 保存一个压缩失败文件的信息。
    pub input_path: PathBuf, // 保存失败文件的路径。
    pub message: String,     // 保存转换后的错误信息。
} // CompressionFailure 定义结束。

#[derive(Debug, Default)] // 自动生成调试输出和默认值。
pub struct BatchCompressionResult {
    // 保存一次目录批量压缩的汇总结果。
    pub discovered_count: usize,           // 保存发现的 JPEG 文件数量。
    pub successful_count: usize,           // 保存压缩成功的文件数量。
    pub original_bytes: u64,               // 保存成功文件的原始总字节数。
    pub compressed_bytes: u64,             // 保存成功文件的输出总字节数。
    pub failures: Vec<CompressionFailure>, // 保存所有失败文件及其错误。
} // BatchCompressionResult 定义结束。

fn encode_jpeg(image: &DynamicImage, quality: u8) -> image::ImageResult<Vec<u8>> {
    // 把内存中的图片编码为 JPEG 字节，不直接操作磁盘。
    let mut encoded_bytes = Vec::new();
    // 创建用于接收 JPEG 数据的可增长字节数组。
    let encoder = JpegEncoder::new_with_quality(&mut encoded_bytes, quality);
    // 创建指定质量的 JPEG 编码器，并让它写入字节数组。
    image.write_with_encoder(encoder)?;
    // 执行 JPEG 编码；失败时立即返回图片错误。
    Ok(encoded_bytes)
    // 返回完整的 JPEG 字节数据。
} // encode_jpeg 函数结束。

pub fn compress_jpeg(
    // 定义允许其他模块调用的 JPEG 压缩函数。
    input: &Path,
    // 借用输入文件路径。
    output: &Path,
    // 借用输出文件路径。
    quality: u8,
    // 接收 1 到 100 的 JPEG 质量。
) -> Result<CompressionResult, Box<dyn Error>> {
    // 成功时返回压缩统计，失败时返回图片或文件系统错误。
    if !(1..=100).contains(&quality) {
        // 再次验证质量参数，防止其他调用者绕过命令行检查。
        let error = io::Error::new(
            // 创建标准 I/O 参数错误。
            io::ErrorKind::InvalidInput,
            // 标记错误类型为无效输入。
            "JPEG 质量必须在 1 到 100 之间",
            // 提供人类可读的错误信息。
        ); // 参数错误创建结束。

        return Err(error.into());
        // 将 I/O 错误转换成通用错误并返回。
    } // 质量参数验证结束。

    let original_bytes = fs::metadata(input)?.len();
    // 读取输入文件元数据，并取得原始字节数。
    let image = ImageReader::open(input)?.decode()?;
    // 打开并解码输入 JPEG；失败时立即返回错误。

    let encoded_bytes = encode_jpeg(&image, quality)?;
    // 调用可独立测试的内存编码函数，取得完整 JPEG 数据。

    let mut output_file = OpenOptions::new()
        // 创建输出文件打开选项。
        .write(true)
        // 允许写入输出文件。
        .create_new(true)
        // 只允许创建新文件，绝不覆盖已经存在的文件。
        .open(output)?;
    // 按照上述选项创建输出文件；文件存在时返回错误。

    output_file.write_all(&encoded_bytes)?;
    // 将内存中的完整 JPEG 数据写入输出文件。
    output_file.flush()?;
    // 要求缓冲数据立即交给操作系统。

    let compressed_bytes = encoded_bytes.len() as u64;
    // 取得编码后 JPEG 的字节数。

    Ok(CompressionResult {
        // 创建并返回压缩统计结果。
        original_bytes,   // 保存原始文件大小。
        compressed_bytes, // 保存压缩后文件大小。
    }) // CompressionResult 创建结束。
} // compress_jpeg 函数结束。

fn is_jpeg_file(path: &Path) -> bool {
    // 根据文件扩展名判断是否是 JPEG 候选文件。
    path.extension()
        // 取得路径的扩展名，例如 jpg。
        .and_then(|extension| extension.to_str())
        // 尝试把操作系统字符串转换为 UTF-8 字符串。
        .is_some_and(|extension| {
            // 只有扩展名存在并能转换时才执行判断。
            extension.eq_ignore_ascii_case("jpg")
                // 不区分大小写地匹配 jpg。
                || extension.eq_ignore_ascii_case("jpeg")
            // 或者不区分大小写地匹配 jpeg。
        })
    // 返回最终判断结果。
} // is_jpeg_file 函数结束。

pub fn compress_jpeg_directory(
    // 定义并发目录批量压缩函数。
    input_directory: &Path,
    // 借用输入目录路径。
    output_directory: &Path,
    // 借用输出目录路径。
    quality: u8,
    // 接收 JPEG 压缩质量。
    worker_count: usize,
    // 接收线程池中的工作线程数量。
) -> Result<BatchCompressionResult, Box<dyn Error>> {
    // 成功时返回汇总结果，严重错误时返回通用错误。
    if !input_directory.is_dir() {
        // 验证输入路径确实是目录。
        let message = format!("`{}` 不是有效输入目录", input_directory.display());
        // 创建包含路径的错误信息。
        let error = io::Error::new(io::ErrorKind::InvalidInput, message);
        // 创建无效输入错误。
        return Err(error.into());
        // 返回错误，不创建输出目录。
    } // 输入目录验证结束。

    if output_directory.exists() {
        // 输出目录必须不存在，防止覆盖已有文件。
        let message = format!("输出目录 `{}` 已存在", output_directory.display());
        // 创建包含输出路径的错误信息。
        let error = io::Error::new(io::ErrorKind::AlreadyExists, message);
        // 创建目标已存在错误。
        return Err(error.into());
        // 返回错误，不写入任何文件。
    } // 输出目录验证结束。

    if !(1..=100).contains(&quality) {
        // 验证 JPEG 质量范围。
        let error = io::Error::new(
            // 创建参数错误。
            io::ErrorKind::InvalidInput,
            // 标记为无效输入。
            "JPEG 质量必须在 1 到 100 之间",
            // 设置错误说明。
        ); // 参数错误创建结束。
        return Err(error.into());
        // 返回质量参数错误。
    } // 质量验证结束。

    if !(1..=64).contains(&worker_count) {
        // 限制线程数，避免意外创建过多线程。
        let error = io::Error::new(
            // 创建线程数参数错误。
            io::ErrorKind::InvalidInput,
            // 标记为无效输入。
            "工作线程数必须在 1 到 64 之间",
            // 设置错误说明。
        ); // 线程数错误创建结束。
        return Err(error.into());
        // 返回线程数错误。
    } // 工作线程数验证结束。

    let mut input_files = Vec::new();
    // 创建 JPEG 输入任务清单。

    for entry_result in fs::read_dir(input_directory)? {
        // 读取输入目录第一层。
        let entry = entry_result?;
        // 取得一个目录项目。
        let file_type = entry.file_type()?;
        // 读取项目类型。
        let path = entry.path();
        // 取得项目完整路径。

        if file_type.is_file() && is_jpeg_file(&path) {
            // 只收集普通 JPEG 文件。
            input_files.push(path);
            // 将 JPEG 路径加入任务清单。
        } // JPEG 文件筛选结束。
    } // 输入目录读取结束。

    input_files.sort();
    // 对任务路径排序，使结果顺序稳定。

    if input_files.is_empty() {
        // 没有找到 JPEG 时不创建输出目录。
        let error = io::Error::new(
            // 创建未找到文件错误。
            io::ErrorKind::NotFound,
            // 标记为未找到。
            "输入目录第一层没有找到 JPEG 文件",
            // 设置错误说明。
        ); // 未找到文件错误创建结束。
        return Err(error.into());
        // 返回错误。
    } // 空任务判断结束。

    fs::create_dir(output_directory)?;
    // 原子创建新的输出目录。

    let thread_pool = ThreadPoolBuilder::new()
        // 创建自定义 Rayon 线程池构建器。
        .num_threads(worker_count)
        // 明确设置线程池大小。
        .thread_name(|index| format!("image-worker-{index}"))
        // 为工作线程命名，方便以后诊断。
        .build()?;
    // 创建线程池；失败时返回错误。

    let outcomes: Vec<(PathBuf, Result<CompressionResult, String>)> = thread_pool.install(|| {
        // 在线程池中执行下面的并行任务。
        input_files
            // 访问已经排序的任务清单。
            .par_iter()
            // 创建并行迭代器，让不同文件在不同线程中处理。
            .map(|input_path| {
                // 为每个输入路径创建一个独立任务。
                let outcome = match input_path.file_name() {
                    // 尝试取得输入文件名称。
                    Some(file_name) => {
                        // 成功取得文件名称。
                        let output_path = output_directory.join(file_name);
                        // 使用原文件名构造唯一输出路径。

                        compress_jpeg(input_path, &output_path, quality)
                            // 调用已经验证过的单张压缩函数。
                            .map_err(|error| error.to_string())
                        // 将通用错误立即转换成可以在线程间传递的 String。
                    }

                    None => Err(String::from("无法取得文件名")),
                    // 没有文件名时创建失败结果。
                }; // 当前文件压缩结果创建结束。

                (input_path.clone(), outcome)
                // 返回输入路径及其成功或失败结果。
            })
            // 所有输入文件都会执行上述并行映射。
            .collect()
        // 收集全部任务结果；输入 Vec 的并行迭代会保持顺序。
    }); // 线程池任务执行结束。

    let mut result = BatchCompressionResult {
        // 创建批量压缩汇总。
        discovered_count: input_files.len(),
        // 记录发现的 JPEG 文件数量。
        ..BatchCompressionResult::default() // 其他字段使用默认值。
    }; // 汇总结果创建结束。

    for (input_path, outcome) in outcomes {
        // 在主线程中按顺序汇总并行任务结果。
        match outcome {
            // 分别处理成功和失败。
            Ok(compression) => {
                // 当前文件压缩成功。
                result.successful_count += 1;
                // 成功数量增加 1。
                result.original_bytes = result
                    // 访问原始总字节数。
                    .original_bytes
                    // 取得当前累计值。
                    .saturating_add(compression.original_bytes);
                // 安全累加当前文件原始大小。
                result.compressed_bytes = result
                    // 访问压缩后总字节数。
                    .compressed_bytes
                    // 取得当前累计值。
                    .saturating_add(compression.compressed_bytes);
                // 安全累加当前输出大小。
            }

            Err(message) => {
                // 当前文件压缩失败。
                result.failures.push(CompressionFailure {
                    // 创建失败记录。
                    input_path, // 保存失败文件路径。
                    message,    // 保存工作线程返回的错误文本。
                }); // 失败记录创建结束。
            }
        } // 当前任务结果处理结束。
    } // 全部并行结果汇总结束。

    Ok(result)
    // 返回最终汇总。
} // compress_jpeg_directory 函数结束。

#[cfg(test)] // 只有执行 cargo test 时才编译测试模块。
mod tests {
    // 定义图片压缩模块的自动化测试。
    use super::*; // 引入父模块中的 encode_jpeg 和 DynamicImage。
    use image::ImageFormat; // 引入图片格式枚举，用于按 JPEG 格式解码。

    #[test] // 标记下面的函数是一项自动化测试。
    fn encodes_valid_jpeg_data() {
        // 验证内存图片能被编码成有效的 JPEG 数据。
        let source_image = DynamicImage::new_rgb8(2, 2);
        // 创建一张宽 2 像素、高 2 像素的内存 RGB 图片。

        let encoded_bytes = encode_jpeg(&source_image, 80)
            // 使用质量 80 对内存图片进行 JPEG 编码。
            .expect("内存图片应该能够成功编码为 JPEG");
        // 如果编码失败，就让测试失败并显示说明。

        assert!(encoded_bytes.starts_with(&[0xff, 0xd8, 0xff]));
        // 有效 JPEG 通常以 FF D8 FF 三个字节开头。

        let decoded_image =
            // 准备重新解码刚刚生成的 JPEG。
            image::load_from_memory_with_format(&encoded_bytes, ImageFormat::Jpeg)
                // 明确告诉图片库按 JPEG 格式解码内存数据。
                .expect("生成的数据应该能够被重新解码为 JPEG");
        // 如果无法解码，说明编码结果不是有效 JPEG。

        assert_eq!(decoded_image.width(), 2);
        // 重新解码后的图片宽度应该仍然是 2。
        assert_eq!(decoded_image.height(), 2);
        // 重新解码后的图片高度应该仍然是 2。
    } // encodes_valid_jpeg_data 测试结束。
} // tests 模块结束。
