//! 判别力样本：不在 e8 的族里的另一份实验源码。它的读数恒为字面量 0 而登记了一行豁免，基准提交里没有这个文件，也不是基准里哪一份入口文件拆出来的。

pub struct Tally {
    pub rows_seen: u64,
}

fn main() {
    let tally = Tally { rows_seen: 0 };
    println!("{}", tally.rows_seen);
}
