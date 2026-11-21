//! 每次 API 调用的结局与理想模型的判、停机核对、阳性对照与关掉 oracle 的开关的单测。

use super::*;

#[test]
fn an_error_member_is_the_first_identifier_of_its_debug_text() {
    #[derive(Debug)]
    #[allow(
        dead_code,
        reason = "字段只经 Debug 文本读，测的就是 Debug 文本的第一个标识符"
    )]
    enum Failure {
        Plain,
        WithFields { reason: u8 },
        Tuple(u8),
    }
    assert_eq!(error_member_of(&Failure::Plain), "Plain");
    assert_eq!(
        error_member_of(&Failure::WithFields { reason: 1 }),
        "WithFields"
    );
    assert_eq!(error_member_of(&Failure::Tuple(2)), "Tuple");
}
