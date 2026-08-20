// Extracted from lib.rs — one stage of the engine. See docs/13 §7.44.
use super::*;

#[derive(Debug, Deserialize)]
pub(crate) struct Ir {
    pub(crate) model: IrModel,
    pub(crate) time: IrTime,
    #[serde(default)]
    pub(crate) streams: Vec<IrStream>,
    /// Per-stream record of what each pack rule consumed. Deserialized as
    /// opaque JSON and republished verbatim: the engine has no use for it, and
    /// giving it a typed shape here would mean maintaining that shape in two
    /// crates for no gain.
    #[serde(default)]
    pub(crate) stream_inputs: Vec<serde_json::Value>,
    /// Per-period subtotals, in dependency order. The compiler has already
    /// rejected forward references, so a lookup here always finds something
    /// already computed.
    #[serde(default)]
    pub(crate) subtotals: Vec<IrSubtotal>,
    #[serde(default)]
    pub(crate) assumptions: IrAssumptions,
    #[serde(default)]
    pub(crate) events: Vec<IrEvent>,
    #[serde(default)]
    pub(crate) options: Vec<IrOption>,
    #[serde(default)]
    pub(crate) phases: Vec<IrPhase>,
    #[serde(default)]
    pub(crate) curves: Vec<IrCurve>,
    #[serde(default)]
    pub(crate) waterfalls: Vec<IrWaterfall>,
    /// Declared entities. Read so an entity's lifecycle STARTS where the model
    /// says rather than at null — the totality the ontology exists to give.
    #[serde(default)]
    pub(crate) entities: Vec<IrEntityDecl>,
    /// Run modes the model declares for itself. A `run monte_carlo trials N
    /// seed S` in source used to be parsed, lowered, and then dropped here, so
    /// the model asked for trials and got a single deterministic pass.
    #[serde(default)]
    pub(crate) runs: Vec<IrRun>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct IrCurve {
    pub(crate) name: String,
    /// "step" (flat-forward) or "linear".
    #[serde(default = "default_interpolation")]
    pub(crate) interpolation: String,
    pub(crate) points: Vec<IrCurvePoint>,
}

pub(crate) fn default_interpolation() -> String {
    "step".to_string()
}

#[derive(Debug, Deserialize)]
pub(crate) struct IrCurvePoint {
    pub(crate) date: String,
    pub(crate) value: f64,
}

#[derive(Debug, Deserialize)]
pub(crate) struct IrEvent {
    pub(crate) name: String,
    pub(crate) when: IrExpr,
    #[serde(default)]
    pub(crate) actions: Vec<IrAction>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct IrAction {
    pub(crate) kind: String,
    #[serde(default)]
    pub(crate) entity: Option<IrEntityRef>,
    #[serde(default)]
    pub(crate) field: Option<String>,
    #[serde(default)]
    pub(crate) value: Option<IrExpr>,
    #[serde(default)]
    pub(crate) stream: Option<String>,
    #[serde(default)]
    #[allow(dead_code)] // deserialized for contract actions, which warn-and-skip for now
    pub(crate) contract: Option<String>,
    #[serde(default)]
    pub(crate) option: Option<String>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct IrOption {
    pub(crate) name: String,
    pub(crate) exercise_when: IrExpr,
    pub(crate) payoff: IrExpr,
    #[serde(default)]
    pub(crate) exercisable_in_phase: Option<String>,
    /// The asset the option is written on. An option is a contract, so it has
    /// one; with it, `entity.<field>` in a guard means the same thing it means
    /// in a stream.
    #[serde(default)]
    pub(crate) owner: Option<IrEntityRef>,
}

/// Only the part the engine needs: whether a pack generated this stream.
#[derive(Debug, Deserialize)]
pub(crate) struct IrProvenance {
    #[serde(default)]
    pub(crate) generated_by: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct IrWaterfall {
    pub(crate) name: String,
    pub(crate) entity: String,
    #[serde(default)]
    pub(crate) schedule: Option<IrSchedule>,
    pub(crate) source: IrExpr,
    #[serde(default)]
    pub(crate) steps: Vec<IrWaterfallStep>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct IrWaterfallStep {
    pub(crate) name: String,
    pub(crate) payee: String,
    pub(crate) amount: IrExpr,
}

#[derive(Debug, Deserialize)]
pub(crate) struct IrFieldRule {
    pub(crate) init: IrExpr,
    pub(crate) next: IrExpr,
    /// A field a CONTRACT brought carries that contract's rhythm: a
    /// monthly-paying pool on a daily book compounds twelve times a year, not
    /// 365. A field a modeller wrote has none and steps every period.
    #[serde(default)]
    pub(crate) schedule: Option<IrSchedule>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct IrEntityDecl {
    pub(crate) symbol: String,
    /// Declared attributes — `rentable_area = 30000`.
    ///
    /// These were parsed, validated against the ontology, and carried into the
    /// IR, and then the engine deserialised right past them: an attribute read
    /// as ZERO in every expression that touched it. The ontology checked the
    /// name and nothing checked the value ever arrived.
    ///
    /// Carried as strings because that is the IR's shape; parsed to a number
    /// where they look like one, so arithmetic works and a label stays a label.
    #[serde(default)]
    pub(crate) fields: BTreeMap<String, String>,
    /// Fields that MOVE. A recurrence owned by this entity, evaluated in the
    /// same pass as a declared state because it is the same construct.
    #[serde(default)]
    pub(crate) rules: BTreeMap<String, IrFieldRule>,
    /// The lifecycle state this entity opens in. `None` when its type declares
    /// no lifecycle, which is most entities.
    #[serde(default)]
    pub(crate) initial_state: Option<String>,
    /// The entity this one is part of, when the model groups it. Absent for
    /// most entities: hierarchy is available at every grain and required at
    /// none.
    #[serde(default)]
    pub(crate) parent: Option<String>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct IrPhase {
    pub(crate) name: String,
    pub(crate) range: IrDateRange,
}

#[derive(Debug, Deserialize)]
pub(crate) struct IrDateRange {
    pub(crate) start: String,
    pub(crate) end: String,
}

#[derive(Debug, Deserialize)]
pub(crate) struct IrRun {
    pub(crate) kind: String,
    #[serde(default)]
    pub(crate) trials: Option<u32>,
    #[serde(default)]
    pub(crate) seed: Option<u64>,
}

#[derive(Debug, Default, Deserialize)]
pub(crate) struct IrAssumptions {
    #[serde(default)]
    pub(crate) constants: BTreeMap<String, IrAssumeConstant>,
    #[serde(default)]
    pub(crate) random: BTreeMap<String, IrAssumeRandom>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct IrAssumeConstant {
    pub(crate) expr: IrExpr,
}

#[derive(Debug, Deserialize)]
pub(crate) struct IrAssumeRandom {
    pub(crate) dist: IrDistribution,
}

#[derive(Debug, Deserialize)]
pub(crate) struct IrDistribution {
    pub(crate) kind: String,
    #[serde(default)]
    pub(crate) params: BTreeMap<String, f64>,
    #[serde(default)]
    pub(crate) clip: Option<[f64; 2]>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct IrModel {
    #[serde(default)]
    pub(crate) name: Option<String>,
    pub(crate) currency: String,
}

#[derive(Debug, Deserialize)]
pub(crate) struct IrTime {
    pub(crate) calendar: String,
    pub(crate) start: String,
    pub(crate) periods: u32,
    #[serde(default)]
    pub(crate) projection: u32,
}

#[derive(Debug, Deserialize)]
pub(crate) struct IrStream {
    pub(crate) name: String,
    pub(crate) owner: IrEntityRef,
    pub(crate) direction: String,
    /// What this stream is, economically. The fold layer aggregates on this
    /// rather than on the name — the one field of stream metadata the engine
    /// genuinely needs, read once per stream rather than per period.
    #[serde(default)]
    pub(crate) category: Option<String>,
    pub(crate) schedule: IrSchedule,
    pub(crate) amount: IrExpr,
    #[serde(default)]
    pub(crate) active_when: Option<IrExpr>,
    /// Read for one purpose: telling a pack's expression from a modeller's.
    #[serde(default)]
    pub(crate) provenance: Option<IrProvenance>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct IrSubtotal {
    pub(crate) id: String,
    // `kind` is in the IR but not read here: `op` already determines the shape
    // (a sum is money, a ratio is a number), and the pack loader has rejected
    // any spec where the two disagree. Deserializing a field only to ignore it
    // would be the kind of accepted-and-discarded the repo rejects elsewhere.
    pub(crate) op: String,
    #[serde(default)]
    pub(crate) categories: Vec<String>,
    #[serde(default)]
    pub(crate) streams: Vec<String>,
    #[serde(default)]
    pub(crate) subtotals: Vec<String>,
    #[serde(default)]
    pub(crate) numerator: Option<String>,
    #[serde(default)]
    pub(crate) denominator: Option<String>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct IrEntityRef {
    pub(crate) symbol: String,
}

#[derive(Debug, Deserialize)]
pub(crate) struct IrOnRule {
    pub(crate) kind: String,
    #[serde(default)]
    pub(crate) day: i32,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub(crate) struct IrSchedule {
    pub(crate) kind: String,
    pub(crate) on: Option<String>,
    #[serde(default)]
    pub(crate) every: Option<String>,
    /// Annuity due: payment at the start of each interval. Absent means an
    /// ordinary annuity — the interval elapses, then payment falls.
    #[serde(default)]
    pub(crate) due: bool,
    /// A one-shot flow that settles at the END of its period.
    #[serde(default)]
    pub(crate) at_period_end: bool,
    /// Mid-period convention: cash treated as arriving halfway through the
    /// period it was earned in. A discounting convention rather than a date,
    /// so it is 0.5 of a period on every calendar.
    #[serde(default)]
    pub(crate) mid: bool,
    /// How long after a flow is earned its cash moves. Absent means the cash
    /// lands in the period that earned it.
    #[serde(default)]
    pub(crate) net_days: Option<i64>,
    #[serde(default)]
    pub(crate) net_months: Option<i64>,
    pub(crate) from: Option<String>,
    pub(crate) to: Option<String>,
    /// Places an occurrence within its interval (`on day <n>` / `on eom`).
    /// Previously not even deserialized, so the compiler emitted it and the
    /// engine dropped it — `on day 15` had no effect on any cash flow.
    #[serde(default)]
    pub(crate) on_rule: Option<IrOnRule>,
    #[serde(default)]
    pub(crate) phase: Option<String>,
    #[serde(default)]
    pub(crate) convention: Option<String>,
    #[serde(default)]
    pub(crate) calendar: Option<String>,
    #[serde(default)]
    pub(crate) except_dates: Vec<String>,
    #[serde(default)]
    pub(crate) also_dates: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct IrExpr {
    #[serde(default)]
    pub(crate) lang: Option<String>,
    pub(crate) src: String,
}
