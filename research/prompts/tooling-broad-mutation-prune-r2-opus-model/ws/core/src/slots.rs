//! FA：被改函数 slot_of，同文件的兄弟函数 describe 调它。第二版只改 describe。

pub fn slot_of(offset: u64) -> u64 {
    offset / 4096
}

pub fn describe(offset: u64) -> String {
    format!("slot-{}", slot_of(offset))
}
