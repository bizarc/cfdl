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
    pub(crate) quantiles: Vec<IrQuantile>,
    /// Resolved quantile call sites, passed straight through to the results
    /// document. Opaque here on purpose: the compiler computed them and the
    /// engine has no reason to reinterpret them.
    #[serde(default)]
    pub(crate) quantile_inputs: Vec<serde_json::Value>,
    #[serde(default)]
    pub(crate) accounts: Vec<IrAccount>,
    /// Every machine an entity binds (`docs/28` §6.1). Absent for the many
    /// models that declare none.
    #[serde(default)]
    pub(crate) lifecycles: Vec<IrLifecycle>,
    #[serde(default)]
    pub(crate) waterfalls: Vec<IrWaterfall>,
    /// Declared metrics, in declaration order. Evaluated in the valuation
    /// plane, after every series they can read has settled.
    #[serde(default)]
    pub(crate) metrics: Vec<IrMetric>,
    /// Views — lenses on a completed result, outside the model's identity.
    /// The engine still EVALUATES a slice, because the valuation plane is its
    /// stage (`docs/28` §2); what changed is which part of the document the
    /// declaration is read from.
    #[serde(default)]
    pub(crate) views: IrViews,
    /// Declared entities. Read so an entity's lifecycle STARTS where the model
    /// says rather than at null — the totality the ontology exists to give.
    #[serde(default)]
    pub(crate) entities: Vec<IrEntityDecl>,
    /// Run modes the model declares for itself. A `run monte_carlo trials N
    /// seed S` in source used to be parsed, lowered, and then dropped here, so
    /// the model asked for trials and got a single deterministic pass.
    #[serde(default)]
    pub(crate) runs: Vec<IrRun>,
    /// Declared contracts, resolved by the compiler to their type and master
    /// (docs/40 §8). Read only to be republished in the results graph: the
    /// engine lowers nothing from them — their cash arrived as streams.
    #[serde(default)]
    pub(crate) contracts: Vec<IrContractDecl>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct IrContractDecl {
    pub(crate) name: String,
    #[serde(rename = "type")]
    pub(crate) type_id: String,
    #[serde(default)]
    pub(crate) master: Option<String>,
    #[serde(default)]
    pub(crate) contract_name: Option<String>,
    #[serde(default)]
    pub(crate) instance: Option<String>,
    pub(crate) subject: IrEntityRef,
    #[serde(default)]
    pub(crate) parties: Vec<IrContractParty>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct IrContractParty {
    pub(crate) role: String,
    #[serde(default)]
    pub(crate) master_role: Option<String>,
    pub(crate) entity: IrEntityRef,
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

/// A quantile as the compiler emitted it: always ascending by share, whatever
/// the source declared.
#[derive(Debug, Deserialize)]
pub(crate) struct IrQuantile {
    pub(crate) name: String,
    #[serde(default = "default_interpolation")]
    pub(crate) interpolation: String,
    pub(crate) points: Vec<IrQuantilePoint>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct IrQuantilePoint {
    pub(crate) share: f64,
    pub(crate) value: f64,
}

#[derive(Debug, Deserialize)]
pub(crate) struct IrEvent {
    pub(crate) name: String,
    /// The occurrences this event is tested at (`docs/34` D1a). Absent means
    /// the condition's own rising edges supply them.
    #[serde(default)]
    pub(crate) schedule: Option<IrSchedule>,
    /// Absent means every scheduled occurrence fires. At least one of
    /// `schedule` and `when` is present, which the compiler enforces.
    #[serde(default)]
    pub(crate) when: Option<IrExpr>,
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
/// A declared cash location whose balance carries across periods.
///
/// `available` is unchanged and still means this period's netted cash; an
/// account is the ACCUMULATED cash. There is no currency: an account is
/// denominated by the model.
pub(crate) struct IrAccount {
    pub(crate) name: String,
    /// Read when a step allocates into a party's account, which is the next
    /// increment. It is carried here regardless because the IR is a PUBLISHED
    /// contract: a consumer reading the document can see who an account
    /// belongs to whether or not this engine has a use for it yet.
    #[allow(dead_code)]
    #[serde(default)]
    pub(crate) owner: Option<String>,
    #[serde(default)]
    pub(crate) inflow: Option<IrExpr>,
}

/// A declared finite state machine. The states are enumerated; the edges are
/// declared only as used, and an undeclared edge does not exist. A guard-less
/// edge is a permission an event's write may take; a guarded edge is
/// evaluated each period the entity is in `from` — there is no latch, because
/// edge availability is the memory (`docs/28` §6.1).
#[derive(Debug, Deserialize)]
pub(crate) struct IrLifecycle {
    pub(crate) id: String,
    pub(crate) initial: String,
    /// Validated at compile (`E1316`); carried because the IR is a published
    /// contract and a consumer reads the set whether or not this engine
    /// needs it again.
    #[allow(dead_code)]
    #[serde(default)]
    pub(crate) states: Vec<String>,
    #[serde(default)]
    pub(crate) edges: Vec<IrLifecycleEdge>,
    /// What is true of a STATE however it was reached. Runs BEFORE the taken
    /// edge's actions — the state's own setup, then the path's refinement.
    #[serde(default)]
    pub(crate) entry_actions: Vec<IrStateEntry>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct IrLifecycleEdge {
    pub(crate) from: String,
    pub(crate) to: String,
    #[serde(default)]
    pub(crate) guard: Option<IrExpr>,
    /// What is true of the PATH taken, on every traversal.
    #[serde(default)]
    pub(crate) actions: Vec<IrStateAction>,
}

/// One state's arrival actions.
#[derive(Debug, Deserialize)]
pub(crate) struct IrStateEntry {
    pub(crate) state: String,
    #[serde(default)]
    pub(crate) actions: Vec<IrStateAction>,
}

/// One arrival action, carrying who wrote it. The journal names the author,
/// so a same-field conflict records whose value was `overridden`.
#[derive(Debug, Deserialize)]
pub(crate) struct IrStateAction {
    #[allow(dead_code)]
    pub(crate) kind: String,
    pub(crate) author: String,
    pub(crate) field: String,
    pub(crate) value: IrExpr,
}

#[derive(Debug, Deserialize)]
pub(crate) struct IrMetric {
    pub(crate) name: String,
    pub(crate) expr: IrExpr,
}

#[derive(Debug, Deserialize, Default)]
pub(crate) struct IrViews {
    #[serde(default)]
    pub(crate) slices: Vec<IrSlice>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct IrSlice {
    pub(crate) name: String,
    #[serde(default)]
    pub(crate) entities: Vec<String>,
    #[serde(default)]
    pub(crate) types: Vec<String>,
    /// Lines by role, as declared — lineage; the compiler expanded them
    /// into `type_streams` beside the types.
    #[serde(default)]
    pub(crate) lines: Vec<String>,
    #[serde(default)]
    pub(crate) categories: Vec<String>,
    #[serde(default)]
    pub(crate) streams: Vec<String>,
    #[serde(default)]
    pub(crate) except_streams: Vec<String>,
    #[serde(default)]
    pub(crate) except_categories: Vec<String>,
    #[serde(default)]
    pub(crate) except_entities: Vec<String>,
    /// The `type` clauses' expansion, resolved by the compiler — the engine
    /// is pack-free and cannot walk the ontology itself.
    #[serde(default)]
    pub(crate) type_streams: Vec<String>,
    /// A reporting window, inclusive. Absent when the slice spans the whole
    /// horizon.
    #[serde(default)]
    pub(crate) window: Option<IrDateRange>,
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
    /// The payee is an ACCOUNT rather than a party.
    ///
    /// Read when the waterfall stage runs at its own period and can move a
    /// balance; carried here regardless because the IR is a published
    /// contract, and a consumer can see where a step allocates whether or not
    /// this engine acts on it yet.
    #[allow(dead_code)]
    #[serde(default)]
    pub(crate) payee_is_account: bool,
    /// The agreement and line this step pays, when the model says so
    /// (docs/40 §6). Republished on the step's series and in the graph.
    #[serde(default)]
    pub(crate) contract: Option<String>,
    #[serde(default)]
    pub(crate) line: Option<String>,
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
    /// The ontology type the declaration states, republished into the
    /// results graph (docs/13 §7.43).
    #[serde(rename = "type", default)]
    pub(crate) type_id: Option<String>,
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
    /// The machine this entity is governed by — an id into `lifecycles`.
    #[serde(default)]
    pub(crate) lifecycle: Option<String>,
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

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct IrOnRule {
    pub(crate) kind: String,
    #[serde(default)]
    pub(crate) day: i32,
}

/// Where in its period a flow sits. The single axis that `due` / `mid` /
/// `at_period_end` used to spell as three booleans.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub(crate) enum Placement {
    Start,
    Mid,
    End,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub(crate) struct IrSchedule {
    pub(crate) kind: String,
    pub(crate) on: Option<String>,
    #[serde(default)]
    pub(crate) every: Option<String>,
    /// WHERE IN ITS PERIOD THE FLOW SITS. One axis with three positions, not
    /// three independent flags: `start` and `mid` and `end` are mutually
    /// exclusive by construction, so the conflicting-placement state cannot
    /// be written rather than being rejected at run time.
    ///
    /// Absent means the FORM's default, which differs and cannot be a single
    /// constant: a one-shot defaults to its period's start (right for an
    /// acquisition), a recurrence to its period's end (an ordinary annuity —
    /// the interval elapses, then payment falls). `placement_of` is the one
    /// place that resolution lives.
    #[serde(default)]
    pub(crate) placement: Option<Placement>,
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
    /// `state_enter` anchor (`docs/28` §6.2), present only for kind
    /// "StateEnter": each ENTRY of the entity into the state opens its own
    /// window of `anchor_periods` grid periods, resolved during the walk,
    /// re-anchoring on re-entry.
    #[serde(default)]
    pub(crate) anchor_entity: Option<String>,
    #[serde(default)]
    pub(crate) anchor_state: Option<String>,
    #[serde(default)]
    pub(crate) anchor_periods: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct IrExpr {
    #[serde(default)]
    pub(crate) lang: Option<String>,
    pub(crate) src: String,
}
