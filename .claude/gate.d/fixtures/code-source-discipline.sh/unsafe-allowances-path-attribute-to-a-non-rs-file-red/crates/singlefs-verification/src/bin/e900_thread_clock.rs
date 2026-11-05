//! 样本：bin 根用 #[path] 把一份不是 .rs 的文件引成模块，那份文件里的 unsafe 照扫（没登记的 bin 根照 forbid，应当判红）。
#[path = "e900_thread_clock_helper.inc"]
mod zeroed_byte_maker;

fn main() {
    zeroed_byte_maker::zeroed_byte();
}
