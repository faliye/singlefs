//! 样本：bin 根旁边同名目录里的子模块归这个 bin 根；放开登记在子模块那份文件里。
#![deny(unsafe_code)]

mod zeroed_byte_maker;

fn main() {
    zeroed_byte_maker::zeroed_byte();
}
