//! C329 / C330 第一轮反推腿（Opus）的第二遍：同一个模型（build.rs 把 ../src/main.rs 搬进来，只去掉 `//!` 行、把入口改名），
//! 加三件事：回退那次发布的实例表以哪一版为底（归因第一遍枚举里甲的 U1）、M1 不带一直被引用的节点（把重放施加进来的节点那一类 U2 摘出去）、
//! 择根倒挂（读不出的根的 txg 高过新实例已确认的根）。

#[allow(dead_code, reason = "第一遍的全部项原样搬进来，第二遍只用其中一部分")]
mod model {
    include!(concat!(env!("OUT_DIR"), "/model.rs"));
    include!("attribution_items.rs");
}

fn main() {
    model::attribution_main();
}
