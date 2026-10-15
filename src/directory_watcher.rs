use notify::{Event, EventKind, RecursiveMode, Watcher};
// 引入文件系统事件、事件类型、递归模式和监控接口。
use std::error::Error;
// 引入通用错误接口。
use std::io;
// 引入 I/O 错误类型。
use std::path::Path;
// 引入借用路径类型。
use std::sync::mpsc;
// 引入多生产者单消费者通道，用于接收监控事件。

pub fn watch_directory(path: &Path) -> Result<(), Box<dyn Error>> {
    // 持续监控指定目录；发生启动错误时返回。
    if !path.is_dir() {
        // 如果路径不存在或者不是目录，就进入这里。
        let message = format!("`{}` 不是有效目录", path.display());
        // 创建包含具体路径的错误信息。
        let error = io::Error::new(io::ErrorKind::InvalidInput, message);
        // 创建无效输入类型的错误。
        return Err(error.into());
        // 转换为通用错误并立即返回。
    } // 目录有效性检查结束。

    let (sender, receiver) = mpsc::channel::<notify::Result<Event>>();
    // 创建事件通道；监控线程发送事件，当前线程接收事件。

    let mut watcher = notify::recommended_watcher(sender)?;
    // 创建适合当前操作系统的文件监控器。

    watcher.watch(path, RecursiveMode::Recursive)?;
    // 开始递归监控目录及其所有子目录。

    println!("正在监控：{}", path.display());
    // 输出当前监控路径。
    println!("按 Ctrl+C 停止监控");
    // 告诉用户如何终止持续运行的命令。

    for event_result in receiver {
        // 持续阻塞并等待文件系统事件。
        match event_result {
            // 分别处理正常事件和监控错误。
            Ok(event) => {
                // 成功收到文件系统事件。
                print_event(&event);
                // 将事件转换成容易阅读的输出。
            }

            Err(error) => {
                // 文件监控后端报告单次错误。
                eprintln!("目录监控事件错误：{error}");
                // 输出错误，但继续等待后续事件。
            }
        } // 当前事件处理结束。
    } // 通道关闭后退出监控循环。

    Ok(())
    // 监控通道正常关闭时返回成功。
} // watch_directory 函数结束。

fn print_event(event: &Event) {
    // 输出一个文件系统事件的类型和关联路径。
    let event_name = event_kind_name(&event.kind);
    // 将 notify 的事件类型转换成中文名称。

    if event.paths.is_empty() {
        // 某些特殊事件可能没有关联路径。
        println!("[{event_name}] 没有路径信息");
        // 输出事件名称和说明。
        return;
        // 提前结束当前事件输出。
    } // 空路径事件判断结束。

    for path in &event.paths {
        // 一个事件可能同时包含源路径和目标路径。
        println!("[{event_name}] {}", path.display());
        // 输出事件类型和当前路径。
    } // 事件路径输出循环结束。
} // print_event 函数结束。

fn event_kind_name(kind: &EventKind) -> &'static str {
    // 将 EventKind 转换成静态中文说明。
    match kind {
        // 根据事件的大类进行匹配。
        EventKind::Create(_) => "创建",
        // 文件或目录被创建。
        EventKind::Modify(_) => "修改",
        // 文件内容、元数据或名称发生变化。
        EventKind::Remove(_) => "删除",
        // 文件或目录被删除。
        EventKind::Access(_) => "访问",
        // 文件被打开、读取或关闭。
        EventKind::Any => "任意事件",
        // 后端无法提供更具体的事件类型。
        EventKind::Other => "其他事件",
        // 不属于常见分类的事件。
    } // 事件类型匹配结束。
} // event_kind_name 函数结束。
