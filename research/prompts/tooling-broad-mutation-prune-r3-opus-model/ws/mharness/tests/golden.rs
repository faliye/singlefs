//! X1、X1b：父测试起本测试二进制（current_exe）跑子测试，比子进程打印的行。
use std::io::Write;
use std::process::Command;

const CHILD: &str = "MODEL_GOLDEN_CHILD";
const CHILD_B: &str = "MODEL_GOLDEN_CHILD_B";
const CHILD_C: &str = "MODEL_GOLDEN_CHILD_C";
const CHILD_D: &str = "MODEL_GOLDEN_CHILD_D";

fn printed(f: fn(u64) -> u64) -> Vec<String> {
    (0..5).map(|i| format!("SCORE {}", f(i))).collect()
}

fn child_lines(test: &str, variable: &str) -> (bool, Vec<String>) {
    let out = Command::new(std::env::current_exe().expect("测试二进制"))
        .args(["--exact", test, "--nocapture", "--test-threads=1"])
        .env(variable, "1")
        .output()
        .expect("起子进程");
    let text = String::from_utf8(out.stdout).expect("UTF-8");
    let lines = text
        .lines()
        .filter_map(|line| line.find("SCORE ").map(|at| line[at..].to_string()))
        .collect();
    (out.status.success(), lines)
}

/// 平时自己也跑一遍，只核行数（形同 print_the_unsharded_enumerations_for_the_golden_comparison）。
#[test]
fn child_prints_scores() {
    let lines = printed(mcore::sub::score);
    if std::env::var(CHILD).is_ok() {
        for line in &lines {
            println!("{line}");
        }
    }
    assert_eq!(lines.len(), 5);
}

/// 父进程比子进程打印的每一行（形同 unsharded_enumeration_prints_and_writes_byte_for_byte_what_it_did_before_sharding）。
#[test]
fn golden_parent_compares_child_output() {
    let (ok, lines) = child_lines("child_prints_scores", CHILD);
    assert!(ok);
    assert_eq!(lines, ["SCORE 3", "SCORE 10", "SCORE 17", "SCORE 24", "SCORE 31"]);
}

/// 子进程打印完 abort（模拟崩溃）。平时自己跑只核行数。
#[test]
fn child_prints_scores_then_crashes() {
    let lines = printed(mcore::sub::score_b);
    assert_eq!(lines.len(), 5);
    if std::env::var(CHILD_B).is_ok() {
        for line in &lines {
            println!("{line}");
        }
        std::io::stdout().flush().expect("flush");
        std::process::abort();
    }
}

#[test]
fn golden_parent_reads_the_crashed_child() {
    let (ok, lines) = child_lines("child_prints_scores_then_crashes", CHILD_B);
    assert!(!ok, "子进程按设计 abort");
    assert_eq!(lines, ["SCORE 2", "SCORE 7", "SCORE 12", "SCORE 17", "SCORE 22"]);
}

/// X1c：父测试先后起两个子进程；第一个算分数，第二个只打一行。平时自己跑只核行数。
#[test]
fn child_prints_scores_c() {
    let lines = printed(mcore::sub::score_c);
    if std::env::var(CHILD_C).is_ok() {
        for line in &lines {
            println!("{line}");
        }
    }
    assert_eq!(lines.len(), 5);
}

#[test]
fn child_says_hello() {
    if std::env::var(CHILD_D).is_ok() {
        println!("SCORE hello");
    }
}

#[test]
fn golden_parent_runs_two_children_in_turn() {
    let (ok, lines) = child_lines("child_prints_scores_c", CHILD_C);
    assert!(ok);
    let (ok_hello, hello) = child_lines("child_says_hello", CHILD_D);
    assert!(ok_hello);
    assert_eq!(hello, ["SCORE hello"]);
    assert_eq!(lines, ["SCORE 4", "SCORE 13", "SCORE 22", "SCORE 31", "SCORE 40"]);
}
