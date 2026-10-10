//! E176（inode 树叶合并三候选的容量与写代价）的对拍 bin：跑前登记 `research/prompts/e176-preregistration.md` 第三、五、十一节与第十二节修订 2。
//!
//! 把每一条登记序列在 M1 被拒之前的每一步，同时喂给模型的 M1（`../e176_inode_leaf_merge_capacity_and_rewrites.rs`，以 `#[path]` 引进来）
//! 与 `crates/singlefs-core/src/inode_tree.rs` 的两个纯函数（`write_records_into_leaf_containers` / `remove_record_from_leaf_containers`），逐步比：
//! 片数、每片（容器号、记录数、最小号、最大号）、重写清单（位置记法逐项、内容记法 = 清单里内容真变了的那几片）、摘掉的片数、被拒的那一步与拒的格。
//! 另回比模型的本地常量（233、135）与 `crates/` 单测钉住的三种形态。只读 `crates/`，不改它。
//! 用同一目录下自己的 `Cargo.toml` 编（编法与为什么不挂在 e7-index-bench 的可选特性上，写在那份 manifest 的文件头）。
//! 第一处不同就停这条序列、整段退 1（第十一节停机：两边都查，不改模型迁就 `crates/`）。

#[path = "../e176_inode_leaf_merge_capacity_and_rewrites.rs"]
#[allow(dead_code, reason = "模型那份文件自带 main、臂的 M2 / M3 与判定，对拍只用它的 M1、生成器与登记")]
mod model;

use model::{
    InodeLeafTree, InsertOutcome, LeafContainerLimit, MergePolicy, Operation, RegisteredSequence, WorkloadGenerator, LEAF_RECORD_CAPACITY,
    OPERATIONS_PER_ARM_AT_MOST, ROOT_NODE_LEAF_CONTAINER_CAPACITY,
};
use singlefs_core::address::{CheckpointTxg, InodeNumber, TreeIdentifier};
use singlefs_core::inode_tree::{
    leaf_containers_one_root_node_holds, remove_record_from_leaf_containers, write_records_into_leaf_containers, InodeLeafContainer,
    InodeLeafContainersAfterThisPublish, InodeTreeWriteRefusal,
};
use singlefs_core::records::InodeRecord;
use singlefs_format::INODE_LEAF_RECORDS;

/// 对拍用的 inode 树 ID 与第一次发布的 checkpoint_txg（取值不影响分组与重写清单，只要每步单调）。
const ORACLE_INODE_TREE: TreeIdentifier = TreeIdentifier(1);
const FIRST_CHECKPOINT_TXG: u64 = 3;

fn record(inode: u64, checkpoint_txg: u64) -> InodeRecord {
    InodeRecord { inode, object_birth: CheckpointTxg(checkpoint_txg), nlink: 1, size: 0, change_count: checkpoint_txg, write_time_seconds: 0 }
}

/// 两边每片的（容器号、记录数、最小号、最大号）。
fn model_shape(tree: &InodeLeafTree) -> Vec<(u64, usize, u64, u64)> {
    tree.containers
        .iter()
        .map(|container| (container.container_number, container.inode_numbers.len(), container.smallest_inode_number(), container.largest_inode_number()))
        .collect()
}

fn crates_shape(containers: &[InodeLeafContainer]) -> Vec<(u64, usize, u64, u64)> {
    containers
        .iter()
        .map(|container| {
            (
                container.identity.container,
                container.records.len(),
                container.smallest_inode_number().expect("crates 不留空容器").0,
                container.largest_inode_number().expect("crates 不留空容器").0,
            )
        })
        .collect()
}

/// `crates/` 重写清单里内容真变了的那几片：按容器号找上一版，记录不同（或上一版没有它）才算。
fn crates_content_positions(before: &[InodeLeafContainer], after: &InodeLeafContainersAfterThisPublish) -> Vec<usize> {
    after
        .rewritten
        .iter()
        .map(|index| index.position())
        .filter(|position| {
            let rewritten = &after.containers[*position];
            before
                .iter()
                .find(|previous| previous.identity.container == rewritten.identity.container)
                .is_none_or(|previous| previous.records != rewritten.records)
        })
        .collect()
}

/// 一条序列的对拍结局：比了几步、几次删除、M1 被拒在第几个操作、第一处不同（没有就是 None）。
struct OracleOutcome {
    operations_compared: u64,
    unlinks_compared: u64,
    refused_operation: Option<u64>,
    first_difference: Option<String>,
}

fn compare_one_sequence(sequence: RegisteredSequence) -> OracleOutcome {
    let mut generator = WorkloadGenerator::new(sequence.workload, sequence.seed);
    let mut model_tree = InodeLeafTree::empty(MergePolicy::DropEmptyOnly, LeafContainerLimit::OneRootNode);
    let mut crates_containers: Vec<InodeLeafContainer> = Vec::new();
    let mut outcome = OracleOutcome { operations_compared: 0, unlinks_compared: 0, refused_operation: None, first_difference: None };
    for step in 0..OPERATIONS_PER_ARM_AT_MOST {
        let operation_number = step + 1;
        let checkpoint_txg = FIRST_CHECKPOINT_TXG + step;
        let operation = generator.next_operation();
        let difference = match operation {
            Operation::Create(inode) => {
                let model_insert = model_tree.insert(inode);
                let crates_insert = write_records_into_leaf_containers(&crates_containers, &[record(inode, checkpoint_txg)], CheckpointTxg(checkpoint_txg), ORACLE_INODE_TREE);
                match (model_insert, crates_insert) {
                    (InsertOutcome::Accepted, Ok(after)) => {
                        crates_containers = after.containers;
                        None
                    }
                    (
                        InsertOutcome::RefusedMoreLeafContainersThanOneRootNodeHolds { containers_it_would_take },
                        Err(InodeTreeWriteRefusal::MoreLeafContainersThanOneRootNodeHolds { containers, capacity }),
                    ) => {
                        outcome.refused_operation = Some(operation_number);
                        outcome.operations_compared = operation_number;
                        if containers == containers_it_would_take && capacity == ROOT_NODE_LEAF_CONTAINER_CAPACITY {
                            return outcome;
                        }
                        Some(format!("拒的格不同：模型 {containers_it_would_take} 片，crates {containers} 片、上限 {capacity}"))
                    }
                    (model_side, crates_side) => Some(format!("插入结局不同：模型 {model_side:?}，crates {crates_side:?}")),
                }
            }
            Operation::Unlink(inode) => {
                outcome.unlinks_compared += 1;
                let model_rewrites = model_tree.remove(inode);
                match remove_record_from_leaf_containers(&crates_containers, InodeNumber(inode)) {
                    Err(refusal) => Some(format!("crates 拒了这次删除：{refusal:?}")),
                    Ok(after) => {
                        let crates_positions: Vec<usize> = after.rewritten.iter().map(|index| index.position()).collect();
                        let crates_content = crates_content_positions(&crates_containers, &after);
                        let found = if crates_positions != model_rewrites.position_rewritten_positions {
                            Some(format!("位置记法重写清单不同：crates {crates_positions:?}（长 {}），模型 {:?}（长 {}）", crates_positions.len(), model_rewrites.position_rewritten_positions, model_rewrites.position_rewritten_positions.len()))
                        } else if crates_content != model_rewrites.content_rewritten_positions {
                            Some(format!("内容记法不同：crates {crates_content:?}，模型 {:?}", model_rewrites.content_rewritten_positions))
                        } else if after.containers_dropped != model_rewrites.containers_dropped {
                            Some(format!("摘片数不同：crates {}，模型 {}", after.containers_dropped, model_rewrites.containers_dropped))
                        } else {
                            None
                        };
                        crates_containers = after.containers;
                        found
                    }
                }
            }
        };
        let difference = difference.or_else(|| {
            let (model_side, crates_side) = (model_shape(&model_tree), crates_shape(&crates_containers));
            (model_side != crates_side).then(|| format!("片的形态不同：模型 {} 片，crates {} 片；第一处不同的片 {:?}", model_side.len(), crates_side.len(), model_side.iter().zip(&crates_side).position(|(left, right)| left != right)))
        });
        if let Some(text) = difference {
            outcome.operations_compared = operation_number;
            outcome.first_difference = Some(format!("operation={operation_number} {operation:?} {text}"));
            return outcome;
        }
        outcome.operations_compared = operation_number;
    }
    outcome
}

/// `crates/` 单测钉住的三种形态在 `crates/` 那边现算的读数，写法与模型的 `crates_unit_test_readings` 逐字相同，好直接比。
fn crates_unit_test_readings_from_crates() -> Vec<String> {
    let five_hundred: Vec<InodeRecord> = (1..=500_u64).map(|inode| record(inode, FIRST_CHECKPOINT_TXG)).collect();
    let after_five_hundred =
        write_records_into_leaf_containers(&[], &five_hundred, CheckpointTxg(FIRST_CHECKPOINT_TXG), ORACLE_INODE_TREE).expect("500 条装得下");
    let sizes = after_five_hundred.containers.iter().map(|container| container.records.len().to_string()).collect::<Vec<_>>().join("/");
    let numbers = after_five_hundred.containers.iter().map(|container| container.identity.container.to_string()).collect::<Vec<_>>().join("/");
    let mut grown: Vec<InodeLeafContainer> = Vec::new();
    let mut refusal = String::from("never");
    for inode in 1..=(233 * 135 + 1_u64) {
        match write_records_into_leaf_containers(&grown, &[record(inode, FIRST_CHECKPOINT_TXG)], CheckpointTxg(FIRST_CHECKPOINT_TXG), ORACLE_INODE_TREE) {
            Ok(after) => grown = after.containers,
            Err(InodeTreeWriteRefusal::MoreLeafContainersThanOneRootNodeHolds { containers, capacity }) => {
                refusal = format!("refused_inode={inode} refused_containers={containers} capacity={capacity}");
                break;
            }
            Err(other) => panic!("只会被拒在 135 片那一格：{other:?}"),
        }
    }
    let first_three: Vec<InodeRecord> = (1..=233_u64).chain([234, 467]).map(|inode| record(inode, FIRST_CHECKPOINT_TXG)).collect();
    let mut three = write_records_into_leaf_containers(&[], &first_three[..233], CheckpointTxg(FIRST_CHECKPOINT_TXG), ORACLE_INODE_TREE)
        .expect("一满片")
        .containers;
    for inode in [234_u64, 467] {
        let mut container = three[0].clone();
        container.identity.container = inode;
        container.records = vec![record(inode, FIRST_CHECKPOINT_TXG)];
        three.push(container);
    }
    let dropped = remove_record_from_leaf_containers(&three, InodeNumber(234)).expect("234 在树里");
    let positions: Vec<usize> = dropped.rewritten.iter().map(|index| index.position()).collect();
    vec![
        format!("sizes={sizes} container_numbers={numbers}"),
        refusal,
        format!("position_rewritten={positions:?} dropped={}", dropped.containers_dropped),
    ]
}

fn main() {
    let threads = match std::env::var("SINGLEFS_THREAD_CAP") {
        Ok(text) => text.parse::<usize>().ok().filter(|threads| *threads >= 1).expect("SINGLEFS_THREAD_CAP 要是正整数"),
        Err(_) => 1,
    };
    eprintln!("E176 对拍：{threads} 个线程（线程数不进产物）");
    let mut lines = Vec::new();
    let crates_leaf_records = usize::try_from(INODE_LEAF_RECORDS).expect("233 装得进 usize");
    let crates_root_capacity = leaf_containers_one_root_node_holds();
    let constants_equal = LEAF_RECORD_CAPACITY == crates_leaf_records && ROOT_NODE_LEAF_CONTAINER_CAPACITY == crates_root_capacity;
    lines.push(format!(
        "name=oracle_constants model_leaf_record_capacity={LEAF_RECORD_CAPACITY} crates_inode_leaf_records={crates_leaf_records} \
         model_root_node_leaf_container_capacity={ROOT_NODE_LEAF_CONTAINER_CAPACITY} crates_leaf_containers_one_root_node_holds={crates_root_capacity} equal={constants_equal}"
    ));
    let model_readings: Vec<String> = model::crates_unit_test_readings().into_iter().map(|reading| reading.observed).collect();
    let crates_readings = crates_unit_test_readings_from_crates();
    let mut shapes_equal = true;
    for (model_reading, crates_reading) in model_readings.iter().zip(&crates_readings) {
        let equal = model_reading == crates_reading;
        shapes_equal &= equal;
        lines.push(format!("name=oracle_crates_unit_test_shape model={} crates={} equal={equal}", model_reading.replace(' ', ","), crates_reading.replace(' ', ",")));
    }
    shapes_equal &= model_readings.len() == crates_readings.len();
    let sequences = model::registered_sequences();
    let next_sequence = std::sync::atomic::AtomicUsize::new(0);
    let mut collected: Vec<(usize, OracleOutcome)> = std::thread::scope(|scope| {
        let workers: Vec<_> = (0..threads)
            .map(|_| {
                scope.spawn(|| {
                    let mut finished = Vec::new();
                    loop {
                        let sequence_index = next_sequence.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                        let Some(sequence) = sequences.get(sequence_index) else { break };
                        finished.push((sequence_index, compare_one_sequence(*sequence)));
                    }
                    finished
                })
            })
            .collect();
        workers.into_iter().flat_map(|worker| worker.join().expect("对拍线程不 panic")).collect()
    });
    collected.sort_by_key(|(sequence_index, _)| *sequence_index);
    assert_eq!(collected.len(), sequences.len(), "派出去几条序列就收回几条");
    let mut without_difference = 0;
    for (sequence_index, outcome) in &collected {
        if outcome.first_difference.is_none() && outcome.refused_operation.is_some() {
            without_difference += 1;
        }
        lines.push(format!(
            "name=oracle sequence={} operations_compared={} unlinks_compared={} m1_refused_operation={} first_difference={}",
            sequences[*sequence_index].name,
            outcome.operations_compared,
            outcome.unlinks_compared,
            outcome.refused_operation.map_or_else(|| "none".to_string(), |operation| operation.to_string()),
            outcome.first_difference.clone().unwrap_or_else(|| "none".to_string())
        ));
    }
    let all_equal = constants_equal && shapes_equal && without_difference == sequences.len();
    lines.push(format!(
        "name=verdict scope=oracle sequences={} sequences_compared_to_refusal_without_difference={without_difference} constants_equal={constants_equal} crates_unit_test_shapes_equal={shapes_equal} all_equal={all_equal}",
        sequences.len()
    ));
    let emitted = lines.len() + 1;
    for line in lines {
        println!("E7RESULT {line}");
    }
    println!("E7RESULT name=done emitted={emitted}");
    if !all_equal {
        eprintln!("对拍有不同（第十一节停机）：见 first_difference 不是 none 的那几行与 equal=false 的那几行");
        std::process::exit(1);
    }
}
