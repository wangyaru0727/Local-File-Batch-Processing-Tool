use std::env;
use std::process::ExitCode;

fn main() -> ExitCode {
    let mut args = env::args(); //env::args() 返回的不是普通数组，而是一个迭代器。迭代器可以使用 next() 依次取出内容。

    let program_name = args
        .next() //从参数迭代器中取出第一个参数。通常是程序自身的路径。
        //next() 的返回类型不是直接的 String，而是：Option<String>
        //因为迭代器可能有下一个值，也可能没有，所以 Rust 用 Option 表达这两种情况：
        //Some(字符串) // 成功取得参数
        //None         // 没有取得参数
        .unwrap_or_else(|| String::from("local-file-batch-tool"));
    //如果有值就取出它；如果没有值，就执行括号中的函数产生默认值。
    match args.next().as_deref() {
        //.as_deref()，把：Option<String>临时转换为：Option<&str>
        None | Some("-h") | Some("--help") => {
            print_help(&program_name); //调用后面定义的 print_help 函数。
            ExitCode::SUCCESS //返回成功退出码。
        }
        Some(command) => {
            eprintln!("错误：未知命令 `{command}`");
            print_help(&program_name);
            ExitCode::from(2)
        } //如果参数不是：
          //- 没有参数；
          //- -h；
          //- --help；
          //就会进入这个分支
    }
}

fn print_help(program_name: &str) {
    println!(
        "\
本地文件批处理工具

用法：
  {program_name} <命令>

命令：
  help        显示帮助
  scan        扫描指定目录

选项：
  -h, --help  显示帮助"
    );
}
