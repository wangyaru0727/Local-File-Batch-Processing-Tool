use image::{DynamicImage, ImageReader};
// 引入动态图片类型和图片读取器。
use image::codecs::jpeg::JpegEncoder; // 引入支持质量参数的 JPEG 编码器。
use std::error::Error; // 引入通用错误接口。
use std::fs::{self, OpenOptions}; // 引入文件元数据和安全的新建文件选项。
use std::io::{self, Write}; // 引入 I/O 错误和文件写入接口。
use std::path::Path; // 引入借用路径类型。

#[derive(Debug)] // 自动生成调试输出能力。
pub struct CompressionResult {
    // 保存一次图片压缩产生的统计结果。
    pub original_bytes: u64,   // 保存原始文件大小。
    pub compressed_bytes: u64, // 保存输出文件大小。
} // CompressionResult 定义结束。

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
