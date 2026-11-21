//! O1-RefFS 子列：参照文件系统的事件流与它对一个崩溃状态的投票的单测。

use super::*;

fn content_of(byte: u8) -> Vec<u8> {
    vec![byte; 10]
}

fn calls_for_the_reference_filesystem(third_call_outcome: CallOutcome) -> Vec<RecordedCall> {
    let call = |label: &'static str,
                first_operation: usize,
                return_point: usize,
                outcome: CallOutcome,
                content: Option<PublishedContent>| RecordedCall {
        label,
        first_operation,
        return_point,
        outcome,
        model: ModelVerdictOfCall::NotModelled,
        return_value: None,
        memory_objects: Vec::new(),
        published_content: content,
        observation_sha256: None,
    };
    vec![
        call("acquire", 0, 2, CallOutcome::Succeeded, None),
        call("warm_up", 2, 12, CallOutcome::Succeeded, None),
        call(
            "publish",
            12,
            40,
            CallOutcome::Succeeded,
            Some(PublishedContent::FirstFile(content_of(1))),
        ),
        call(
            "publish",
            40,
            70,
            third_call_outcome,
            Some(PublishedContent::Overwrite(content_of(2))),
        ),
    ]
}

/// 70 次写都在盘 0 上，下标 39 是一道屏障（关上第 0 段）：段由第 2 次调用关上。
fn operations_closing_one_segment_at_39() -> Vec<RetainedOperation> {
    (0..70usize)
        .map(|index| RetainedOperation {
            operation: singlefs_verification::RecordedOperation {
                device: DeviceIdentity(0),
                kind: if index == 39 {
                    RecordedOperationKind::Barrier
                } else {
                    RecordedOperationKind::Write
                },
                offset: DeviceOffsetInBytes(0),
                length: 0,
                content_hash: 0,
            },
            contents: None,
        })
        .collect()
}

/// RefFS 的崩溃时刻按「关上这一段的那道屏障由哪一步发出」归属（V7：归到下一次调用，这里红）：第 0 段由第 2 次调用关上，
/// hi 是第 2 次调用里最后进序列的操作（发布调用，第 2 个），下界 lo 还是 0；读回「没有文件」（第 2 次调用之前那一版）判绿。
/// RefFS 判出不在允许集合里的记红（V6：票恒为绿，这里红）。
#[test]
fn crash_in_the_segment_the_publish_closes_falls_inside_that_publish() {
    let stream = reference_filesystem_stream_of(
        &calls_for_the_reference_filesystem(CallOutcome::Succeeded),
        &operations_closing_one_segment_at_39(),
        &[],
        &[],
    );
    assert_eq!(stream.segment_closing_stream_indexes.len(), 2);
    let moment = reference_filesystem_crash_moment(&stream, Some(0));
    assert_eq!(
        moment,
        ReferenceFileSystemCrashMoment::InsideStep(StepPosition::Operation(2))
    );
    let no_file = ObservedReadBack::NoFile {
        root: model_root_key(InstanceGeneration(1), CheckpointTxg(2)),
    };
    assert_eq!(
        reference_filesystem_vote_of_a_state(&stream, moment, &no_file),
        ReferenceFileSystemVoteOfAState::Green {
            allowed: "S_0,S_2".to_string()
        }
    );
    let after_everything = reference_filesystem_crash_moment(&stream, None);
    assert_eq!(
        after_everything,
        ReferenceFileSystemCrashMoment::AfterEveryTranslatedStep
    );
    assert!(matches!(
        reference_filesystem_vote_of_a_state(&stream, after_everything, &no_file),
        ReferenceFileSystemVoteOfAState::Red { .. }
    ));
}

/// 截断处在途的那次覆盖写按「写了之后报错」交（V8：记成「成」，下界抬到它之后，这里红）：崩溃在全部步之后，读回第 2 次调用那一版
/// （第一个文件的内容）在允许集合里。
#[test]
fn an_overwrite_in_flight_at_the_truncation_leaves_the_confirmed_version_allowed() {
    let stream = reference_filesystem_stream_of(
        &calls_for_the_reference_filesystem(CallOutcome::InFlightAtTheTruncation),
        &operations_closing_one_segment_at_39(),
        &[],
        &[],
    );
    let read_back_of_the_confirmed_version = ObservedReadBack::FileRead {
        root: model_root_key(InstanceGeneration(1), CheckpointTxg(3)),
        content: content_of(1),
    };
    assert_eq!(
        reference_filesystem_vote_of_a_state(
            &stream,
            ReferenceFileSystemCrashMoment::AfterEveryTranslatedStep,
            &read_back_of_the_confirmed_version
        ),
        ReferenceFileSystemVoteOfAState::Green {
            allowed: "S_2,S_4".to_string()
        }
    );
}

/// unlink 写了盘：翻译停在那一步，崩在它里面或之后的弃权，种类是「RefFS 第一版没有删除」（第七节 A5）。
#[test]
fn the_reference_filesystem_abstains_from_the_unlink_on() {
    let mut calls = calls_for_the_reference_filesystem(CallOutcome::Succeeded);
    calls[3].label = "unlink_intent";
    calls[3].published_content = None;
    let stream =
        reference_filesystem_stream_of(&calls, &operations_closing_one_segment_at_39(), &[], &[]);
    assert_eq!(
        reference_filesystem_vote_of_a_state(
            &stream,
            ReferenceFileSystemCrashMoment::AfterEveryTranslatedStep,
            &ObservedReadBack::NoFile {
                root: model_root_key(InstanceGeneration(1), CheckpointTxg(4))
            }
        ),
        ReferenceFileSystemVoteOfAState::Abstained {
            kind: "RefFS 第一版没有删除"
        }
    );
}
