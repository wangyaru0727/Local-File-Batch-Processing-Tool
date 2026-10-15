use notify::{Event, EventKind, RecursiveMode, Watcher};
// 引入文件系统事件、事件类型、递归模式和监控接口。
use std::error::Error;
// 引入通用错误接口。
use std::io;
// 引入 I/O 错误类型。
use std::path::{Path, PathBuf};
// 引入借用路径和拥有所有权的路径类型。
use std::sync::mpsc::{self, RecvTimeoutError};
// 引入事件通道和带超时接收的错误类型。
use std::time::Duration;
// 引入时间长度类型，用于设置去抖窗口。
use std::collections::BTreeMap;
// 引入有序映射，用于按照路径合并重复事件。

const DEBOUNCE_MILLISECONDS: u64 = 300;
// 连续 300 毫秒没有新事件时，输出当前批次。

pub fn watch_directory(path: &Path) -> Result<(), Box<dyn Error>> {
    // 持续监控指定目录，并对短时间内的重复事件进行合并。
    if !path.is_dir() {
        // 如果路径不存在或者不是目录，就进入这里。
        let message = format!("`{}` 不是有效目录", path.display());
        // 创建包含具体路径的错误信息。
        let error = io::Error::new(io::ErrorKind::InvalidInput, message);
        // 创建无效输入类型的错误。
        return Err(error.into());
        // 转换成通用错误并返回。
    } // 目录检查结束。

    let (sender, receiver) = mpsc::channel::<notify::Result<Event>>();
    // 创建监控后端和当前线程之间的事件通道。

    let mut watcher = notify::recommended_watcher(sender)?;
    // 创建当前操作系统推荐的监控器。
    watcher.watch(path, RecursiveMode::Recursive)?;
    // 递归监控指定目录及其子目录。

    println!("正在监控：{}", path.display());
    // 输出监控目录。
    println!("去抖时间：{DEBOUNCE_MILLISECONDS} 毫秒");
    // 输出当前事件合并时间。
    println!("按 Ctrl+C 停止监控");
    // 输出停止方式。

    loop {
        // 持续等待新的事件批次。
        let first_result = match receiver.recv() {
            // 阻塞等待当前批次的第一个事件。
            Ok(result) => result,
            // 成功接收到监控结果。
            Err(_) => return Ok(()),
            // 所有发送端都关闭时正常结束监控。
        }; // 第一个事件接收结束。

        let mut pending_events: BTreeMap<PathBuf, (&'static str, u8)> = BTreeMap::new();
        // 创建“路径 → 事件名称和优先级”的当前批次映射。

        collect_event(first_result, &mut pending_events);
        // 将第一个事件加入当前批次。

        loop {
            // 继续收集紧接着发生的事件。
            match receiver.recv_timeout(Duration::from_millis(DEBOUNCE_MILLISECONDS)) {
                // 最多等待 300 毫秒的新事件。
                Ok(result) => {
                    // 在去抖时间内收到了新事件。
                    collect_event(result, &mut pending_events);
                    // 将新事件合并到当前批次，并重新开始等待 300 毫秒。
                }

                Err(RecvTimeoutError::Timeout) => {
                    // 连续 300 毫秒没有新事件。
                    break;
                    // 结束当前收集阶段，准备输出。
                }

                Err(RecvTimeoutError::Disconnected) => {
                    // 监控后端已经关闭事件通道。
                    return Ok(());
                    // 正常结束目录监控。
                }
            } // 带超时事件接收结束。
        } // 当前事件批次收集结束。

        for (event_path, (event_name, _priority)) in pending_events {
            // 按照路径顺序输出合并后的事件。
            println!("[{event_name}] {}", event_path.display());
            // 每个路径在当前批次中最多输出一次。
        } // 当前事件批次输出结束。
    } // 持续监控循环结束。
} // watch_directory 函数结束。

fn collect_event(
    // 定义单个监控结果收集函数。
    event_result: notify::Result<Event>,
    // 接收正常事件或监控后端错误。
    pending_events: &mut BTreeMap<PathBuf, (&'static str, u8)>,
    // 借用并修改当前事件批次。
) {
    // 收集事件，并按路径去重。
    let event = match event_result {
        // 分别处理正常事件和监控错误。
        Ok(event) => event,
        // 正常事件继续处理。
        Err(error) => {
            // 监控后端报告错误。
            eprintln!("目录监控事件错误：{error}");
            // 输出错误，但不中断整个监控。
            return;
            // 结束当前错误事件处理。
        }
    }; // 监控结果处理结束。

    let Some((event_name, priority)) = event_kind_info(&event.kind) else {
        // 尝试取得事件显示名称和优先级。
        return;
        // Access 事件会返回 None，因此不会输出。
    }; // 事件过滤结束。

    for event_path in event.paths {
        // 逐个处理事件关联的路径。
        pending_events
            // 访问当前事件批次。
            .entry(event_path)
            // 查找当前路径是否已经存在事件。
            .and_modify(|existing| {
                // 如果路径已经存在，就比较事件优先级。
                if priority >= existing.1 {
                    // 新事件优先级更高或相同时进行替换。
                    *existing = (event_name, priority);
                    // 保存更值得报告的事件。
                } // 优先级比较结束。
            })
            // 已存在路径的更新结束。
            .or_insert((event_name, priority));
        // 路径首次出现时直接插入。
    } // 事件路径收集结束。
} // collect_event 函数结束。

fn event_kind_info(kind: &EventKind) -> Option<(&'static str, u8)> {
    // 返回事件显示名称和优先级；None 表示忽略该事件。
    match kind {
        // 根据 notify 事件大类进行匹配。
        EventKind::Remove(_) => Some(("删除", 3)),
        // 删除优先级最高。
        EventKind::Create(_) => Some(("创建", 2)),
        // 创建优先级高于普通修改。
        EventKind::Modify(_) => Some(("修改", 1)),
        // 修改使用普通优先级。
        EventKind::Any => Some(("任意事件", 0)),
        // 无法分类的任意事件使用最低优先级。
        EventKind::Other => Some(("其他事件", 0)),
        // 其他事件使用最低优先级。
        EventKind::Access(_) => None,
        // 读取、打开和关闭事件通常噪声较大，因此忽略。
    } // 事件类型匹配结束。
} // event_kind_info 函数结束。

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
