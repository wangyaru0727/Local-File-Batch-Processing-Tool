use crate::scanner::{ScanSummary, scan_directory};
// 从当前库的 scanner 模块引入扫描结果类型和扫描函数。

use std::path::Path;
// 引入 Path，用于把字符串路径转换为文件系统路径。

pub fn scan_for_frontend(path: String) -> Result<ScanSummary, String> {
    // 定义面向前端的扫描函数：成功返回扫描结果，失败返回错误文字。
    let path = Path::new(&path);
    // 把前端传入的 String 临时转换成文件系统能够使用的 Path。

    scan_directory(path).map_err(|error| error.to_string())
    // 调用核心扫描函数，并把 io::Error 转换为前端容易接收的 String。
}

#[cfg(test)]
// 只在执行 cargo test 时编译下面的测试模块。
mod tests {
    // 定义当前文件的单元测试模块。
    use super::scan_for_frontend;
    // 从父模块引入被测试的前端扫描函数。

    #[test]
    // 把下面的函数标记为一个单元测试。
    fn rejects_empty_directory_path() {
        // 测试空路径不会被当成有效扫描目录。
        let result = scan_for_frontend(String::new());
        // 使用空字符串调用前端扫描函数。

        assert!(result.is_err());
        // 确认函数返回错误，而不是错误地返回扫描成功。
    }
}
