//! 样本：2021 版不带 unsafe 写的导出符号属性，unsafe_code 照样报；没登记的 bin 根照 forbid，应当判红。
#[no_mangle]
pub extern "C" fn exported_without_the_unsafe_keyword() {}

fn main() {}
