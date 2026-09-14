//! C329 / C330 第一轮反推腿（Opus）第二遍的另一个入口：择根倒挂（读不出的是另一个实例的全部根）。

#[allow(dead_code, reason = "第一遍与第二遍的全部项原样搬进来，这个入口只用其中一部分")]
mod model {
    include!(concat!(env!("OUT_DIR"), "/model.rs"));
    include!("../attribution_items.rs");
    include!("../inversion_items.rs");
}

fn main() {
    model::inversion_main();
}
