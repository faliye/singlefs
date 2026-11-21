//! 一个项的发布与结算与操作与重挂载记录、被量的项、目标、从字段读出各记录、读者与读项、操作的步与行。

use super::*;

// ---- 产物的读法 ----

/// 一次发布（固定点）的节点读数：`name=nodes` 行与同一次的 `name=pointer_graph` 行。
#[derive(Clone, Debug, Default)]
pub(super) struct PublishRecord {
    /// `user`、`write_row`、`warm_up`。
    pub(super) kind: String,
    pub(super) order: u64,
    pub(super) settle_call: u64,
    pub(super) site: String,
    pub(super) seed: NodeSet,
    pub(super) pulled: NodeSet,
    pub(super) final_set: NodeSet,
    pub(super) released_records_pre: u64,
    pub(super) graph: PointerGraph,
    pub(super) root_level: u32,
    /// 走链读数自查（边的重走与实现并进的结果相同）。
    pub(super) walk_matches: bool,
    pub(super) edges: Vec<(Node, Node)>,
}

/// 一次固定点的读数（`name=settle` 行）：旗吃 `final_nodes`。
#[derive(Clone, Debug, Default)]
pub(super) struct SettleReading {
    pub(super) settle_call: u64,
    judgement: u64,
    pub(super) rounds: u64,
    outcome: String,
    pub(super) final_nodes: u64,
    /// 每一轮：(出口, 这一轮各行步数之和)。
    pub(super) rounds_detail: Vec<(String, u64)>,
}

/// 一次被量的用户操作（声明行起头）。
#[derive(Clone, Debug, Default)]
pub(super) struct Operation {
    pub(super) epoch: u64,
    pub(super) step_index: u64,
    pub(super) name: String,
    pub(super) user_units: u64,
    pub(super) old_units: u64,
    pub(super) new_units: u64,
    pub(super) settles: Vec<SettleReading>,
    pub(super) publishes: Vec<PublishRecord>,
}

/// 重挂载自己的发布（写行一次、暖机若干次）。
#[derive(Clone, Debug, Default)]
pub(super) struct RemountRecord {
    pub(super) epoch: u64,
    pub(super) step_index: u64,
    pub(super) settles: Vec<SettleReading>,
    pub(super) publishes: Vec<PublishRecord>,
}

/// 一份产物：一个项。
#[derive(Clone, Debug, Default)]
pub(super) struct MeasuredItem {
    pub(super) name: String,
    pub(super) point: String,
    pub(super) arm: String,
    pub(super) script: String,
    pub(super) build: String,
    pub(super) files: u64,
    pub(super) units_per_file: u64,
    pub(super) unit_area_slots: u64,
    pub(super) constants: Fields,
    pub(super) history_user_changes: Option<u64>,
    pub(super) history_settle_calls: u64,
    pub(super) history_carried_final: NodeSet,
    pub(super) history_carried_pulled: NodeSet,
    /// `script_step` 行：(第几步, 步名, 挂载序号)。
    pub(super) step_lines: Vec<(u64, String, u64)>,
    pub(super) operations: Vec<Operation>,
    pub(super) remounts: Vec<RemountRecord>,
    pub(super) images: Vec<(String, String)>,
    pub(super) judgements: BTreeMap<String, JudgementReadings>,
    pub(super) first_state: Fields,
    pub(super) is_twin: bool,
    pub(super) done: bool,
    pub(super) refused: bool,
    /// 去掉产物头之后的全部读数行（确定性孪生项逐字节对拍用）。
    pub(super) body_lines: Vec<String>,
}

/// 读数行当前挂在哪个对象上：`settle` 之后的 `round`、`round_row` 行要接到它的最后一次固定点。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Target {
    Operation(usize),
    Remount(usize),
}

fn text_of(fields: &Fields, key: &str) -> String {
    fields.get(key).cloned().unwrap_or_default()
}

fn value_of(fields: &Fields, key: &str) -> u64 {
    number(fields, key).unwrap_or(0)
}

fn publish_from_fields(fields: &Fields) -> PublishRecord {
    let set = |key: &str| parse_node_set(&text_of(fields, key)).unwrap_or_default();
    let edges = text_of(fields, "edges")
        .split(',')
        .filter_map(|edge| edge.split_once('>'))
        .filter_map(|(from, to)| Some((parse_node(from)?, parse_node(to)?)))
        .collect();
    PublishRecord {
        kind: text_of(fields, "publish_kind"),
        order: value_of(fields, "publish_order"),
        settle_call: value_of(fields, "settle_call"),
        site: text_of(fields, "first_merge_site"),
        seed: set("seed"),
        pulled: set("pulled"),
        final_set: set("final"),
        released_records_pre: value_of(fields, "released_records_pre"),
        walk_matches: text_of(fields, "edges_walk_matches_the_implementation") == "1",
        edges,
        ..PublishRecord::default()
    }
}

/// `graph=节点@盘.槽+盘.槽;…` 的读法：节点与两份旧落点。读不出的交 `None`。
pub(super) fn parse_pointer_graph(text: &str) -> Option<PointerGraph> {
    if text == "-" {
        return Some(PointerGraph::new());
    }
    let mut graph = PointerGraph::new();
    for entry in text.split(',') {
        let (node, places) = entry.split_once('@')?;
        let locations: Option<Vec<(u32, u64)>> = places
            .split('+')
            .map(|place| {
                let (device, slot) = place.split_once('.')?;
                Some((device.parse().ok()?, slot.parse().ok()?))
            })
            .collect();
        graph.insert(parse_node(node)?, locations?);
    }
    Some(graph)
}

fn settle_from_fields(fields: &Fields) -> SettleReading {
    SettleReading {
        settle_call: value_of(fields, "settle_call"),
        judgement: value_of(fields, "judgement"),
        rounds: value_of(fields, "rounds"),
        outcome: text_of(fields, "outcome"),
        final_nodes: value_of(fields, "final_nodes"),
        rounds_detail: Vec::new(),
    }
}

/// 读数行里标「第几次挂载、哪一步、第几步」的三个键拼成的判键前缀（E203 的 `state`、`row`、`delta`、`publish` 行按它归一次判）。
fn judgement_prefix(fields: &Fields) -> String {
    format!("{}/{}/{}/", value_of(fields, "mount_epoch"), text_of(fields, "operation"), value_of(fields, "operation_index"))
}

struct Reader {
    pub(super) item: MeasuredItem,
    current: Option<Target>,
    last_publish: Option<Target>,
}

impl Reader {
    fn operation_index(&self, fields: &Fields) -> Option<usize> {
        let step_index = value_of(fields, "operation_index");
        self.item.operations.iter().position(|operation| operation.step_index == step_index)
    }

    fn remount_index(&self, fields: &Fields) -> Option<usize> {
        let epoch = value_of(fields, "mount_epoch");
        self.item.remounts.iter().position(|remount| remount.epoch == epoch)
    }

    fn target_of(&mut self, fields: &Fields) -> Option<Target> {
        if text_of(fields, "phase") == "remount" {
            let position = self.remount_index(fields).unwrap_or_else(|| {
                self.item.remounts.push(RemountRecord {
                    epoch: value_of(fields, "mount_epoch"),
                    step_index: value_of(fields, "operation_index"),
                    ..RemountRecord::default()
                });
                self.item.remounts.len() - 1
            });
            Some(Target::Remount(position))
        } else {
            self.operation_index(fields).map(Target::Operation)
        }
    }

    fn settles_of(&mut self, target: Target) -> &mut Vec<SettleReading> {
        match target {
            Target::Operation(position) => &mut self.item.operations[position].settles,
            Target::Remount(position) => &mut self.item.remounts[position].settles,
        }
    }

    fn publishes_of(&mut self, target: Target) -> &mut Vec<PublishRecord> {
        match target {
            Target::Operation(position) => &mut self.item.operations[position].publishes,
            Target::Remount(position) => &mut self.item.remounts[position].publishes,
        }
    }

    fn take_line(&mut self, fields: &Fields) {
        let name = fields.get("name").map_or("", String::as_str);
        match name {
            "header" => self.item.is_twin = text_of(fields, "twin") == "yes",
            "point" => {
                let item = &mut self.item;
                item.name = text_of(fields, "item");
                item.point = text_of(fields, "point");
                item.arm = text_of(fields, "arm");
                item.script = text_of(fields, "script");
                item.build = text_of(fields, "build");
                item.files = value_of(fields, "files");
                item.units_per_file = value_of(fields, "units_per_small_file");
                item.unit_area_slots = value_of(fields, "unit_area_slots");
            }
            "constants" => self.item.constants = fields.clone(),
            "history" => {
                self.item.history_user_changes = number(fields, "user_changes");
                self.item.history_settle_calls = value_of(fields, "settle_calls");
            }
            "history_carried" => {
                self.item.history_carried_final = parse_node_set(&text_of(fields, "ever_final")).unwrap_or_default();
                self.item.history_carried_pulled = parse_node_set(&text_of(fields, "ever_pulled")).unwrap_or_default();
            }
            "image_sha256" => self.item.images.push((text_of(fields, "when"), text_of(fields, "sha256"))),
            "script_step" => self.item.step_lines.push((value_of(fields, "step_index"), text_of(fields, "operation"), value_of(fields, "mount_epoch"))),
            "declared" => {
                self.item.operations.push(Operation {
                    epoch: value_of(fields, "mount_epoch"),
                    step_index: value_of(fields, "operation_index"),
                    name: text_of(fields, "operation"),
                    user_units: value_of(fields, "user_units"),
                    old_units: value_of(fields, "old_units"),
                    new_units: value_of(fields, "new_units"),
                    ..Operation::default()
                });
            }
            "settle" => {
                if let Some(target) = self.target_of(fields) {
                    self.settles_of(target).push(settle_from_fields(fields));
                    self.current = Some(target);
                }
            }
            "round" => {
                if let Some(target) = self.current {
                    if let Some(settle) = self.settles_of(target).last_mut() {
                        settle.rounds_detail.push((text_of(fields, "round_exit"), 0));
                    }
                }
            }
            "round_row" => {
                if let Some(target) = self.current {
                    if let Some(round) = self.settles_of(target).last_mut().and_then(|settle| settle.rounds_detail.last_mut()) {
                        round.1 += value_of(fields, "steps");
                    }
                }
            }
            "nodes" => {
                if let Some(target) = self.target_of(fields) {
                    self.publishes_of(target).push(publish_from_fields(fields));
                    self.last_publish = Some(target);
                }
            }
            "pointer_graph" => {
                if let Some(target) = self.last_publish {
                    let graph = parse_pointer_graph(&text_of(fields, "graph"));
                    let root_level = u32::try_from(value_of(fields, "root_level")).unwrap_or(0);
                    if let Some(publish) = self.publishes_of(target).last_mut() {
                        publish.graph = graph.unwrap_or_default();
                        publish.root_level = root_level;
                    }
                }
            }
            "state" => {
                let prefix = judgement_prefix(fields);
                if self.item.first_state.is_empty() {
                    self.item.first_state = fields.clone();
                }
                self.item.judgements.entry(prefix).or_default().state = fields.clone();
            }
            "row" => {
                let cell = self.item.judgements.entry(text_of(fields, "publish")).or_default();
                let key = (text_of(fields, "segment"), text_of(fields, "row"));
                cell.rows.insert(key.clone(), (value_of(fields, "calls"), value_of(fields, "query_levels")));
                cell.row_steps.insert(key, value_of(fields, "steps"));
            }
            "delta" => {
                self.item.judgements.entry(text_of(fields, "publish")).or_default().deltas.push((
                    text_of(fields, "segment"),
                    value_of(fields, "fixed_point_round"),
                    text_of(fields, "key"),
                    value_of(fields, "value"),
                ));
            }
            "publish" => {
                self.item.judgements.entry(text_of(fields, "publish")).or_default().fixed_point_rounds = number(fields, "fixed_point_rounds");
            }
            "history_refused" | "window_refused" => self.item.refused = true,
            "item_done" => self.item.done = true,
            _ => {}
        }
    }
}

/// 读一份产物的正文：每个项一份，`E209RESULT` 行以外的行不读。
pub(super) fn read_item_text(text: &str) -> Result<MeasuredItem, String> {
    let mut reader = Reader { item: MeasuredItem::default(), current: None, last_publish: None };
    for line in text.lines().filter(|line| line.starts_with("E209RESULT ")) {
        let fields = fields_of(line);
        reader.take_line(&fields);
        if fields.get("name").map(String::as_str) != Some("header") && fields.get("name").map(String::as_str) != Some("source_sha256") && !line.starts_with("E209RESULT name=anchor") {
            reader.item.body_lines.push(line.to_string());
        }
    }
    if reader.item.name.is_empty() {
        return Err("产物里没有 name=point 行".to_string());
    }
    Ok(reader.item)
}

pub(super) fn read_item(path: &str) -> Result<MeasuredItem, String> {
    let text = std::fs::read_to_string(path).map_err(|error| format!("读不了 {path}：{error}"))?;
    read_item_text(&text).map_err(|error| format!("{path}：{error}"))
}

/// 一次判的固定点的 B8 合计（三段之和）；读不到交 `None`。
fn judgement_of(item: &MeasuredItem, operation: &Operation, settle: &SettleReading) -> Option<JudgementReadings> {
    let key = format!("{}/{}/{}/{}", operation.epoch, operation.name, operation.step_index, settle.judgement);
    let prefix = judgement_prefix_of(operation);
    let mut judgement = item.judgements.get(&key)?.clone();
    judgement.state = item.judgements.get(&prefix)?.state.clone();
    Some(judgement)
}

fn judgement_prefix_of(operation: &Operation) -> String {
    format!("{}/{}/{}/", operation.epoch, operation.name, operation.step_index)
}

/// 一个操作各次固定点的 B8 合计之和；有一次读不到整个操作交 `None`（不当 0）。
fn steps_of_the_operation(item: &MeasuredItem, operation: &Operation) -> Option<u64> {
    rows_of_the_operation(item, operation).map(|rows| rows.values().sum())
}

/// 一个操作各次固定点三段里 B8 每一行的步数之和（行名 → 步数）。
pub(super) fn rows_of_the_operation(item: &MeasuredItem, operation: &Operation) -> Option<BTreeMap<&'static str, u64>> {
    let mut rows: BTreeMap<&'static str, u64> = BTreeMap::new();
    for settle in &operation.settles {
        let judgement = judgement_of(item, operation, settle)?;
        for segment in SEGMENTS_BEFORE_ADMISSION {
            let input = by_delta_input(&judgement, segment, item.unit_area_slots).ok()?;
            for (row, steps) in by_delta_rows(&input) {
                *rows.entry(row).or_insert(0) += steps;
            }
        }
    }
    Some(rows)
}
