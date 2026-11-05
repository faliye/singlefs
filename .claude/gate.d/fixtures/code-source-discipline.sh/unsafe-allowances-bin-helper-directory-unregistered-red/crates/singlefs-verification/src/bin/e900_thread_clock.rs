//! 样本：bin 根旁边同名目录里的子模块归这个 bin 根；bin 根没登记、照 forbid，子模块里的 unsafe 应当判红、点名这个 bin 根。
mod zeroed_byte_maker;

fn main() {
    zeroed_byte_maker::zeroed_byte();
}
