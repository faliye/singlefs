//! Y8：函数体里 include_str! 读的数据文件。
pub fn scaled(x: u32) -> u32 {
    x * include_str!("y8_k.txt").trim().parse::<u32>().unwrap()
}
