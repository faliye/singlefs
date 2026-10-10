// 攻方：卡池模块外面、文本上在它之前定义的同名 macro_rules! 顶替钉住那段 Drop 里调的 eprintln!（方法体一个字不动）。
#![allow(dead_code)]
macro_rules! eprintln {
    ($($anything:tt)*) => {{
        println!("ATK 外面定义的 eprintln! 被调到了：钉住那段里的 eprintln! 不再是标准库那一个");
    }};
}

pub mod gpu_test_start {
    pub mod card_pool {
        pub struct Lease;
        impl Drop for Lease {
            fn drop(&mut self) {
                let index = 3;
                eprintln!("GPU_CARD_LEASE_RETURN_WAIT_FAILED index={index}");
            }
        }
    }
}

fn main() {
    drop(gpu_test_start::card_pool::Lease);
}
