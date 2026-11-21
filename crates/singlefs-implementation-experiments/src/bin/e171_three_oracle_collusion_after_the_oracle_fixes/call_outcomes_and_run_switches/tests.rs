//! 每次 API 调用的结局、返回值摘要与理想模型的判，停机核对、阳性对照与关掉 oracle 的开关的单测。

use super::*;

/// 阳性对照「panic」只在流二第 3 次调用（从 0 数）的闭包里先 panic：之前的调用照跑，第 3 次记 panic（不调实现）。
#[test]
fn the_panic_control_panics_only_in_the_fourth_call_of_the_second_stream() {
    let switches = RunSwitches {
        positive_control: Some(PositiveControl::PanicInTheCall),
        disabled_oracle: None,
        writable_mount_probe: false,
    };
    let mut driver = StreamDriver::new(
        StreamName::FixedScriptReuseAfterRaisingFloorThenUnmount,
        SharedStream::retaining_contents(),
        0,
        switches,
    );
    let mut implementation_ran = Vec::new();
    for call_index in 0..=PANIC_CONTROL_CALL {
        let _ = driver.call("publish", unit_text, || -> Result<(), RefusedForTheTest> {
            implementation_ran.push(call_index);
            Ok(())
        });
    }
    assert_eq!(implementation_ran, vec![0, 1, 2]);
    assert!(matches!(
        driver.calls[PANIC_CONTROL_CALL].outcome,
        CallOutcome::Panicked(_)
    ));
    let mut first_stream = StreamDriver::new(
        StreamName::NewPoolFileCreation,
        SharedStream::retaining_contents(),
        0,
        switches,
    );
    for _ in 0..=PANIC_CONTROL_CALL {
        assert!(first_stream
            .call("publish", unit_text, || -> Result<(), RefusedForTheTest> {
                Ok(())
            })
            .is_ok());
    }
}

/// 交回值的摘要：整份与顶层字段逐个；字段名照 `{:?}` 文本切（嵌套括号、引号里的逗号不切）；两份只差一个字段，整份摘要不同、只那个字段的摘要不同
/// （V2：摘要取成常量，这里红）。
#[test]
fn return_value_digest_names_every_top_level_field_and_changes_with_one_field() {
    let text = r#"TransactionOutput { root: RootRecord { checkpoint_txg: CheckpointTxg(3) }, record_bytes: [1, 2, 3], name: "a, b" }"#;
    let changed = r#"TransactionOutput { root: RootRecord { checkpoint_txg: CheckpointTxg(4) }, record_bytes: [1, 2, 3], name: "a, b" }"#;
    let digest = return_value_digest_of_debug_text(text);
    let changed_digest = return_value_digest_of_debug_text(changed);
    assert_eq!(
        digest
            .fields
            .iter()
            .map(|(name, _)| name.as_str())
            .collect::<Vec<_>>(),
        vec!["root", "record_bytes", "name"]
    );
    assert_eq!(digest.whole, sha256_hexadecimal(text.as_bytes()));
    assert_ne!(digest.whole, changed_digest.whole);
    assert_ne!(digest.fields[0].1, changed_digest.fields[0].1);
    assert_eq!(digest.fields[1..], changed_digest.fields[1..]);
    assert_eq!(
        top_level_fields_of_debug_text("InstanceGeneration(1)"),
        vec![("#0".to_string(), "1".to_string())]
    );
    assert_eq!(
        top_level_fields_of_debug_text("None"),
        vec![("#0".to_string(), "None".to_string())]
    );
}

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
