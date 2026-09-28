/// 屏障把写流切成段；FUA 写完成才发下一条，装置把它也当段边界（可关，字节表两种口径都报）。
fn split_into_segments(operations: &[RecordedOperation], fua_is_boundary: bool) -> (Vec<WriteRequest>, Vec<Vec<usize>>) {
    let mut writes = Vec::new();
    let mut segments: Vec<Vec<usize>> = Vec::new();
    let mut current: Vec<usize> = Vec::new();
    for operation in operations {
        match operation {
            RecordedOperation::Write(write) => {
                let is_fua = write.is_fua();
                writes.push(write.clone());
                current.push(writes.len() - 1);
                if fua_is_boundary && is_fua {
                    segments.push(std::mem::take(&mut current));
                }
            }
            RecordedOperation::Barrier => {
                if !current.is_empty() {
                    segments.push(std::mem::take(&mut current));
                }
            }
        }
    }
    if !current.is_empty() {
        segments.push(current);
    }
    (writes, segments)
}

/// 每段的步骤种类，按与 `split_into_segments` 相同的切法分组——这是 D17（实现分层与第三方管道） 已定项 2
/// 说的「每段步骤种类集合」的机器可读输入（C316 ②）。关掉一段的那道屏障算进它关掉的那一段；
/// 段里还一个写都没有时（流首那道屏障）算进即将开始的那一段；流尾那一串只有屏障没有写的，并进上一段——
/// 这样录到的每一步都恰好出现在一个段里，段数也与 `split_into_segments` 一样。
/// 末尾的断言把两者钉在一起：逐段数出来的写数必须相等，对不上就是切法分了叉。
fn segment_step_kinds(operations: &[RecordedOperation], fua_is_boundary: bool) -> Vec<Vec<RecordedStepKind>> {
    let mut groups: Vec<Vec<RecordedStepKind>> = Vec::new();
    let mut current: Vec<RecordedStepKind> = Vec::new();
    let mut writes_in_current: usize = 0;
    for operation in operations {
        current.push(RecordedStepKind::of(operation));
        match operation {
            RecordedOperation::Write(write) => {
                writes_in_current += 1;
                if fua_is_boundary && write.is_fua() {
                    groups.push(std::mem::take(&mut current));
                    writes_in_current = 0;
                }
            }
            RecordedOperation::Barrier => {
                if writes_in_current > 0 {
                    groups.push(std::mem::take(&mut current));
                    writes_in_current = 0;
                }
            }
        }
    }
    if !current.is_empty() {
        match (writes_in_current, groups.last_mut()) {
            (0, Some(last_group)) => last_group.append(&mut current),
            (_, _) => groups.push(current),
        }
    }
    let (_, segments) = split_into_segments(operations, fua_is_boundary);
    let writes_per_group: Vec<usize> = groups
        .iter()
        .map(|group| group.iter().filter(|kind| **kind != RecordedStepKind::Barrier).count())
        .collect();
    assert_eq!(writes_per_group, segments.iter().map(Vec::len).collect::<Vec<usize>>(), "步骤种类的切法与 split_into_segments 对不上");
    groups
}

/// 把每段的步骤种类写成一行可解析的文字：段之间用 `|`，段内按枚举声明序排成规范多重集
/// `[种类×次数,…]`，只出现一次的不带次数。规范序是为了让两段只要多重集相同、文字就一模一样。
fn format_segment_kinds(groups: &[Vec<RecordedStepKind>]) -> String {
    let mut rendered: Vec<String> = Vec::new();
    for group in groups {
        let mut counts: BTreeMap<RecordedStepKind, usize> = BTreeMap::new();
        for kind in group {
            *counts.entry(*kind).or_insert(0) += 1;
        }
        let entries: Vec<String> = counts
            .into_iter()
            .map(|(kind, count)| if count == 1 { kind.tag().to_string() } else { format!("{}×{count}", kind.tag()) })
            .collect();
        rendered.push(format!("[{}]", entries.join(",")));
    }
    rendered.join("|")
}

/// E77 的闭式：1 + Σ(2^|段| − 1)。
fn closed_form_state_count(segments: &[Vec<usize>]) -> u64 {
    1 + segments.iter().map(|segment| (1u64 << segment.len()) - 1).sum::<u64>()
}