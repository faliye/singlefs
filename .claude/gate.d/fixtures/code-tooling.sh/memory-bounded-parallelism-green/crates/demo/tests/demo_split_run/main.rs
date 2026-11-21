// 样本：目录式测试目标，main.rs 只声明子模块
mod part;

fn main() {
    println!("{}", part::start_group(8));
}
