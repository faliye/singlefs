// 样本：目录式二进制目标 split_tool，根 main.rs 引子模块 checks；checks 里的内联 tests 模块有一条用例（libtest 全名 checks::tests::tool_case）。
mod checks;

fn main() {
    println!("{}", checks::tool_value());
}
