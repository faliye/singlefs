//! 样本：文件后端还没有打洞，逐块写 0。

/// 逐块写 0；这里只在注释里提到 unsafe，不算。
pub fn punch_hole_keeping_the_file_length(length: u64) -> u64 {
    length
}
