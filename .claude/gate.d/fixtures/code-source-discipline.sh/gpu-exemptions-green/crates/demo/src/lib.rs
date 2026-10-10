//! 样本：一截不走 GPU 的计算在函数上一行写豁免标记，豁免表里有同名一行。
// gpu-exempt: writable-mount
pub fn judge_the_writable_mount_on_the_cpu() -> bool {
    true
}
