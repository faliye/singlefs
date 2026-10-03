//! Y4：别的文件里的宏，展开进被改函数。
macro_rules! step {
    ($x:expr) => {
        $x * 3
    };
}
