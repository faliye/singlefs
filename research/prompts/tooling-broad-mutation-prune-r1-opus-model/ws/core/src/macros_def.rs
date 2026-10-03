/// S5 宏定义在这个文件，展开在 macro_user.rs：量覆盖把展开出来的代码记在哪个文件。
#[macro_export]
macro_rules! wrap_once {
    ($publish:expr, $slots:expr) => {
        if $publish >= $slots {
            $publish - $slots
        } else {
            $publish
        }
    };
}
