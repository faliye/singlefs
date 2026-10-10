//! 样本：一处标记有表行，一处标记（continuation）没有表行；表里 second-crash-during-mount 那一行在代码里没有标记。
// gpu-exempt: writable-mount
pub fn judge_the_writable_mount_on_the_cpu() -> bool {
    true
}

// gpu-exempt: continuation
pub fn continue_after_the_mount_on_the_cpu() -> bool {
    true
}
