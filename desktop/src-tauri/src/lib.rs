use local_file_batch_tool::app_api::scan_for_frontend;
// 从主项目的应用接口层引入前端扫描函数。

use local_file_batch_tool::scanner::ScanSummary;
// 从主项目的扫描模块引入可序列化的扫描结果类型。

#[tauri::command]
// 将 greet 注册为前端可以调用的 Tauri 命令。
fn greet(name: &str) -> String {
    // 接收前端传来的姓名，并返回问候文字。
    format!("Hello, {}! You've been greeted from Rust!", name)
    // 使用姓名生成最终的问候字符串。
}

#[tauri::command]
// 将 scan_directory 注册为前端可以调用的 Tauri 命令。
fn scan_directory(path: String) -> Result<ScanSummary, String> {
    // 接收前端传来的目录路径，成功时返回扫描结果，失败时返回错误文字。
    scan_for_frontend(path)
    // 把实际工作交给主项目的应用接口层完成。
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
// 在移动端构建时，把这个函数标记为移动应用入口。
pub fn run() {
    // 创建并启动 Tauri 应用。
    tauri::Builder::default()
        // 创建使用默认设置的 Tauri 应用构建器。
        .plugin(tauri_plugin_opener::init())
        // 注册模板自带的链接和文件打开插件。
        .invoke_handler(tauri::generate_handler![greet, scan_directory])
        // 注册前端可以调用的 greet 和 scan_directory 命令。
        .run(tauri::generate_context!())
        // 读取 Tauri 配置并启动桌面应用。
        .expect("error while running tauri application");
    // 如果 Tauri 无法启动，立即终止程序并显示错误。
}
