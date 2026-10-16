import { invoke } from "@tauri-apps/api/core";
// 引入 invoke，用于从 TypeScript 调用 Tauri 的 Rust 命令。

interface FileInfo {
  // 描述 Rust 返回的单个文件信息。
  path: string;
  // 保存文件路径。
  size: number;
  // 保存文件字节数。
}

interface ScanSummary {
  // 描述 Rust 返回的完整扫描结果。
  files: FileInfo[];
  // 保存扫描到的文件列表。
  file_count: number;
  // 保存普通文件数量。
  directory_count: number;
  // 保存子目录数量。
  ignored_directory_count: number;
  // 保存被忽略的目录数量。
  other_count: number;
  // 保存其他类型项目数量。
  total_bytes: number;
  // 保存所有普通文件的总字节数。
}

let pathInputElement: HTMLInputElement | null;
// 保存目录路径输入框。

let scanButtonElement: HTMLButtonElement | null;
// 保存开始扫描按钮。

let scanStatusElement: HTMLElement | null;
// 保存状态提示元素。

let scanResultElement: HTMLElement | null;
// 保存扫描结果元素。

function formatScanSummary(summary: ScanSummary): string {
  // 把结构化的扫描结果整理成适合显示的多行文字。
  const fileLines = summary.files.map(
    // 遍历扫描到的每一个文件。
    (file) => `  ${file.path}（${file.size} 字节）`,
    // 把一个文件转换成一行文字。
  );

  return [
    // 创建最终显示内容的每一行。
    `文件数量：${summary.file_count}`,
    // 显示普通文件数量。
    `子目录数量：${summary.directory_count}`,
    // 显示子目录数量。
    `忽略目录数量：${summary.ignored_directory_count}`,
    // 显示忽略目录数量。
    `其他项目数量：${summary.other_count}`,
    // 显示其他类型项目数量。
    `文件总字节数：${summary.total_bytes}`,
    // 显示所有文件的总大小。
    "",
    // 在统计信息和文件列表之间增加空行。
    "文件列表：",
    // 显示文件列表标题。
    ...fileLines,
    // 把所有文件行展开到最终数组中。
  ].join("\n");
  // 使用换行符把所有内容连接成一个字符串。
}

async function scanDirectory(): Promise<void> {
  // 定义调用 Rust 扫描命令的异步函数。
  if (
    !pathInputElement ||
    // 确认路径输入框已经找到。
    !scanButtonElement ||
    // 确认扫描按钮已经找到。
    !scanStatusElement ||
    // 确认状态元素已经找到。
    !scanResultElement
    // 确认结果元素已经找到。
  ) {
    return;
    // 页面元素不完整时停止执行，避免空值错误。
  }

  const path = pathInputElement.value.trim();
  // 读取目录路径，并删除开头和结尾的空白字符。

  if (path.length === 0) {
    // 检查用户是否输入了目录路径。
    scanStatusElement.textContent = "请输入目录路径";
    // 提示用户补充路径。
    return;
    // 路径为空时停止扫描。
  }

  scanButtonElement.disabled = true;
  // 扫描过程中禁用按钮，防止用户重复提交。

  scanStatusElement.textContent = "正在扫描……";
  // 告诉用户扫描已经开始。

  scanResultElement.textContent = "";
  // 清除上一次扫描的结果。

  try {
    // 尝试调用 Rust 命令。
    const summary = await invoke<ScanSummary>("scan_directory", { path });
    // 把路径交给 Tauri，并等待 Rust 返回 ScanSummary。

    scanStatusElement.textContent = "扫描完成";
    // 告诉用户扫描成功。

    scanResultElement.textContent = formatScanSummary(summary);
    // 安全地把扫描结果显示到页面中。
  } catch (error) {
    // 捕获 Rust 返回的错误或者 Tauri 调用错误。
    scanStatusElement.textContent = `扫描失败：${String(error)}`;
    // 把错误转换成文字并显示给用户。
  } finally {
    // 无论扫描成功还是失败，都会执行这里。
    scanButtonElement.disabled = false;
    // 重新启用扫描按钮。
  }
}

window.addEventListener("DOMContentLoaded", () => {
  // 等待 HTML 页面加载完成。
  pathInputElement = document.querySelector("#path-input");
  // 取得目录路径输入框。

  scanButtonElement = document.querySelector("#scan-button");
  // 取得开始扫描按钮。

  scanStatusElement = document.querySelector("#scan-status");
  // 取得状态提示元素。

  scanResultElement = document.querySelector("#scan-result");
  // 取得扫描结果元素。

  document.querySelector("#scan-form")?.addEventListener("submit", (event) => {
    // 监听扫描表单的提交事件。
    event.preventDefault();
    // 阻止浏览器刷新整个页面。

    void scanDirectory();
    // 启动异步扫描，并明确忽略它返回的 Promise。
  });
});