//! 样本：原始 C 字符串里的反斜杠不是转义；认得 cr 前缀才看得见后面那一处 unsafe（没登记的 bin 根照 forbid，应当判红）。
fn main() {
    let _opening = cr"\";
    let _status = unsafe { std::mem::zeroed::<u8>() };
    let _closing = cr"\";
}
