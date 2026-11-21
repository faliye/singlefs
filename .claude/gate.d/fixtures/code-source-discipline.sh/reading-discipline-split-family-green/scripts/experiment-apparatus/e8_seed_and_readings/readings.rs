//! 判别力样本：e8 拆出来的读数结构体。`visited_nodes` 是对照，在 main.rs 里被算出来；`units_read_on_commit` 恒为字面量 0。

pub struct Reading {
    pub visited_nodes: u64,
    pub units_read_on_commit: u64,
}
