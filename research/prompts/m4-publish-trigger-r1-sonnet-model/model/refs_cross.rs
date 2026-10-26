// 正推腿：用真 RefFS（冻结副本里的 singlefs-reference-filesystem）逐条判 Python 口径生成的用例，每条两个视角。
// 用例格式（trigger_model.py refs-cases 写出）：full_ops|full_crash|explicit_ops|explicit_crash|读出的内容号|真值
// ops：O<k> = OverwriteTheFile(内容 [k])，P = ExplicitPublish；开头隐含 CreateTheFirstFile([0]) + ExplicitPublish。
// 崩溃位置：after = 全部返回之后；inside<j> = 崩在第 j 个（0 起）操作里（必为 P）。
// 输出每条一行 "<full 的判> <explicit 的判>"，1 = 收（落在允许集合里），0 = 判红。
use singlefs_reference_filesystem::{
    FileContent, Operation, OperationNumber, OperationOutcome, ReferenceFileSystem, VisibleState,
};

fn judge(ops_text: &str, crash_text: &str, recovered: u8) -> bool {
    let mut fs = ReferenceFileSystem::on_a_freshly_formatted_pool();
    fs.submit(
        Operation::CreateTheFirstFile(FileContent(vec![0])),
        OperationOutcome::Succeeded,
    )
    .unwrap();
    fs.submit(Operation::ExplicitPublish, OperationOutcome::Succeeded)
        .unwrap();
    let mut length = 0usize;
    for token in ops_text.split_whitespace() {
        let operation = if token == "P" {
            Operation::ExplicitPublish
        } else {
            let k: u8 = token[1..].parse().unwrap();
            Operation::OverwriteTheFile(FileContent(vec![k]))
        };
        fs.submit(operation, OperationOutcome::Succeeded).unwrap();
        length += 1;
    }
    // 序列开头有 2 个隐含的操作；崩溃落在第 hi 个操作里（1 起）
    let hi = if crash_text == "after" {
        length + 2 + 1
    } else {
        crash_text["inside".len()..].parse::<usize>().unwrap() + 2 + 1
    };
    fs.crash_and_recover(
        OperationNumber(hi),
        VisibleState::OneFile(FileContent(vec![recovered])),
    )
    .is_ok()
}

#[test]
fn judge_every_case_with_the_real_reference_filesystem() {
    let path = std::env::var("REFS_CASES").expect("REFS_CASES 指向用例文件");
    let text = std::fs::read_to_string(path).expect("读用例文件");
    let mut out = String::new();
    let mut counted = 0usize;
    for line in text.lines() {
        let fields: Vec<&str> = line.split('|').collect();
        assert_eq!(fields.len(), 6, "用例行字段数：{line}");
        let recovered: u8 = fields[4].parse().unwrap();
        let full = judge(fields[0], fields[1], recovered);
        let explicit = judge(fields[2], fields[3], recovered);
        out.push_str(&format!("{} {}\n", u8::from(full), u8::from(explicit)));
        counted += 1;
    }
    let out_path = std::env::var("REFS_VERDICTS").expect("REFS_VERDICTS 指向输出文件");
    std::fs::write(out_path, out).expect("写判");
    assert!(counted > 0);
}
