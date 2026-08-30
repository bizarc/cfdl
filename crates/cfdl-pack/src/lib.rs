use serde::Deserialize;
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackLoadError {
    pub message: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PackRegistry {
    packs: BTreeMap<String, LoadedPack>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LoadedPack {
    pub manifest: PackManifest,
    pub aliases: BTreeMap<String, String>,
    pub templates: Vec<PackTemplate>,
    pub lowering_rules: Vec<LoweringRule>,
    pub metric_specs: Vec<MetricSpec>,
    pub subtotal_specs: Vec<SubtotalSpec>,
    pub statement_specs: Vec<StatementSpec>,
    pub validations: Vec<PackValidation>,
    /// What a model using this pack may be ABOUT — the assets, parties,
    /// contract types, lifecycles and references it can declare, and how they
    /// relate. Empty when the pack declares no ontology, which keeps every
    /// existing pack loading unchanged.
    pub ontology: PackOntology,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct PackManifest {
    pub name: String,
    pub version: String,
    #[serde(default)]
    pub description: Option<String>,
    /// Model calendars this pack's rules lower correctly on.
    ///
    /// Empty means all of them, so a pack that says nothing is unconstrained
    /// and third-party packs are unaffected. A pack whose expressions assume
    /// one period is one month — dividing annual figures by a literal 12 —
    /// must say `cadences = ["monthly"]`, or it will silently produce amounts
    /// scaled to the wrong period on any other grid.
    ///
    /// This is a migration scaffold, not a statement about what a pack can
    /// ever do: it is removed as each rule becomes cadence-neutral.
    #[serde(default)]
    pub cadences: Vec<String>,
    /// The categories this pack's streams may be classified into: a closed
    /// vocabulary of dotted **paths**, rooted in the cash flow statement's
    /// three sections.
    ///
    /// ```text
    /// operating.revenue.base_rent
    /// operating.deduction.vacancy
    /// investing.capital.leasing
    /// financing.debt.service
    /// ```
    ///
    /// Hierarchical rather than flat because every system that solves this
    /// converged on the same shape — IAS 7 and ASC 230's three sections, a
    /// chart of accounts' five root types, beancount's `Expenses:Rent:Office`,
    /// XBRL's calculation linkbase. A small universal root, then an arbitrary
    /// domain tree beneath it, with the rollup defined by the tree.
    ///
    /// The payoff is that a subtotal is a PREFIX QUERY over the same selector
    /// streams already use (`cfdl_expr::selector_matches`): NOI is
    /// `operating.*`, effective gross income is `operating.revenue.*` plus
    /// `operating.deduction.*`. No second matching mechanism, and a generic
    /// statement works against a pack it has never seen.
    ///
    /// The ROOT is enforced (`operating`, `investing`, `financing`); which root
    /// a given category takes is the pack's call, because that genuinely varies:
    /// interest paid is operating under IFRS and financing under US GAAP, and
    /// for a lender the interest RECEIVED on a pool is operating revenue. CFDL
    /// fixes the vocabulary of sections, not the accounting policy.
    ///
    /// Empty means the pack does not classify, and every rule's `category` must
    /// then be empty too.
    #[serde(default)]
    pub categories: Vec<String>,
    #[serde(default)]
    pub entrypoints: PackEntrypoints,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
pub struct PackEntrypoints {
    #[serde(default)]
    pub aliases: Option<String>,
    #[serde(default)]
    pub templates: Option<String>,
    #[serde(default)]
    pub lowering: Option<String>,
    #[serde(default)]
    pub metrics: Option<String>,
    #[serde(default)]
    pub validations: Option<String>,
    /// `statements.toml`: `[[subtotals]]` now, `[[statements]]` next.
    #[serde(default)]
    pub statements: Option<String>,
    /// `ontology/types.toml`: what the model may be about.
    #[serde(default)]
    pub ontology: Option<String>,
}

// ---------------------------------------------------------------------------
// Ontology
//
// A pack's lowering rules say how a contract becomes cash. The ontology says
// what the model is ABOUT, which nothing said before: an entity was a two-part
// name, and the namespace half was doing informal typing badly.
//
// FOUR FAMILIES, fixed here and filled in per pack: `asset` produces or
// consumes cash, `party` contracts and owns, `contract` is an agreement
// between parties attached to an asset, `reference` is a market observable.
// ---------------------------------------------------------------------------

/// The families an entity type may belong to. Closed, because the language —
/// not the pack — decides what kinds of thing a model contains.
pub const ENTITY_FAMILIES: &[&str] = &["asset", "party"];

/// The classes an asset may take. The split every asset taxonomy starts from,
/// and the one that decides how an asset is underwritten: a turbine is real, a
/// tax-equity interest in it is financial, a royalty is intangible.
pub const ASSET_CLASSES: &[&str] = &["real", "financial", "intangible"];

#[derive(Debug, Clone, Default, PartialEq)]
pub struct PackOntology {
    pub entities: Vec<OntologyEntity>,
    pub contracts: Vec<OntologyContract>,
    pub lifecycles: Vec<OntologyLifecycle>,
    pub references: Vec<OntologyReference>,
    pub relations: Vec<OntologyRelation>,
}

impl PackOntology {
    /// The vocabulary EVERY model has, pack or no pack.
    ///
    /// An ontology is a LANGUAGE capability that packs supply defaults for, not
    /// one they own — the same argument the category vocabulary already makes:
    /// rejecting it with no pack active is circular, because nothing reads it
    /// only so long as nothing may declare it. A model with no pack can still
    /// say that a thing is an asset, that the asset is real rather than
    /// financial, who the parties are, and what belongs to what. What it cannot
    /// do without a pack is name a CONTRACT type, because a contract type binds
    /// to a lowering rule and lowering rules are what packs are.
    ///
    /// A pack's own types are added to these; a pack cannot remove them.
    pub fn language_base() -> Self {
        fn asset(type_id: &str, class: &str, description: &str) -> OntologyEntity {
            OntologyEntity {
                type_id: type_id.to_string(),
                family: "asset".to_string(),
                class: Some(class.to_string()),
                lifecycle: None,
                description: Some(description.to_string()),
                fields: Vec::new(),
            }
        }
        fn relation(
            id: &str,
            from: &str,
            to: &str,
            card: &str,
            inverse: &str,
            d: &str,
        ) -> OntologyRelation {
            OntologyRelation {
                relation_id: id.to_string(),
                from_family: from.to_string(),
                to_family: to.to_string(),
                cardinality: card.to_string(),
                inverse: inverse.to_string(),
                description: Some(d.to_string()),
            }
        }
        Self {
            entities: vec![
                asset(
                    "Asset.Real",
                    "real",
                    "A physical thing — land, a building, plant, equipment, a reserve.",
                ),
                asset(
                    "Asset.Financial",
                    "financial",
                    "A claim on cash — a loan, a pool, a security, an equity interest, a going concern.",
                ),
                asset(
                    "Asset.Intangible",
                    "intangible",
                    "A right without a physical form — a royalty, a license, a patent.",
                ),
                OntologyEntity {
                    type_id: "Party".to_string(),
                    family: "party".to_string(),
                    class: None,
                    lifecycle: None,
                    description: Some(
                        "Someone who contracts, owns, lends or invests. A pack names roles more precisely; this is the generic one."
                            .to_string(),
                    ),
                    fields: vec![OntologyField {
                        name: "name".to_string(),
                        field_type: "string".to_string(),
                        required: false,
                        unit: None,
                        description: None,
                    }],
                },
            ],
            contracts: Vec::new(),
            lifecycles: Vec::new(),
            references: Vec::new(),
            relations: vec![
                relation(
                    "part_of",
                    "asset",
                    "asset",
                    "many_to_one",
                    "contains",
                    "Optional hierarchy. Never required: the modeller chooses the grain, and an asset stands alone unless grouped.",
                ),
                relation(
                    "owns",
                    "party",
                    "asset",
                    "many_to_many",
                    "owned_by",
                    "Who holds the asset.",
                ),
            ],
        }
    }

    /// A pack's vocabulary on top of the language's. Pack types win on a
    /// collision, so a pack may refine `Asset.Real` into `CRE.Asset.RealProperty`
    /// without the base disappearing.
    pub fn merged_with_base(&self) -> Self {
        let mut merged = Self::language_base();
        let pack_types: BTreeSet<&str> = self.entities.iter().map(|e| e.type_id.as_str()).collect();
        merged
            .entities
            .retain(|e| !pack_types.contains(e.type_id.as_str()));
        merged.entities.extend(self.entities.iter().cloned());

        let pack_relations: BTreeSet<&str> = self
            .relations
            .iter()
            .map(|r| r.relation_id.as_str())
            .collect();
        merged
            .relations
            .retain(|r| !pack_relations.contains(r.relation_id.as_str()));
        merged.relations.extend(self.relations.iter().cloned());

        merged.contracts = self.contracts.clone();
        merged.lifecycles = self.lifecycles.clone();
        merged.references = self.references.clone();
        merged
    }

    pub fn is_empty(&self) -> bool {
        self.entities.is_empty() && self.contracts.is_empty()
    }

    pub fn entity(&self, type_id: &str) -> Option<&OntologyEntity> {
        self.entities.iter().find(|e| e.type_id == type_id)
    }

    pub fn lifecycle(&self, lifecycle_id: &str) -> Option<&OntologyLifecycle> {
        self.lifecycles
            .iter()
            .find(|l| l.lifecycle_id == lifecycle_id)
    }

    /// The contract type bound to a lowering rule's `contract_name`.
    pub fn contract_for_rule(&self, contract_name: &str) -> Option<&OntologyContract> {
        self.contracts
            .iter()
            .find(|c| c.contract_name.as_deref() == Some(contract_name))
    }

    /// Contract types that are elections rather than lowered rules — options.
    pub fn elections(&self) -> impl Iterator<Item = &OntologyContract> {
        self.contracts.iter().filter(|c| c.contract_name.is_none())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OntologyEntity {
    pub type_id: String,
    pub family: String,
    /// Required for `asset`, absent for `party` — only an asset has a class.
    #[serde(default)]
    pub class: Option<String>,
    /// The lifecycle this type moves through. Absent means the type has no
    /// states; present means it is ALWAYS in exactly one of them.
    #[serde(default)]
    pub lifecycle: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub fields: Vec<OntologyField>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OntologyField {
    pub name: String,
    pub field_type: String,
    #[serde(default)]
    pub required: bool,
    /// The dimension the number carries. A quantity without one is how a PTC
    /// gets rounded to a hundredth of a cent instead of a tenth.
    #[serde(default)]
    pub unit: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OntologyContract {
    pub type_id: String,
    /// The lowering rule that turns this contract into cash. Binding the two
    /// is what keeps the vocabulary and the arithmetic from drifting: a rule
    /// with no type has no counterparties.
    ///
    /// ABSENT MEANS AN ELECTION — an option. An option is a contract whose cash
    /// is a payoff the holder elects to take, resolved by the engine rather
    /// than lowered by a pack rule, so there is no rule to name.
    #[serde(default)]
    pub contract_name: Option<String>,
    #[serde(default)]
    pub subject_family: Option<String>,
    /// Role names, not entity references — a party fills a role per contract,
    /// so the same party can be lessor in one and lender in another.
    #[serde(default)]
    pub parties: Vec<String>,
    #[serde(default)]
    pub description: Option<String>,
}

/// A declared state space. The point of declaring it is totality: an entity is
/// ALWAYS in exactly one state, starting at `initial`. There is no null state
/// and no undeclared state, which is what makes a misspelled status impossible
/// rather than merely unlikely.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OntologyLifecycle {
    pub lifecycle_id: String,
    pub initial: String,
    pub states: Vec<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub transitions: Vec<OntologyTransition>,
    /// `[[lifecycles.entry_actions]]` — what is true of a STATE however it was
    /// reached (`docs/34` D3). The primary domain spelling: a pack declares it
    /// once and every model using the type inherits it, including for an edge
    /// somebody adds later.
    #[serde(default)]
    pub entry_actions: Vec<OntologyStateEntry>,
}

/// One state's arrival actions, as a pack declares them.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OntologyStateEntry {
    pub state: String,
    #[serde(default)]
    pub description: Option<String>,
    pub actions: Vec<OntologyAction>,
}

/// One arrival action. `set <field> = <expr>`, and nothing else (`docs/34`
/// D4). The field is ENTITY-RELATIVE: it resolves against whichever entity
/// bound this machine, because one lifecycle is bound by many.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OntologyAction {
    pub set: String,
    pub value: String,
    #[serde(default)]
    pub description: Option<String>,
}

impl OntologyLifecycle {
    pub fn has_state(&self, state: &str) -> bool {
        self.states.iter().any(|s| s == state)
    }

    /// Whether the declared relation permits this move. An empty transition
    /// list means the pack has not constrained the machine, so every move
    /// between declared states is allowed.
    pub fn permits(&self, from: &str, to: &str) -> bool {
        self.transitions.is_empty()
            || self
                .transitions
                .iter()
                .any(|t| t.from == from && t.to == to)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OntologyTransition {
    pub from: String,
    pub to: String,
    #[serde(default)]
    pub description: Option<String>,
    /// `guard = "<expr>"` — evaluated each period an entity of this type is
    /// in `from`, exactly as a model-declared edge's `when` is (`docs/28`
    /// §6.1): the core has the full functionality, and a pack tailors it. A
    /// guard-less edge stays a permission an event's write may take, which
    /// every shipped pack edge is.
    #[serde(default)]
    pub guard: Option<String>,
    /// `[[lifecycles.transitions.actions]]` — what is true of the PATH taken
    /// rather than of the state arrived in. A renewal and a re-let both land
    /// in `leased`, and the rent is struck differently because of how you
    /// arrived; an entry action cannot say that.
    #[serde(default)]
    pub actions: Vec<OntologyAction>,
}

/// A market observable. Declared in the model today; the shape admits an
/// external source later without the model changing.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OntologyReference {
    pub reference_id: String,
    pub kind: String,
    #[serde(default)]
    pub unit: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
}

/// A typed link, with an inverse so either end reads naturally.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OntologyRelation {
    pub relation_id: String,
    pub from_family: String,
    pub to_family: String,
    pub cardinality: String,
    pub inverse: String,
    #[serde(default)]
    pub description: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct OntologyFile {
    #[serde(default)]
    pack: Option<OntologyFileHeader>,
    #[serde(default)]
    entities: Vec<OntologyEntity>,
    #[serde(default)]
    contracts: Vec<OntologyContract>,
    #[serde(default)]
    lifecycles: Vec<OntologyLifecycle>,
    #[serde(default)]
    references: Vec<OntologyReference>,
    #[serde(default)]
    relations: Vec<OntologyRelation>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct OntologyFileHeader {
    /// Checked against the pack's own name. Catches the copy-paste — an
    /// ontology lifted from one pack into another and left self-identifying as
    /// the first.
    #[serde(default)]
    ontology_id: Option<String>,
    #[serde(default)]
    #[allow(dead_code)]
    version: Option<String>,
}

/// A single declarative domain check supplied by a pack.
///
/// Packs own *what* to check (which term, which bound, which stable code);
/// the compiler owns spans, timeline access, and diagnostic emission. The
/// check kinds are a closed set with no expressions, recursion, or
/// interpolation, so a pack can never crash, hang, or allocate unboundedly
/// in the compiler.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PackValidation {
    /// Exactly one of `contract` / `contracts` must be set.
    #[serde(default)]
    pub contract: Option<String>,
    #[serde(default)]
    pub contracts: Vec<String>,
    pub code: String,
    pub message: String,
    #[serde(default)]
    pub severity: ValidationSeverity,
    pub check: ValidationCheck,
    /// Term under test (`term_present`, `term_number`, `term_enum`).
    #[serde(default)]
    pub term: Option<String>,
    /// Terms for `any_term_present`.
    #[serde(default)]
    pub terms: Vec<String>,
    #[serde(default)]
    pub number: NumberKind,
    #[serde(default)]
    pub when: WhenPresence,
    #[serde(default)]
    pub on_invalid: OnInvalid,
    #[serde(default)]
    pub min: Option<f64>,
    #[serde(default)]
    pub max: Option<f64>,
    #[serde(default)]
    pub exclusive_min: Option<f64>,
    #[serde(default)]
    pub exclusive_max: Option<f64>,
    /// Allowed values for `term_enum`.
    #[serde(default)]
    pub values: Vec<ValidationValue>,
    /// `term_compare` operands.
    #[serde(default)]
    pub left: Option<String>,
    #[serde(default)]
    pub right: Option<String>,
    #[serde(default)]
    pub op: Option<CompareOp>,
}

/// Does a pack rule declared for `declared` apply to a contract the model calls
/// `contract_name`?
///
/// A model must suffix a contract whenever the deal has more than one of
/// something — two tenants are `cre.lease_unit.tenant_a` and `.tenant_b` — so a
/// rule naming the type has to reach its instances. The next character must be
/// `.`, which is what stops `cre.debt` from claiming `cre.debt_service`.
///
/// THIS IS NOT A CHOICE, and it used to be one. `PackValidation` carried a
/// `match` field defaulting to exact matching, so a validation that did not
/// declare `match = "instance"` was silently skipped on the form models
/// actually use — it never fired and nothing said so. Two thirds of them were
/// dead that way. Lowering never had the option: `rule_matches_contract` in
/// `cfdl-compile` has always matched instances unconditionally, for the case
/// that decides what cash a contract produces. Validations were the outlier
/// and no reason was ever recorded, so the field is gone and both callers
/// share this.
pub fn matches_contract_name(declared: &str, contract_name: &str) -> bool {
    contract_name == declared
        || contract_name
            .strip_prefix(declared)
            .is_some_and(|rest| rest.starts_with('.'))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ValidationSeverity {
    #[default]
    Error,
    Warning,
    Info,
}

impl ValidationSeverity {
    pub fn as_str(self) -> &'static str {
        match self {
            ValidationSeverity::Error => "error",
            ValidationSeverity::Warning => "warning",
            ValidationSeverity::Info => "info",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ValidationCheck {
    /// The term must be present.
    TermPresent,
    /// At least one of `terms` must be present.
    AnyTermPresent,
    /// At most one of `terms` may be present.
    ///
    /// For a pair that says the same thing in different units — a per-period
    /// `amount` and an annual `amount_year`. Templates have no conditional, so
    /// a rule sums both with zero defaults; stating both would silently add
    /// them, which is almost never what anyone means.
    TermsMutuallyExclusive,
    /// The term must parse as a number and satisfy any declared bounds.
    TermNumber,
    /// The contract term range must be valid and inside the model timeline.
    TermRangeWithinTimeline,
    /// The term must equal one of `values`.
    TermEnum,
    /// Two numeric terms must satisfy `left <op> right`.
    TermCompare,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NumberKind {
    #[default]
    Decimal,
    Integer,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WhenPresence {
    /// Run even when the term is absent (absence is itself a failure).
    #[default]
    Always,
    /// Only run when the term is present.
    Present,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OnInvalid {
    /// An unparseable value fails this check.
    #[default]
    Report,
    /// An unparseable value is another check's responsibility.
    Skip,
}

/// A literal an enum check compares against. Accepts TOML strings, integers,
/// and floats without exposing the TOML value type to consumers.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(untagged)]
pub enum ValidationValue {
    Integer(i64),
    Float(f64),
    Text(String),
}

impl ValidationValue {
    /// Whether a raw term value equals this literal.
    pub fn matches(&self, raw: &str) -> bool {
        match self {
            ValidationValue::Text(text) => text == raw,
            ValidationValue::Integer(number) => raw
                .parse::<i64>()
                .map(|parsed| parsed == *number)
                .unwrap_or(false),
            ValidationValue::Float(number) => raw
                .parse::<f64>()
                .map(|parsed| (parsed - *number).abs() < f64::EPSILON)
                .unwrap_or(false),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CompareOp {
    Le,
    Lt,
    Ge,
    Gt,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
struct ValidationsFile {
    #[serde(default)]
    schema_version: Option<u32>,
    #[serde(default)]
    code_prefix: Option<String>,
    #[serde(default)]
    validations: Vec<PackValidation>,
}

impl PackValidation {
    /// Contract names this validation applies to.
    pub fn contract_names(&self) -> Vec<&str> {
        match &self.contract {
            Some(name) => vec![name.as_str()],
            None => self.contracts.iter().map(String::as_str).collect(),
        }
    }

    /// Whether this validation applies to a contract declared in a model.
    pub fn applies_to(&self, contract_name: &str) -> bool {
        self.contract_names()
            .into_iter()
            .any(|declared| matches_contract_name(declared, contract_name))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct LoweringFile {
    #[serde(default)]
    pub rules: Vec<LoweringRule>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct LoweringRule {
    pub id: String,
    pub contract_name: String,
    pub stream_name: String,
    pub owner_entity: String,
    pub direction: String,
    /// Currency the stream is denominated in.
    ///
    /// Omit it unless the instrument is genuinely fixed to one currency: an
    /// empty value inherits the model's declared currency, which is what makes
    /// a pack usable outside the United States. A PPA in Rajasthan is not a
    /// USD contract.
    #[serde(default)]
    pub currency: String,
    /// What this stream IS, economically — `revenue`, `opex`, `debt_service`.
    ///
    /// Aggregation reads this rather than pattern-matching the stream's name.
    /// A name is an address, not a meaning: deciding that `cre.vacancy.loss` is
    /// a deduction by looking at its spelling means every consumer re-derives
    /// the same judgement independently, and they drift — which is exactly how
    /// two selector dialects came to disagree. Classified once, at the point of
    /// emission, a stream is necessarily both reported as a line and counted in
    /// its subtotal.
    ///
    /// Must be one of the categories the pack manifest declares; an unlisted
    /// value is `E5022`. Empty means unclassified, which is legal but leaves
    /// the stream out of every category fold.
    #[serde(default)]
    pub category: String,
    /// May contain `{{contract.<key>}}` placeholders (see expand_rule_template).
    pub amount_expr: String,
    pub schedule_kind: String,
    /// Recurrence interval for a recurring rule: `day`, `week`, `month`,
    /// `quarter` or `year`.
    ///
    /// Omit it to pay at the model's calendar cadence, which is what most
    /// rules want. Set it when the instrument genuinely pays on its own
    /// rhythm — a quarterly coupon or an annual true-up on a monthly model.
    /// The interval must be no finer than the calendar, since several
    /// payments in one period would collapse into one.
    #[serde(default)]
    pub schedule_every: String,
    /// Model calendars this rule lowers correctly on; empty means all.
    ///
    /// Overrides the pack manifest's `cadences`, so a pack can carry a mix of
    /// neutral rules and month-locked ones while it is being migrated, rather
    /// than being gated wholesale. As each rule is made cadence-neutral its
    /// entry widens and then disappears.
    #[serde(default)]
    pub cadences: Vec<String>,
    /// How long after a flow is earned its cash moves, overriding the
    /// contract's `payment net <n>` for this rule. Omit both to inherit.
    ///
    /// Templated, so a rule can defer to a contract term:
    /// `schedule_net_months = "{{contract.recovery_lag_months}}"`.
    #[serde(default)]
    pub schedule_net_days: String,
    #[serde(default)]
    pub schedule_net_months: String,
    /// Annuity due: the stream pays at the start of each interval.
    ///
    /// WHERE IN ITS PERIOD THE STREAM'S CASH SITS — `"start"`, `"mid"` or
    /// `"end"`. One axis with three positions rather than three booleans, so
    /// a rule cannot state two.
    ///
    /// Omitted means the FORM's default, which differs: an `on_date` rule
    /// opens its period (right for an acquisition), a recurring rule closes
    /// it (an ordinary annuity — the interval elapses, then payment falls).
    ///
    /// `"start"` is what an expense-like stream wants — opex, rent paid,
    /// fees, capex fall due in the period they belong to. `"mid"` is the
    /// project-finance and banker-DCF convention: a year's cash arrives
    /// evenly, so treating it as a single point at the year's midpoint beats
    /// treating it as arriving on 31 December. It is a convention, not a
    /// date — half a period on every calendar.
    ///
    /// `"end"` matters most on a one-shot. `on_date` otherwise discounts from
    /// the period's open, which is wrong for a disposal: a reversion is taken
    /// at the end of the holding period, so a year-5 sale must be discounted
    /// five periods, not four. On a monthly model that is one month and easy
    /// to miss; on an annual model it is a full year.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub schedule_placement: Option<String>,
    /// May contain `{{contract.term_start}}` / `{{contract.<key>}}` placeholders.
    pub schedule_from: String,
    /// May contain `{{contract.term_end}}` / `{{contract.<key>}}` placeholders.
    pub schedule_to: String,
    /// Name of a `state` this rule declares alongside its stream. Empty for
    /// the great majority of rules, which need no recurrence.
    ///
    /// A state is how a rule compounds a rate that MOVES. `pow(1 + g, t)`
    /// applies one period's rate as though it had held from the start — exact
    /// while the rate is flat, wrong the moment it varies, which is precisely
    /// what a decaying growth path or an escalating expense does.
    ///
    /// Must expand to a single identifier, since `state.<name>` resolves one
    /// segment: use `{{contract.suffix_ident}}`, not `{{contract.dot_suffix}}`,
    /// to keep each contract instance's state distinct.
    #[serde(default)]
    pub field_name: String,
    /// The state's value at period 0. Templated.
    #[serde(default)]
    pub field_init: String,
    /// The state's value at every later period, with `prev` bound to its own
    /// previous value. Templated.
    #[serde(default)]
    pub field_next: String,
    /// How often the state STEPS: `day`, `week`, `month`, `quarter`, `year`.
    /// Templated, so a rule can defer to a contract term
    /// (`field_every = "{{contract.payment_frequency}}"`).
    ///
    /// Empty means every model period. Set it whenever the recurrence belongs
    /// to the INSTRUMENT's rhythm rather than the book's: a pool carried on a
    /// daily calendar but paying monthly must compound twelve times a year,
    /// not three hundred and sixty-five. Between ticks the state holds.
    ///
    /// An interval finer than the model calendar is
    /// `E2108_SCHEDULE_FINER_THAN_CALENDAR`, the same rule a stream's schedule
    /// obeys — a pack cannot express what a model may not.
    #[serde(default)]
    pub field_every: String,
    /// Window over which the state steps. Templated; both default to the
    /// model timeline. Outside the window the state holds rather than zeroing.
    #[serde(default)]
    pub field_from: String,
    #[serde(default)]
    pub field_to: String,
    /// Default values for template placeholders when the contract does not
    /// declare the term. Keys are the bare placeholder names (no `contract.`
    /// prefix), e.g. `"lease_up.months" = "18"`.
    #[serde(default)]
    pub defaults: BTreeMap<String, String>,
    /// The unit each term is expressed in — `"credit_per_mwh" = "USD/MWh"`.
    ///
    /// A quantity without a stated dimension is a number that means whatever
    /// the reader assumes. The energy pack's own comments spend a paragraph
    /// warning that 0.1 c/kWh is $1.00/MWh and that getting it wrong rounds to
    /// a hundredth of a cent — indistinguishable from not rounding at all. This
    /// is where that warning becomes checkable.
    ///
    /// A model may ANNOTATE a term with the unit it believes it is writing
    /// (`credit_per_mwh = 27.50 USD/MWh`). The annotation is an assertion, and
    /// the rule is the truth: they must agree.
    #[serde(default)]
    pub units: BTreeMap<String, String>,
}

/// Declarative domain-metric definition (metrics.toml). Metrics are
/// evaluated in file order, so ratio metrics may reference earlier ones.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct MetricSpec {
    /// Output key, e.g. "domain.cre.noi".
    pub id: String,
    /// "money" | "number"
    pub kind: String,
    /// "sum" (numerator + denominator stream totals, signed),
    /// "negated_sum" (-sum of numerator stream totals),
    /// "ratio" (numerator_metric / denominator_metric).
    pub op: String,
    #[serde(default)]
    pub numerator_streams: Vec<String>,
    #[serde(default)]
    pub denominator_streams: Vec<String>,
    #[serde(default)]
    pub numerator_metric: Option<String>,
    #[serde(default)]
    pub denominator_metric: Option<String>,
    /// Human-readable lineage formula, emitted verbatim.
    pub formula: String,
    /// Omit the metric unless its value is strictly positive.
    #[serde(default)]
    pub require_positive: bool,
    /// `subtotal_total` only: the per-period subtotal series to reduce.
    ///
    /// This is how a lifetime scalar stops being a second, independent
    /// definition of the same quantity. Before it, `domain.cre.noi` existed
    /// twice — nine hand-listed stream selectors here, and a category fold in
    /// statements.toml — and two independent statements of one quantity drift.
    #[serde(default)]
    pub subtotal: Option<String>,
}

/// A per-period subtotal: a named fold over the ledger.
///
/// Where a `MetricSpec` reduces to one lifetime scalar, this produces a value
/// per period — the middle rows of a statement, which had no representation at
/// all. `domain.cre.noi` was a single number for a ten-year hold.
///
/// Folds are declared over CATEGORIES rather than stream names wherever
/// possible, which is the point of categories being dotted paths: net operating
/// income is everything under `operating.*`, and effective gross income is
/// `operating.revenue.*` plus `operating.deduction.*`. No stream is named, so
/// adding a contract to a pack does not mean remembering to add its stream to a
/// subtotal — the classification already said where it belongs.
///
/// Verified against a published source: those two definitions reproduce the HUD
/// Sample workbook's own Effective Gross Income and Net Operating Income rows
/// exactly, and `financing.*` reproduces the debt service its published DSCR
/// divides by.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SubtotalSpec {
    /// Output series key, e.g. `domain.cre.noi`. Must start with `domain.`.
    pub id: String,
    /// `money` (a sum of cash) or `number` (a ratio).
    pub kind: String,
    /// `sum`, `negated_sum`, or `ratio`.
    pub op: String,
    /// Category path prefixes to fold, e.g. `operating.revenue.*`. The
    /// preferred form.
    #[serde(default)]
    pub categories: Vec<String>,
    /// Stream-name selectors, for the cases a category cannot express — a
    /// single named stream rather than a class of them.
    #[serde(default)]
    pub streams: Vec<String>,
    /// Ids of subtotals declared EARLIER in this file. Order is the dependency
    /// order, so a forward reference is a compile error and no cycle is
    /// reachable. Same discipline `metrics.toml` ratios already use.
    #[serde(default)]
    pub subtotals: Vec<String>,
    /// `ratio` only: the subtotal ids to divide.
    #[serde(default)]
    pub numerator: Option<String>,
    #[serde(default)]
    pub denominator: Option<String>,
    /// Human-readable lineage, emitted verbatim so a published row can be
    /// audited without reading the pack.
    #[serde(default)]
    pub formula: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
struct MetricsFile {
    #[serde(default)]
    metrics: Vec<MetricSpec>,
}

/// A declared statement: an ordered tree of rows over the subtotals and
/// categories, carrying order, labels, depth and display signs.
///
/// Rows COMPUTE nothing. Everything numeric was already folded by the engine;
/// this says how to present it. Keeping the two apart is the fix for backlog
/// 1.3: a deduction can be shown as a positive number in a "less:" row while
/// still being counted negatively, which the old sign-flipping could not do.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StatementSpec {
    pub id: String,
    #[serde(default)]
    pub label: String,
    /// Shown when a consumer asks for "the" statement and the pack has several.
    #[serde(default)]
    pub default: bool,
    /// The grain this statement reports at. `None` is the model grid.
    ///
    /// Grain belongs to the OUTPUT, not the run: one package can carry a
    /// monthly cash flow statement and an annual summary of the same cash,
    /// because each declaration names its own.
    #[serde(default)]
    pub grain: Option<String>,
    #[serde(default)]
    pub rows: Vec<StatementRow>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StatementRow {
    /// `line` | `subtotal` | `ratio` | `spacer`. `residual` is emitted by the
    /// evaluator for streams no row claimed and may not be authored.
    pub kind: String,
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub label: String,
    /// Indent level for presentation only.
    #[serde(default)]
    pub depth: u32,
    /// `line` rows: the categories folded into this row.
    #[serde(default)]
    pub categories: Vec<String>,
    /// `line` rows: stream selectors, for what a category cannot express.
    #[serde(default)]
    pub streams: Vec<String>,
    /// `subtotal` / `ratio` rows: the published series to read.
    #[serde(default)]
    pub subtotal: Option<String>,
    /// How to RENDER the sign: `natural` (default), `positive`, `negative`.
    /// Never changes what is summed — the row carries the signed value too.
    #[serde(default)]
    pub display: Option<String>,
}

/// `statements.toml`. Holds `[[subtotals]]`, which the compiler lowers into the
/// IR for the engine to evaluate, and will hold `[[statements]]` — the ordering
/// and labelling read after a run. One file because a subtotal exists to be a
/// statement row, and splitting them would make the cross-reference between
/// them unvalidatable at load time.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
struct StatementsFile {
    #[serde(default)]
    subtotals: Vec<SubtotalSpec>,
    #[serde(default)]
    statements: Vec<StatementSpec>,
}

/// Standard packs compiled into the library for hosts without filesystem
/// access (WASM playground, API server). Enabled by the `embedded-packs`
/// feature. NOTE: `include_str!` paths assume the repo layout (packs/ at the
/// workspace root); for crates.io publishing the pack data moves into the
/// crate — tracked for the 1.0 packaging pass.
#[cfg(feature = "embedded-packs")]
mod embedded {
    pub type EmbeddedFile = (&'static str, &'static str);

    pub const CRE: &[EmbeddedFile] = &[
        ("pack.toml", include_str!("../../../packs/cre/pack.toml")),
        (
            "aliases.toml",
            include_str!("../../../packs/cre/aliases.toml"),
        ),
        (
            "templates.toml",
            include_str!("../../../packs/cre/templates.toml"),
        ),
        (
            "lowering/rules.toml",
            include_str!("../../../packs/cre/lowering/rules.toml"),
        ),
        (
            "metrics.toml",
            include_str!("../../../packs/cre/metrics.toml"),
        ),
        (
            "statements.toml",
            include_str!("../../../packs/cre/statements.toml"),
        ),
        (
            "validations.toml",
            include_str!("../../../packs/cre/validations.toml"),
        ),
        (
            "ontology/types.toml",
            include_str!("../../../packs/cre/ontology/types.toml"),
        ),
    ];

    pub const OPCO: &[EmbeddedFile] = &[
        ("pack.toml", include_str!("../../../packs/opco/pack.toml")),
        (
            "aliases.toml",
            include_str!("../../../packs/opco/aliases.toml"),
        ),
        (
            "lowering/rules.toml",
            include_str!("../../../packs/opco/lowering/rules.toml"),
        ),
        (
            "metrics.toml",
            include_str!("../../../packs/opco/metrics.toml"),
        ),
        (
            "statements.toml",
            include_str!("../../../packs/opco/statements.toml"),
        ),
        (
            "validations.toml",
            include_str!("../../../packs/opco/validations.toml"),
        ),
        (
            "ontology/types.toml",
            include_str!("../../../packs/opco/ontology/types.toml"),
        ),
    ];

    pub const CREDIT: &[EmbeddedFile] = &[
        ("pack.toml", include_str!("../../../packs/credit/pack.toml")),
        (
            "aliases.toml",
            include_str!("../../../packs/credit/aliases.toml"),
        ),
        (
            "lowering/rules.toml",
            include_str!("../../../packs/credit/lowering/rules.toml"),
        ),
        (
            "metrics.toml",
            include_str!("../../../packs/credit/metrics.toml"),
        ),
        (
            "statements.toml",
            include_str!("../../../packs/credit/statements.toml"),
        ),
        (
            "validations.toml",
            include_str!("../../../packs/credit/validations.toml"),
        ),
        (
            "ontology/types.toml",
            include_str!("../../../packs/credit/ontology/types.toml"),
        ),
    ];

    pub const ENERGY: &[EmbeddedFile] = &[
        ("pack.toml", include_str!("../../../packs/energy/pack.toml")),
        (
            "aliases.toml",
            include_str!("../../../packs/energy/aliases.toml"),
        ),
        (
            "lowering/rules.toml",
            include_str!("../../../packs/energy/lowering/rules.toml"),
        ),
        (
            "metrics.toml",
            include_str!("../../../packs/energy/metrics.toml"),
        ),
        (
            "statements.toml",
            include_str!("../../../packs/energy/statements.toml"),
        ),
        (
            "validations.toml",
            include_str!("../../../packs/energy/validations.toml"),
        ),
        (
            "ontology/types.toml",
            include_str!("../../../packs/energy/ontology/types.toml"),
        ),
    ];

    pub const ALL: &[&[EmbeddedFile]] = &[CRE, CREDIT, ENERGY, OPCO];
}

impl PackRegistry {
    /// Load the standard packs compiled into the library (no filesystem).
    #[cfg(feature = "embedded-packs")]
    pub fn load_embedded() -> Result<Self, PackLoadError> {
        let mut packs = BTreeMap::new();
        for files in embedded::ALL {
            let lookup = |relative: Option<&str>| -> Option<&'static str> {
                let relative = relative?;
                files
                    .iter()
                    .find(|(name, _)| *name == relative)
                    .map(|(_, content)| *content)
            };
            let manifest_raw = lookup(Some("pack.toml")).ok_or_else(|| PackLoadError {
                message: "Embedded pack is missing pack.toml".to_string(),
            })?;
            let manifest: PackManifest =
                toml::from_str(manifest_raw).map_err(|err| PackLoadError {
                    message: format!("Failed to parse embedded pack manifest: {err}"),
                })?;
            let source = format!("embedded:{}", manifest.name);
            let aliases = match lookup(manifest.entrypoints.aliases.as_deref()) {
                Some(raw) => parse_aliases(raw, &source)?,
                None => BTreeMap::new(),
            };
            let templates = match lookup(manifest.entrypoints.templates.as_deref()) {
                Some(raw) => parse_templates(raw, &source)?,
                None => Vec::new(),
            };
            let lowering_rules = match lookup(manifest.entrypoints.lowering.as_deref()) {
                Some(raw) => parse_lowering_rules(raw, &source)?,
                None => Vec::new(),
            };
            validate_category_vocabulary(&manifest.categories, &source)?;
            validate_rule_categories(&lowering_rules, &manifest.categories, &source)?;
            let metric_specs = match lookup(manifest.entrypoints.metrics.as_deref()) {
                Some(raw) => parse_metric_specs(raw, &source)?,
                None => Vec::new(),
            };
            let validations = match lookup(manifest.entrypoints.validations.as_deref()) {
                Some(raw) => parse_validations(raw, &source)?,
                None => Vec::new(),
            };
            let subtotal_specs = match lookup(manifest.entrypoints.statements.as_deref()) {
                Some(raw) => parse_subtotal_specs(raw, &source)?,
                None => Vec::new(),
            };
            let statement_specs = match lookup(manifest.entrypoints.statements.as_deref()) {
                Some(raw) => parse_statement_specs(
                    raw,
                    &source,
                    &manifest.categories,
                    &subtotal_specs,
                    &lowering_rules,
                )?,
                None => Vec::new(),
            };
            let ontology = match lookup(manifest.entrypoints.ontology.as_deref()) {
                Some(raw) => parse_ontology(raw, &source, &manifest.name)?,
                None => PackOntology::default(),
            };
            validate_ontology_against_rules(&ontology, &lowering_rules, &source)?;
            packs.insert(
                manifest.name.clone(),
                LoadedPack {
                    manifest,
                    aliases,
                    templates,
                    lowering_rules,
                    metric_specs,
                    subtotal_specs,
                    statement_specs,
                    validations,
                    ontology,
                },
            );
        }
        Ok(Self { packs })
    }
}

/// Expand `{{contract.<key>}}` placeholders in a lowering-rule template.
///
/// `resolve` maps a bare key (e.g. `base_rent`, `term_start`,
/// `lease_up.months`) to its value; unresolved keys are collected and
/// returned as `Err` so the caller can emit one diagnostic per missing term.
/// Substitution is textual: numeric contract terms yield valid expression
/// fragments, string terms must be quoted inside the template.
/// The placeholder keys a template refers to, in order of first appearance.
///
/// The same scan `expand_rule_template` performs, without resolving anything.
/// It exists so the compiler can record WHICH contract terms a rule actually
/// consumed: a contract may lower to several streams, each reading a different
/// subset of its terms, so "the contract's terms" is not the answer to "what
/// struck this line". Keys are returned with any `contract.` prefix stripped,
/// matching what `resolve` is handed.
pub fn template_placeholders(template: &str) -> Vec<String> {
    let mut keys: Vec<String> = Vec::new();
    let mut rest = template;
    while let Some(start) = rest.find("{{") {
        let after = &rest[start + 2..];
        let Some(end) = after.find("}}") else { break };
        let raw_key = after[..end].trim();
        let key = raw_key.strip_prefix("contract.").unwrap_or(raw_key);
        if !keys.iter().any(|k| k == key) {
            keys.push(key.to_string());
        }
        rest = &after[end + 2..];
    }
    keys
}

pub fn expand_rule_template(
    template: &str,
    resolve: &dyn Fn(&str) -> Option<String>,
) -> Result<String, Vec<String>> {
    let mut out = String::with_capacity(template.len());
    let mut missing: Vec<String> = Vec::new();
    let mut rest = template;
    while let Some(start) = rest.find("{{") {
        out.push_str(&rest[..start]);
        let after = &rest[start + 2..];
        let Some(end) = after.find("}}") else {
            // Unterminated placeholder: treat the remainder as literal text.
            out.push_str(&rest[start..]);
            rest = "";
            break;
        };
        let raw_key = after[..end].trim();
        let key = raw_key.strip_prefix("contract.").unwrap_or(raw_key);
        match resolve(key) {
            Some(value) => out.push_str(&value),
            None => {
                if !missing.iter().any(|k| k == key) {
                    missing.push(key.to_string());
                }
            }
        }
        rest = &after[end + 2..];
    }
    out.push_str(rest);
    if missing.is_empty() {
        Ok(out)
    } else {
        Err(missing)
    }
}

/// Outcome of resolving a `use pack` request.
#[derive(Debug, Clone)]
pub enum PackLookup {
    Found(ActivePack),
    /// No pack of that name in the registry.
    Absent,
    /// The pack exists, at a different version.
    VersionMismatch {
        available: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActivePack {
    pub name: String,
    pub version: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TemplateExpansionRequest {
    pub template: String,
    pub params: BTreeMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TemplateExpansion {
    pub generated_nodes: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct TemplateFile {
    #[serde(default)]
    pub templates: Vec<PackTemplate>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct PackTemplate {
    pub id: String,
    #[serde(default)]
    pub label: Option<String>,
    #[serde(default)]
    pub kind: Option<String>,
    pub body: String,
    #[serde(default)]
    pub defaults: BTreeMap<String, String>,
}

impl PackRegistry {
    pub fn load_from_dir(root: &Path) -> Result<Self, PackLoadError> {
        if !root.exists() {
            return Ok(Self {
                packs: BTreeMap::new(),
            });
        }
        if !root.is_dir() {
            return Err(PackLoadError {
                message: format!("Pack root '{}' is not a directory.", root.display()),
            });
        }

        let mut subdirs: Vec<PathBuf> = fs::read_dir(root)
            .map_err(io_err)?
            .filter_map(|entry| entry.ok().map(|e| e.path()))
            .filter(|path| path.is_dir())
            .collect();
        subdirs.sort();

        let mut packs = BTreeMap::new();
        for pack_dir in subdirs {
            let manifest_path = pack_dir.join("pack.toml");
            if !manifest_path.exists() {
                continue;
            }
            let manifest_raw = fs::read_to_string(&manifest_path).map_err(io_err)?;
            let manifest: PackManifest =
                toml::from_str(&manifest_raw).map_err(|err| PackLoadError {
                    message: format!(
                        "Failed to parse manifest '{}': {err}",
                        manifest_path.display()
                    ),
                })?;
            let aliases = load_aliases(&pack_dir, manifest.entrypoints.aliases.as_deref())?;
            let templates = load_templates(&pack_dir, manifest.entrypoints.templates.as_deref())?;
            let lowering_rules =
                load_lowering_rules(&pack_dir, manifest.entrypoints.lowering.as_deref())?;
            validate_category_vocabulary(
                &manifest.categories,
                &manifest_path.display().to_string(),
            )?;
            validate_rule_categories(
                &lowering_rules,
                &manifest.categories,
                &manifest_path.display().to_string(),
            )?;
            let metric_specs =
                load_metric_specs(&pack_dir, manifest.entrypoints.metrics.as_deref())?;
            let validations =
                load_validations(&pack_dir, manifest.entrypoints.validations.as_deref())?;
            let subtotal_specs =
                load_subtotal_specs(&pack_dir, manifest.entrypoints.statements.as_deref())?;
            let statement_specs = load_statement_specs(
                &pack_dir,
                manifest.entrypoints.statements.as_deref(),
                &manifest.categories,
                &subtotal_specs,
                &lowering_rules,
            )?;
            let ontology = load_ontology(
                &pack_dir,
                manifest.entrypoints.ontology.as_deref(),
                &manifest.name,
            )?;
            validate_ontology_against_rules(
                &ontology,
                &lowering_rules,
                &manifest_path.display().to_string(),
            )?;

            packs.insert(
                manifest.name.clone(),
                LoadedPack {
                    manifest,
                    aliases,
                    templates,
                    lowering_rules,
                    metric_specs,
                    subtotal_specs,
                    statement_specs,
                    validations,
                    ontology,
                },
            );
        }

        Ok(Self { packs })
    }

    pub fn list(&self) -> Vec<&LoadedPack> {
        self.packs.values().collect()
    }

    pub fn pack(&self, name: &str) -> Option<&LoadedPack> {
        self.packs.get(name)
    }

    /// What a model using this pack may be about. Empty for a pack that
    /// declares no ontology, so a caller never has to special-case one.
    pub fn ontology(&self, pack_name: &str) -> Option<&PackOntology> {
        self.packs.get(pack_name).map(|pack| &pack.ontology)
    }

    pub fn active_pack(&self, name: &str, version: &str) -> Option<ActivePack> {
        match self.resolve_pack(name, version) {
            PackLookup::Found(active) => Some(active),
            _ => None,
        }
    }

    /// Resolve a `use pack` request, distinguishing absence from a version
    /// mismatch.
    ///
    /// Both used to collapse into `None`, so a pack sitting right there at a
    /// different version was reported as "not found" — sending the reader off
    /// to check their `--packs` path when the real problem was one digit in a
    /// version string.
    pub fn resolve_pack(&self, name: &str, version: &str) -> PackLookup {
        match self.packs.get(name) {
            None => PackLookup::Absent,
            Some(pack) if pack.manifest.version == version => PackLookup::Found(ActivePack {
                name: pack.manifest.name.clone(),
                version: pack.manifest.version.clone(),
            }),
            Some(pack) => PackLookup::VersionMismatch {
                available: pack.manifest.version.clone(),
            },
        }
    }

    pub fn lookup_alias(&self, pack_name: &str, alias: &str) -> Option<&str> {
        self.packs
            .get(pack_name)
            .and_then(|pack| pack.aliases.get(alias))
            .map(String::as_str)
    }

    pub fn metric_specs(&self, pack_name: &str) -> Vec<MetricSpec> {
        self.packs
            .get(pack_name)
            .map(|pack| pack.metric_specs.clone())
            .unwrap_or_default()
    }

    pub fn validations(&self, pack_name: &str) -> Vec<PackValidation> {
        self.packs
            .get(pack_name)
            .map(|pack| pack.validations.clone())
            .unwrap_or_default()
    }

    pub fn lowering_rules(&self, pack_name: &str) -> Vec<LoweringRule> {
        self.packs
            .get(pack_name)
            .map(|pack| pack.lowering_rules.clone())
            .unwrap_or_default()
    }

    /// Model calendars the pack declares it lowers correctly on. Empty means
    /// unconstrained — see `PackManifest::cadences`.
    pub fn cadences(&self, pack_name: &str) -> Vec<String> {
        self.packs
            .get(pack_name)
            .map(|pack| pack.manifest.cadences.clone())
            .unwrap_or_default()
    }

    /// The closed vocabulary a stream's `category` must name, for this pack.
    /// Empty when the pack does not classify.
    pub fn categories(&self, pack_name: &str) -> Vec<String> {
        self.packs
            .get(pack_name)
            .map(|pack| pack.manifest.categories.clone())
            .unwrap_or_default()
    }

    /// Per-period subtotal declarations for this pack, in declaration order.
    /// The order is load-bearing: it is the dependency order.
    /// Declared statements for this pack, in file order.
    pub fn statement_specs(&self, pack_name: &str) -> Vec<StatementSpec> {
        self.packs
            .get(pack_name)
            .map(|pack| pack.statement_specs.clone())
            .unwrap_or_default()
    }

    pub fn subtotal_specs(&self, pack_name: &str) -> Vec<SubtotalSpec> {
        self.packs
            .get(pack_name)
            .map(|pack| pack.subtotal_specs.clone())
            .unwrap_or_default()
    }

    pub fn templates(&self, pack_name: &str) -> Vec<PackTemplate> {
        self.packs
            .get(pack_name)
            .map(|pack| pack.templates.clone())
            .unwrap_or_default()
    }

    pub fn template(&self, pack_name: &str, template_id: &str) -> Option<PackTemplate> {
        self.packs.get(pack_name).and_then(|pack| {
            pack.templates
                .iter()
                .find(|template| template.id == template_id)
                .cloned()
        })
    }

    pub fn expand_template(
        &self,
        pack_name: &str,
        request: TemplateExpansionRequest,
    ) -> Result<TemplateExpansion, PackLoadError> {
        let pack = self.packs.get(pack_name).ok_or_else(|| PackLoadError {
            message: format!("Pack '{pack_name}' is not loaded."),
        })?;
        let template = pack
            .templates
            .iter()
            .find(|template| template.id == request.template)
            .ok_or_else(|| PackLoadError {
                message: format!(
                    "Template '{}' was not found in pack '{}'.",
                    request.template, pack_name
                ),
            })?;
        let text = expand_template_body(template, &request.params);
        Ok(TemplateExpansion {
            generated_nodes: vec![text],
        })
    }
}

fn load_aliases(
    pack_dir: &Path,
    aliases_path: Option<&str>,
) -> Result<BTreeMap<String, String>, PackLoadError> {
    let Some(relative) = aliases_path else {
        return Ok(BTreeMap::new());
    };
    let path = pack_dir.join(relative);
    let raw = fs::read_to_string(&path).map_err(io_err)?;
    parse_aliases(&raw, &path.display().to_string())
}

fn parse_aliases(raw: &str, source: &str) -> Result<BTreeMap<String, String>, PackLoadError> {
    let parsed: AliasFile = toml::from_str(raw).map_err(|err| PackLoadError {
        message: format!("Failed to parse aliases '{source}': {err}"),
    })?;
    Ok(parsed.aliases)
}

fn load_ontology(
    pack_dir: &Path,
    ontology_path: Option<&str>,
    pack_name: &str,
) -> Result<PackOntology, PackLoadError> {
    let Some(relative) = ontology_path else {
        return Ok(PackOntology::default());
    };
    let path = pack_dir.join(relative);
    if !path.exists() {
        return Ok(PackOntology::default());
    }
    let raw = fs::read_to_string(&path).map_err(io_err)?;
    parse_ontology(&raw, &path.display().to_string(), pack_name)
}

/// Parse a pack's ontology and check it is internally coherent.
///
/// The checks run HERE rather than at model-compile time on purpose: a broken
/// vocabulary is the pack author's bug, and catching it at load means every
/// model using the pack is spared a diagnostic that was never about the model.
fn parse_ontology(raw: &str, source: &str, pack_name: &str) -> Result<PackOntology, PackLoadError> {
    let parsed: OntologyFile = toml::from_str(raw).map_err(|err| PackLoadError {
        message: format!("Failed to parse ontology '{source}': {err}"),
    })?;

    if let Some(header) = &parsed.pack {
        if let Some(declared) = &header.ontology_id {
            if declared != pack_name {
                return Err(PackLoadError {
                    message: format!(
                        "Ontology '{source}': declares ontology_id '{declared}' but belongs to pack '{pack_name}'."
                    ),
                });
            }
        }
    }

    let ontology = PackOntology {
        entities: parsed.entities,
        contracts: parsed.contracts,
        lifecycles: parsed.lifecycles,
        references: parsed.references,
        relations: parsed.relations,
    };

    let mut seen_types: BTreeSet<&str> = BTreeSet::new();
    for entity in &ontology.entities {
        if !seen_types.insert(entity.type_id.as_str()) {
            return Err(PackLoadError {
                message: format!(
                    "Ontology '{source}': entity type '{}' is declared twice.",
                    entity.type_id
                ),
            });
        }
        if !ENTITY_FAMILIES.contains(&entity.family.as_str()) {
            return Err(PackLoadError {
                message: format!(
                    "Ontology '{source}': entity '{}' has family '{}', which is not one of {}.",
                    entity.type_id,
                    entity.family,
                    ENTITY_FAMILIES.join(", ")
                ),
            });
        }
        // An asset is underwritten by its class, so an asset without one is
        // not a description of anything. A party has no class to give.
        match (entity.family.as_str(), entity.class.as_deref()) {
            ("asset", None) => {
                return Err(PackLoadError {
                    message: format!(
                        "Ontology '{source}': asset '{}' declares no class. Assets must be one of {}.",
                        entity.type_id,
                        ASSET_CLASSES.join(", ")
                    ),
                });
            }
            ("asset", Some(class)) if !ASSET_CLASSES.contains(&class) => {
                return Err(PackLoadError {
                    message: format!(
                        "Ontology '{source}': asset '{}' has class '{class}', which is not one of {}.",
                        entity.type_id,
                        ASSET_CLASSES.join(", ")
                    ),
                });
            }
            ("party", Some(class)) => {
                return Err(PackLoadError {
                    message: format!(
                        "Ontology '{source}': party '{}' declares class '{class}'. Only assets have a class.",
                        entity.type_id
                    ),
                });
            }
            _ => {}
        }
        if let Some(lifecycle) = &entity.lifecycle {
            if ontology.lifecycle(lifecycle).is_none() {
                return Err(PackLoadError {
                    message: format!(
                        "Ontology '{source}': entity '{}' names lifecycle '{lifecycle}', which is not declared.",
                        entity.type_id
                    ),
                });
            }
        }
    }

    // A lifecycle exists to make the state space TOTAL. An initial state
    // outside the declared set would leave an entity starting nowhere, which
    // is exactly the null-until-first-write behavior this replaces.
    let mut seen_lifecycles: BTreeSet<&str> = BTreeSet::new();
    for lifecycle in &ontology.lifecycles {
        if !seen_lifecycles.insert(lifecycle.lifecycle_id.as_str()) {
            return Err(PackLoadError {
                message: format!(
                    "Ontology '{source}': lifecycle '{}' is declared twice.",
                    lifecycle.lifecycle_id
                ),
            });
        }
        if lifecycle.states.is_empty() {
            return Err(PackLoadError {
                message: format!(
                    "Ontology '{source}': lifecycle '{}' declares no states.",
                    lifecycle.lifecycle_id
                ),
            });
        }
        if !lifecycle.has_state(&lifecycle.initial) {
            return Err(PackLoadError {
                message: format!(
                    "Ontology '{source}': lifecycle '{}' starts at '{}', which is not one of its states ({}).",
                    lifecycle.lifecycle_id,
                    lifecycle.initial,
                    lifecycle.states.join(", ")
                ),
            });
        }
        for transition in &lifecycle.transitions {
            for (label, state) in [("from", &transition.from), ("to", &transition.to)] {
                if !lifecycle.has_state(state) {
                    return Err(PackLoadError {
                        message: format!(
                            "Ontology '{source}': lifecycle '{}' has a transition whose `{label}` is '{state}', which is not one of its states ({}).",
                            lifecycle.lifecycle_id,
                            lifecycle.states.join(", ")
                        ),
                    });
                }
            }
        }
    }

    let mut seen_contracts: BTreeSet<&str> = BTreeSet::new();
    for contract in &ontology.contracts {
        if !seen_contracts.insert(contract.type_id.as_str()) {
            return Err(PackLoadError {
                message: format!(
                    "Ontology '{source}': contract type '{}' is declared twice.",
                    contract.type_id
                ),
            });
        }
        if let Some(family) = &contract.subject_family {
            if !ENTITY_FAMILIES.contains(&family.as_str()) {
                return Err(PackLoadError {
                    message: format!(
                        "Ontology '{source}': contract '{}' has subject_family '{family}', which is not one of {}.",
                        contract.type_id,
                        ENTITY_FAMILIES.join(", ")
                    ),
                });
            }
        }
    }

    // A relation reads from both ends, so both ends have to name a family that
    // exists, and the inverse is what makes the other direction sayable.
    for relation in &ontology.relations {
        for (label, family) in [
            ("from_family", &relation.from_family),
            ("to_family", &relation.to_family),
        ] {
            if !ENTITY_FAMILIES.contains(&family.as_str()) {
                return Err(PackLoadError {
                    message: format!(
                        "Ontology '{source}': relation '{}' has {label} '{family}', which is not one of {}.",
                        relation.relation_id,
                        ENTITY_FAMILIES.join(", ")
                    ),
                });
            }
        }
        if relation.inverse.trim().is_empty() {
            return Err(PackLoadError {
                message: format!(
                    "Ontology '{source}': relation '{}' declares no inverse. Every relation reads from both ends.",
                    relation.relation_id
                ),
            });
        }
    }

    Ok(ontology)
}

/// Every contract type must name a lowering rule that exists, and every rule
/// must have a type.
///
/// This is the join that keeps the vocabulary and the arithmetic from drifting.
/// A type with no rule is a contract a model can declare and get no cash from;
/// a rule with no type is cash with no counterparties and no place in the
/// ontology. Both are silent today, which is how the drift happened.
fn validate_ontology_against_rules(
    ontology: &PackOntology,
    rules: &[LoweringRule],
    source: &str,
) -> Result<(), PackLoadError> {
    if ontology.contracts.is_empty() {
        return Ok(());
    }
    let rule_names: BTreeSet<&str> = rules.iter().map(|r| r.contract_name.as_str()).collect();

    for contract in &ontology.contracts {
        // An election names no rule by design; there is nothing to join.
        let Some(rule_name) = contract.contract_name.as_deref() else {
            continue;
        };
        if !rule_names.contains(rule_name) {
            return Err(PackLoadError {
                message: format!(
                    "Ontology '{source}': contract type '{}' names lowering rule '{rule_name}', which the pack does not declare.",
                    contract.type_id
                ),
            });
        }
    }

    let typed: BTreeSet<&str> = ontology
        .contracts
        .iter()
        .filter_map(|c| c.contract_name.as_deref())
        .collect();
    let untyped: Vec<&str> = rule_names
        .into_iter()
        .filter(|name| !typed.contains(name))
        .collect();
    if !untyped.is_empty() {
        return Err(PackLoadError {
            message: format!(
                "Ontology '{source}': lowering rules with no contract type: {}. A rule with no type has no counterparties and no place in the ontology.",
                untyped.join(", ")
            ),
        });
    }
    Ok(())
}

fn load_validations(
    pack_dir: &Path,
    validations_path: Option<&str>,
) -> Result<Vec<PackValidation>, PackLoadError> {
    let Some(relative) = validations_path else {
        return Ok(vec![]);
    };
    let path = pack_dir.join(relative);
    if !path.exists() {
        return Ok(vec![]);
    }
    let raw = fs::read_to_string(&path).map_err(io_err)?;
    parse_validations(&raw, &path.display().to_string())
}

/// Parses and semantically checks a pack's validations file.
///
/// Every problem here is a load error, surfaced by the compiler as
/// `E4004_MISSING_PACK` — a malformed pack produces a clean diagnostic rather
/// than silently skipping checks or panicking.
fn parse_validations(raw: &str, source: &str) -> Result<Vec<PackValidation>, PackLoadError> {
    let parsed: ValidationsFile = toml::from_str(raw).map_err(|err| PackLoadError {
        message: format!("Failed to parse validations '{source}': {err}"),
    })?;

    if let Some(version) = parsed.schema_version {
        if version != 1 {
            return Err(PackLoadError {
                message: format!(
                    "Validations '{source}': unsupported schema_version {version} (expected 1)."
                ),
            });
        }
    }

    let mut seen: Vec<(String, String)> = Vec::new();
    for validation in &parsed.validations {
        let fail = |message: String| PackLoadError {
            message: format!("Validations '{source}': {message}"),
        };

        if validation.contract.is_some() != validation.contracts.is_empty() {
            return Err(fail(format!(
                "validation '{}' must set exactly one of `contract` or `contracts`.",
                validation.code
            )));
        }
        if validation.code.is_empty() {
            return Err(fail("a validation is missing `code`.".to_string()));
        }
        if let Some(prefix) = &parsed.code_prefix {
            if !validation.code.starts_with(prefix.as_str()) {
                return Err(fail(format!(
                    "code '{}' does not start with the pack's reserved prefix '{prefix}'.",
                    validation.code
                )));
            }
        }

        match validation.check {
            ValidationCheck::TermPresent | ValidationCheck::TermNumber => {
                if validation.term.is_none() {
                    return Err(fail(format!(
                        "validation '{}' requires `term`.",
                        validation.code
                    )));
                }
            }
            ValidationCheck::TermEnum => {
                if validation.term.is_none() || validation.values.is_empty() {
                    return Err(fail(format!(
                        "validation '{}' requires `term` and a non-empty `values`.",
                        validation.code
                    )));
                }
            }
            ValidationCheck::AnyTermPresent | ValidationCheck::TermsMutuallyExclusive => {
                if validation.terms.is_empty() {
                    return Err(fail(format!(
                        "validation '{}' requires a non-empty `terms`.",
                        validation.code
                    )));
                }
            }
            ValidationCheck::TermCompare => {
                if validation.left.is_none()
                    || validation.right.is_none()
                    || validation.op.is_none()
                {
                    return Err(fail(format!(
                        "validation '{}' requires `left`, `op`, and `right`.",
                        validation.code
                    )));
                }
            }
            ValidationCheck::TermRangeWithinTimeline => {}
        }

        let has_bounds = validation.min.is_some()
            || validation.max.is_some()
            || validation.exclusive_min.is_some()
            || validation.exclusive_max.is_some();
        if has_bounds && validation.check != ValidationCheck::TermNumber {
            return Err(fail(format!(
                "validation '{}' declares bounds, which only apply to check 'term_number'.",
                validation.code
            )));
        }
        if let (Some(min), Some(max)) = (validation.min, validation.max) {
            if min > max {
                return Err(fail(format!(
                    "validation '{}' has min {min} greater than max {max}.",
                    validation.code
                )));
            }
        }

        for contract in validation.contract_names() {
            let key = (contract.to_string(), validation.code.clone());
            if seen.contains(&key) {
                return Err(fail(format!(
                    "duplicate code '{}' for contract '{contract}'.",
                    validation.code
                )));
            }
            seen.push(key);
        }
    }

    let mut validations = parsed.validations;
    // Deterministic order: diagnostics are sorted downstream, but a stable
    // load order keeps behavior reproducible.
    validations.sort_by(|a, b| {
        a.contract_names()
            .cmp(&b.contract_names())
            .then_with(|| a.code.cmp(&b.code))
    });
    Ok(validations)
}

fn load_statement_specs(
    pack_dir: &Path,
    statements_path: Option<&str>,
    categories: &[String],
    subtotals: &[SubtotalSpec],
    rules: &[LoweringRule],
) -> Result<Vec<StatementSpec>, PackLoadError> {
    let Some(relative) = statements_path else {
        return Ok(vec![]);
    };
    let path = pack_dir.join(relative);
    let raw = fs::read_to_string(&path).map_err(io_err)?;
    parse_statement_specs(
        &raw,
        &path.display().to_string(),
        categories,
        subtotals,
        rules,
    )
}

fn load_subtotal_specs(
    pack_dir: &Path,
    statements_path: Option<&str>,
) -> Result<Vec<SubtotalSpec>, PackLoadError> {
    let Some(relative) = statements_path else {
        return Ok(vec![]);
    };
    let path = pack_dir.join(relative);
    let raw = fs::read_to_string(&path).map_err(io_err)?;
    parse_subtotal_specs(&raw, &path.display().to_string())
}

fn load_metric_specs(
    pack_dir: &Path,
    metrics_path: Option<&str>,
) -> Result<Vec<MetricSpec>, PackLoadError> {
    let Some(relative) = metrics_path else {
        return Ok(vec![]);
    };
    let path = pack_dir.join(relative);
    let raw = fs::read_to_string(&path).map_err(io_err)?;
    parse_metric_specs(&raw, &path.display().to_string())
}

/// Parse and validate `[[subtotals]]`.
///
/// The forward-reference check is the cycle guard. Subtotals are evaluated in
/// declaration order, so a reference to something declared later cannot be
/// satisfied — and because only earlier ids are reachable, no cycle can be
/// expressed at all. That is the same argument `docs/14_state_and_recurrence.md`
/// §5 makes about waterfalls: an authored order needs no solver.
/// Parse and validate `[[statements]]`.
///
/// The interesting check is the last one: every category the pack declares must
/// appear in exactly ONE `line` row. That is what makes a statement's bottom
/// line reconcile to net cash flow rather than merely resemble it, and it is
/// checkable here — statically, before any model runs — because a category is
/// declared rather than discovered.
///
/// Both failure directions matter, and the second is worse. A category in no
/// row means cash the statement never shows, so the bottom line is short. A
/// category in two rows means cash counted twice, and a statement that is
/// wrong by double-counting looks entirely plausible.
fn parse_statement_specs(
    raw: &str,
    source: &str,
    categories: &[String],
    subtotals: &[SubtotalSpec],
    rules: &[LoweringRule],
) -> Result<Vec<StatementSpec>, PackLoadError> {
    let parsed: StatementsFile = toml::from_str(raw).map_err(|err| PackLoadError {
        message: format!("Failed to parse statements '{source}': {err}"),
    })?;
    let mut seen_ids: Vec<&str> = Vec::new();
    let mut defaults = 0usize;
    for spec in &parsed.statements {
        let err = |msg: String| PackLoadError {
            message: format!("Statement '{}' in '{source}': {msg}", spec.id),
        };
        if seen_ids.contains(&spec.id.as_str()) {
            return Err(err("declared twice.".to_string()));
        }
        seen_ids.push(&spec.id);
        if spec.default {
            defaults += 1;
        }
        // A `line` row may claim by CATEGORY or by STREAM selector. A stream
        // selector names one instance of a family, and the rule that emits that
        // family declares its category — so a stream row claims a category just
        // as surely as a category row does, and the completeness check below
        // must see it or itemising a family becomes undeclarable.
        //
        // Which is exactly what happened: `cre.opex_line` is one contract
        // instanced per expense, every instance carries the same category, and
        // a category row can therefore only ever render one number. Rows by
        // stream are how nine expense lines become nine lines.
        //
        // What a stream row does NOT claim is completeness of the family: a
        // modeller's own instance matches no row, and the evaluator emits a
        // `residual` row for it. That is the runtime half of this guarantee and
        // it is why the static half can stop at the category.
        let category_of = |selector: &str| -> Option<&str> {
            rules.iter().find_map(|rule| {
                let base = rule.stream_name.replace("{{contract.dot_suffix}}", "");
                let hit = selector == base
                    || selector == format!("{base}.*")
                    || selector
                        .strip_prefix(&base)
                        .is_some_and(|rest| rest.starts_with('.'));
                hit.then_some(rule.category.as_str())
            })
        };
        let mut claimed: Vec<&str> = Vec::new();
        let mut claimed_by_stream: Vec<&str> = Vec::new();
        for row in &spec.rows {
            match row.kind.as_str() {
                "spacer" => {}
                "line" => {
                    if row.categories.is_empty() && row.streams.is_empty() {
                        return Err(err(format!(
                            "row '{}' is a line with neither categories nor streams.",
                            row.label
                        )));
                    }
                    for c in &row.categories {
                        if !categories.iter().any(|d| d == c) {
                            return Err(err(format!(
                                "row '{}' claims category '{c}', which the pack does not declare.",
                                row.label
                            )));
                        }
                        if claimed.contains(&c.as_str()) {
                            return Err(err(format!(
                                "category '{c}' appears in more than one line row. Cash counted \
                                 twice makes a bottom line that looks plausible and is wrong."
                            )));
                        }
                        claimed.push(c);
                    }
                    for sel in &row.streams {
                        let Some(cat) = category_of(sel) else {
                            return Err(err(format!(
                                "row '{}' selects stream '{sel}', which no lowering rule emits. \
                                 A stream row names an instance of a family the pack lowers; if \
                                 the name is right, the rule that emits it is missing.",
                                row.label
                            )));
                        };
                        if claimed.contains(&cat) {
                            return Err(err(format!(
                                "row '{}' selects stream '{sel}', whose category '{cat}' is \
                                 already claimed by a category row. Every stream in that family \
                                 carries that category, so it would be counted twice.",
                                row.label
                            )));
                        }
                        if !claimed_by_stream.contains(&cat) {
                            claimed_by_stream.push(cat);
                        }
                    }
                }
                "subtotal" | "ratio" => {
                    let Some(id) = &row.subtotal else {
                        return Err(err(format!(
                            "row '{}' is a {} with no `subtotal`.",
                            row.label, row.kind
                        )));
                    };
                    let Some(found) = subtotals.iter().find(|s| &s.id == id) else {
                        return Err(err(format!(
                            "row '{}' reads '{id}', which is not a declared subtotal.",
                            row.label
                        )));
                    };
                    if row.kind == "ratio" && found.kind != "number" {
                        return Err(err(format!(
                            "row '{}' is a ratio but '{id}' is {}.",
                            row.label, found.kind
                        )));
                    }
                }
                "residual" => {
                    return Err(err(
                        "a `residual` row is emitted by the evaluator for streams no row \
                         claimed; it cannot be authored."
                            .to_string(),
                    ))
                }
                other => {
                    return Err(err(format!(
                        "row '{}' has unknown kind '{other}'.",
                        row.label
                    )))
                }
            }
            if let Some(d) = &row.display {
                if !matches!(d.as_str(), "natural" | "positive" | "negative") {
                    return Err(err(format!(
                        "row '{}' has unknown display '{d}'.",
                        row.label
                    )));
                }
            }
        }
        // Completeness, checked statically. A category counts as claimed if a
        // category row folds it OR a stream row names an instance of a family
        // that carries it.
        if let Some(dup) = claimed.iter().find(|c| claimed_by_stream.contains(c)) {
            return Err(err(format!(
                "category '{dup}' is claimed by a category row and by a stream row. Every \
                 stream a stream row names also carries that category, so it would be \
                 counted twice."
            )));
        }
        let missing: Vec<&String> = categories
            .iter()
            .filter(|c| !claimed.contains(&c.as_str()) && !claimed_by_stream.contains(&c.as_str()))
            .collect();
        if !missing.is_empty() {
            return Err(err(format!(
                "these categories appear in no line row, so their cash would be missing from \
                 the bottom line: {}.",
                missing
                    .iter()
                    .map(|s| s.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            )));
        }
    }
    if defaults > 1 {
        return Err(PackLoadError {
            message: format!("'{source}': more than one statement is marked default."),
        });
    }
    Ok(parsed.statements)
}

fn parse_subtotal_specs(raw: &str, source: &str) -> Result<Vec<SubtotalSpec>, PackLoadError> {
    let parsed: StatementsFile = toml::from_str(raw).map_err(|err| PackLoadError {
        message: format!("Failed to parse statements '{source}': {err}"),
    })?;
    let mut seen: Vec<&str> = Vec::new();
    for spec in &parsed.subtotals {
        let err = |msg: String| PackLoadError {
            message: format!("Subtotal '{}' in '{source}': {msg}", spec.id),
        };
        if !spec.id.starts_with("domain.") {
            return Err(err("id must start with 'domain.'.".to_string()));
        }
        if seen.contains(&spec.id.as_str()) {
            return Err(err("declared twice.".to_string()));
        }
        if !matches!(spec.kind.as_str(), "money" | "number") {
            return Err(err(format!("unknown kind '{}'.", spec.kind)));
        }
        match spec.op.as_str() {
            "sum" | "negated_sum" | "cumulative" | "negated_cumulative" => {
                if spec.categories.is_empty()
                    && spec.streams.is_empty()
                    && spec.subtotals.is_empty()
                {
                    return Err(err(
                        "this op needs at least one of categories, streams or subtotals."
                            .to_string(),
                    ));
                }
                if spec.kind != "money" {
                    return Err(err("a sum or cumulative is money.".to_string()));
                }
            }
            "ratio" => {
                let (Some(num), Some(den)) = (&spec.numerator, &spec.denominator) else {
                    return Err(err(
                        "op 'ratio' requires numerator and denominator.".to_string()
                    ));
                };
                if spec.kind != "number" {
                    return Err(err("a ratio is a number, not money.".to_string()));
                }
                for side in [num, den] {
                    if !seen.contains(&side.as_str()) {
                        return Err(err(format!(
                            "'{side}' is not a subtotal declared earlier in this file. \
                             Order is the dependency order; move it above."
                        )));
                    }
                }
            }
            other => return Err(err(format!("unknown op '{other}'."))),
        }
        for referenced in &spec.subtotals {
            if !seen.contains(&referenced.as_str()) {
                return Err(err(format!(
                    "'{referenced}' is not a subtotal declared earlier in this file. \
                     Order is the dependency order; move it above."
                )));
            }
        }
        seen.push(&spec.id);
    }
    Ok(parsed.subtotals)
}

fn parse_metric_specs(raw: &str, source: &str) -> Result<Vec<MetricSpec>, PackLoadError> {
    let parsed: MetricsFile = toml::from_str(raw).map_err(|err| PackLoadError {
        message: format!("Failed to parse metrics '{source}': {err}"),
    })?;
    for spec in &parsed.metrics {
        match spec.op.as_str() {
            "sum" | "negated_sum" | "cumulative" | "negated_cumulative" => {}
            "subtotal_total" => {
                if spec.subtotal.is_none() {
                    return Err(PackLoadError {
                        message: format!(
                            "Metric '{}': op 'subtotal_total' requires `subtotal`.",
                            spec.id
                        ),
                    });
                }
            }
            "wal_years" => {
                if spec.numerator_streams.is_empty() {
                    return Err(PackLoadError {
                        message: format!(
                            "Metric '{}': op 'wal_years' requires numerator_streams.",
                            spec.id
                        ),
                    });
                }
                if spec.kind != "number" {
                    return Err(PackLoadError {
                        message: format!(
                            "Metric '{}': op 'wal_years' requires kind 'number'.",
                            spec.id
                        ),
                    });
                }
            }
            "ratio" => {
                if spec.numerator_metric.is_none() || spec.denominator_metric.is_none() {
                    return Err(PackLoadError {
                        message: format!(
                            "Metric '{}': op 'ratio' requires numerator_metric and denominator_metric.",
                            spec.id
                        ),
                    });
                }
            }
            other => {
                return Err(PackLoadError {
                    message: format!(
                        "Metric '{}': unknown op '{other}' (expected sum, negated_sum, ratio, wal_years).",
                        spec.id
                    ),
                });
            }
        }
        if !matches!(spec.kind.as_str(), "money" | "number") {
            return Err(PackLoadError {
                message: format!(
                    "Metric '{}': unknown kind '{}' (expected money or number).",
                    spec.id, spec.kind
                ),
            });
        }
    }
    Ok(parsed.metrics)
}

fn load_lowering_rules(
    pack_dir: &Path,
    lowering_path: Option<&str>,
) -> Result<Vec<LoweringRule>, PackLoadError> {
    let Some(relative) = lowering_path else {
        return Ok(vec![]);
    };
    let path = pack_dir.join(relative);
    let raw = fs::read_to_string(&path).map_err(io_err)?;
    parse_lowering_rules(&raw, &path.display().to_string())
}

/// The cash flow statement's three sections. A category path must start with
/// one of these so that a fold written against an unfamiliar pack — or a
/// generic statement — still has something universal to aggregate on.
///
/// CFDL fixes this vocabulary and nothing below it. Which section a category
/// belongs under is a policy question the pack answers: IFRS and US GAAP
/// disagree about interest paid, and a lender's interest received is operating
/// revenue rather than financing at all.
pub const CATEGORY_ROOTS: [&str; 3] = ["operating", "investing", "financing"];

/// Every declared category must be a dotted path rooted in a known section.
fn validate_category_vocabulary(categories: &[String], source: &str) -> Result<(), PackLoadError> {
    for category in categories {
        let root = category.split('.').next().unwrap_or("");
        if !CATEGORY_ROOTS.contains(&root) {
            return Err(PackLoadError {
                message: format!(
                    "Pack '{source}' declares category '{category}', whose root segment \
                     '{root}' is not one of {}. A category is a path into the cash flow \
                     statement, so it has to say which section it belongs to.",
                    CATEGORY_ROOTS.join(", ")
                ),
            });
        }
        if category.split('.').any(|seg| seg.is_empty()) {
            return Err(PackLoadError {
                message: format!(
                    "Pack '{source}' declares category '{category}', which has an empty path \
                     segment."
                ),
            });
        }
    }
    Ok(())
}

/// Every rule's `category` must be one the manifest declares.
///
/// Checked here rather than in `parse_lowering_rules` because the vocabulary
/// lives in the manifest and the rules in another file, so this is the first
/// point that sees both. A pack that fails this does not load at all, which is
/// the right severity: a mistyped category would otherwise become a bucket that
/// no fold names and no statement reports, and the stream would simply go
/// missing from its subtotal while still appearing as a line.
fn validate_rule_categories(
    rules: &[LoweringRule],
    categories: &[String],
    source: &str,
) -> Result<(), PackLoadError> {
    for rule in rules {
        // A RULE THAT EMITS A STREAM STATES ITS CATEGORY.
        //
        // A contract lowers one or more streams, and the pack is the thing that
        // knows what each of them IS. A rule that says nothing emits cash with
        // no category: real money in `model.total`, absent from every subtotal,
        // and not reachable by `E5029`, which sees only what the MODEL declared.
        //
        // Scoped to stream-emitting rules on purpose. A rule may lower a FIELD
        // rather than cash — every pool in `credit` carries one for its balance
        // — and a field is not classified into a cash flow statement, so
        // demanding a category of it would be asking the wrong question.
        if !rule.stream_name.is_empty() && rule.category.is_empty() {
            return Err(PackLoadError {
                message: format!(
                    "Lowering rule '{}' in '{source}' declares no category, so the stream it                      emits would fold into no subtotal. A contract lowers one or more streams                      and the pack states what each one is.",
                    rule.id
                ),
            });
        }
        if categories.iter().any(|c| c == &rule.category) {
            continue;
        }
        let known = if categories.is_empty() {
            "the pack declares none".to_string()
        } else {
            categories.join(", ")
        };
        return Err(PackLoadError {
            message: format!(
                "Lowering rule '{}' in '{source}' declares category '{}', which the pack \
                 manifest does not list. Known categories: {known}.",
                rule.id, rule.category
            ),
        });
    }
    Ok(())
}

fn parse_lowering_rules(raw: &str, source: &str) -> Result<Vec<LoweringRule>, PackLoadError> {
    let parsed: LoweringFile = toml::from_str(raw).map_err(|err| PackLoadError {
        message: format!("Failed to parse lowering rules '{source}': {err}"),
    })?;
    for rule in &parsed.rules {
        // Templated stream names ({{contract.*}}) are validated post-expansion
        // by the compiler.
        if !rule.stream_name.contains("{{") && !is_qualified_name(&rule.stream_name) {
            return Err(PackLoadError {
                message: format!(
                    "Lowering rule '{}' has invalid stream_name '{}'; expected dotted qualified name.",
                    rule.id, rule.stream_name
                ),
            });
        }
        if !(rule.owner_entity.is_empty()
            || rule.owner_entity == "${subject}"
            || is_qualified_name(&rule.owner_entity))
        {
            return Err(PackLoadError {
                message: format!(
                    "Lowering rule '{}' has invalid owner_entity '{}'; expected '${{subject}}' or dotted qualified entity symbol.",
                    rule.id, rule.owner_entity
                ),
            });
        }
    }
    Ok(parsed.rules)
}

fn load_templates(
    pack_dir: &Path,
    templates_path: Option<&str>,
) -> Result<Vec<PackTemplate>, PackLoadError> {
    let Some(relative) = templates_path else {
        return Ok(vec![]);
    };
    let path = pack_dir.join(relative);
    let raw = fs::read_to_string(&path).map_err(io_err)?;
    parse_templates(&raw, &path.display().to_string())
}

fn parse_templates(raw: &str, source: &str) -> Result<Vec<PackTemplate>, PackLoadError> {
    let mut parsed: TemplateFile = toml::from_str(raw).map_err(|err| PackLoadError {
        message: format!("Failed to parse templates '{source}': {err}"),
    })?;
    parsed.templates.sort_by(|a, b| a.id.cmp(&b.id));
    Ok(parsed.templates)
}

fn expand_template_body(template: &PackTemplate, params: &BTreeMap<String, String>) -> String {
    let mut output = String::with_capacity(template.body.len());
    let chars = template.body.chars().collect::<Vec<_>>();
    let mut idx = 0usize;
    while idx < chars.len() {
        if chars[idx] == '$' && idx + 1 < chars.len() && chars[idx + 1] == '{' {
            idx += 2;
            let mut key = String::new();
            while idx < chars.len() && chars[idx] != '}' {
                key.push(chars[idx]);
                idx += 1;
            }
            if idx < chars.len() && chars[idx] == '}' {
                idx += 1;
            }
            let value = params
                .get(&key)
                .or_else(|| template.defaults.get(&key))
                .cloned()
                .unwrap_or_default();
            output.push_str(&value);
        } else {
            output.push(chars[idx]);
            idx += 1;
        }
    }
    output
}

pub fn render_template(template: &PackTemplate, params: &BTreeMap<String, String>) -> String {
    expand_template_body(template, params)
}

fn io_err(err: std::io::Error) -> PackLoadError {
    PackLoadError {
        message: format!("I/O error while loading packs: {err}"),
    }
}

fn is_qualified_name(value: &str) -> bool {
    let mut parts = value.split('.');
    let Some(first) = parts.next() else {
        return false;
    };
    if !is_ident_segment(first) {
        return false;
    }
    let mut count = 1usize;
    for part in parts {
        if !is_ident_segment(part) {
            return false;
        }
        count += 1;
    }
    count >= 2
}

fn is_ident_segment(segment: &str) -> bool {
    let mut chars = segment.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    if !(first.is_ascii_alphabetic() || first == '_') {
        return false;
    }
    chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
struct AliasFile {
    #[serde(default)]
    aliases: BTreeMap<String, String>,
}

#[cfg(all(test, feature = "embedded-packs"))]
mod embedded_tests {
    use super::*;

    #[test]
    fn embedded_matches_filesystem_packs() {
        let embedded = PackRegistry::load_embedded().expect("embedded packs load");
        let fs_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../packs")
            .canonicalize()
            .expect("packs dir");
        let from_fs = PackRegistry::load_from_dir(&fs_root).expect("fs packs load");
        for name in ["cre", "opco"] {
            assert_eq!(
                embedded.lowering_rules(name),
                from_fs.lowering_rules(name),
                "{name} rules"
            );
            assert_eq!(
                embedded.metric_specs(name),
                from_fs.metric_specs(name),
                "{name} metrics"
            );
            assert!(
                !embedded.lowering_rules(name).is_empty(),
                "{name} non-empty"
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn loads_pack_registry_from_filesystem() {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_nanos();
        let root = std::env::temp_dir().join(format!("cfdl-pack-test-{unique}"));
        let pack_dir = root.join("testpack");
        let lowering_dir = pack_dir.join("lowering");
        fs::create_dir_all(&lowering_dir).expect("create test dirs");
        fs::write(
            pack_dir.join("pack.toml"),
            r#"name = "testpack"
version = "0.1.0"
categories = ["operating.revenue.other"]
[entrypoints]
aliases = "aliases.toml"
templates = "templates.toml"
lowering = "lowering/rules.toml"
"#,
        )
        .expect("write manifest");
        fs::write(
            pack_dir.join("aliases.toml"),
            r#"[aliases]
Lease = "core.Contract"
"#,
        )
        .expect("write aliases");
        fs::write(
            pack_dir.join("templates.toml"),
            r#"[[templates]]
id = "lease.basic"
label = "Lease Basic"
kind = "contract"
body = "contract core.lease ${name} term ${term_start}..${term_end}"

[templates.defaults]
name = "lease_main"
term_start = "2026-01"
term_end = "2026-12"
"#,
        )
        .expect("write templates");
        fs::write(
            lowering_dir.join("rules.toml"),
            r#"[[rules]]
id = "rule"
contract_name = "lease_contract"
stream_name = "pack.stream"
owner_entity = "legal.borrower"
direction = "inflow"
category = "operating.revenue.other"
currency = "USD"
amount_expr = "1"
schedule_kind = "every"
schedule_from = "2026-01"
schedule_to = "2026-12"
"#,
        )
        .expect("write lowering");

        let registry = PackRegistry::load_from_dir(&root).expect("registry");
        assert!(registry.active_pack("testpack", "0.1.0").is_some());
        assert_eq!(
            registry.lookup_alias("testpack", "Lease"),
            Some("core.Contract")
        );
        assert_eq!(registry.templates("testpack").len(), 1);
        assert_eq!(registry.lowering_rules("testpack").len(), 1);
        let expansion = registry
            .expand_template(
                "testpack",
                TemplateExpansionRequest {
                    template: "lease.basic".to_string(),
                    params: BTreeMap::from([("name".to_string(), "lease_001".to_string())]),
                },
            )
            .expect("template expansion");
        assert_eq!(
            expansion.generated_nodes,
            vec!["contract core.lease lease_001 term 2026-01..2026-12".to_string()]
        );

        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn rejects_non_qualified_stream_name_in_lowering_rule() {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_nanos();
        let root = std::env::temp_dir().join(format!("cfdl-pack-test-invalid-stream-{unique}"));
        let pack_dir = root.join("testpack");
        let lowering_dir = pack_dir.join("lowering");
        fs::create_dir_all(&lowering_dir).expect("create dirs");
        fs::write(
            pack_dir.join("pack.toml"),
            r#"name = "testpack"
version = "0.1.0"
categories = ["operating.revenue.other"]
[entrypoints]
lowering = "lowering/rules.toml"
"#,
        )
        .expect("write manifest");
        fs::write(
            lowering_dir.join("rules.toml"),
            r#"[[rules]]
id = "rule_bad"
contract_name = "lease_contract"
stream_name = "flatname"
owner_entity = "legal.borrower"
direction = "inflow"
category = "operating.revenue.other"
currency = "USD"
amount_expr = "1"
schedule_kind = "every"
schedule_from = "2026-01"
schedule_to = "2026-12"
"#,
        )
        .expect("write lowering");

        let err = PackRegistry::load_from_dir(&root).expect_err("invalid lowering");
        assert!(err.message.contains("invalid stream_name"));
        let _ = fs::remove_dir_all(&root);
    }
}

#[cfg(test)]
mod validation_tests {
    use super::*;

    const VALID: &str = r#"
schema_version = 1
code_prefix = "E6"

[[validations]]
contract = "cre.lease"
code = "E6001_CRE_LEASE_MISSING_BASE_RENT"
message = "missing base_rent"
check = "term_present"
term = "base_rent"

[[validations]]
contracts = ["cre.ops_revenue", "cre.opex_line"]
code = "E6020_CRE_OPS_MISSING_AMOUNT"
message = "missing amount"
check = "term_number"
term = "amount"
when = "present"
exclusive_min = 0.0
"#;

    #[test]
    fn parses_a_valid_file() {
        let parsed = parse_validations(VALID, "test").expect("parses");
        assert_eq!(parsed.len(), 2);
        assert!(parsed
            .iter()
            .any(|v| v.contract_names() == vec!["cre.lease"]));
    }

    #[test]
    fn rejects_unknown_fields() {
        let raw = VALID.replace("exclusive_min", "exclusiv_min");
        let err = parse_validations(&raw, "test").expect_err("typo must not be silently ignored");
        assert!(err.message.contains("Failed to parse validations"));
    }

    #[test]
    fn rejects_code_outside_the_reserved_prefix() {
        let raw = VALID.replace("E6001_CRE_LEASE_MISSING_BASE_RENT", "E7001_WRONG_PACK");
        let err = parse_validations(&raw, "test").expect_err("prefix is enforced");
        assert!(err.message.contains("reserved prefix"));
    }

    #[test]
    fn rejects_inverted_bounds() {
        let raw = r#"
code_prefix = "E6"
[[validations]]
contract = "c.x"
code = "E6099_X"
message = "m"
check = "term_number"
term = "t"
min = 10.0
max = 1.0
"#;
        let err = parse_validations(raw, "test").expect_err("min > max is rejected");
        assert!(err.message.contains("greater than max"));
    }

    #[test]
    fn rejects_bounds_on_non_numeric_checks() {
        let raw = r#"
[[validations]]
contract = "c.x"
code = "X1"
message = "m"
check = "term_present"
term = "t"
min = 1.0
"#;
        let err = parse_validations(raw, "test").expect_err("bounds need term_number");
        assert!(err.message.contains("only apply to check 'term_number'"));
    }

    #[test]
    fn rejects_both_or_neither_contract_forms() {
        for body in [
            r#"contract = "a.b"
contracts = ["c.d"]"#,
            "",
        ] {
            let raw = format!(
                r#"
[[validations]]
{body}
code = "X1"
message = "m"
check = "term_present"
term = "t"
"#
            );
            let err = parse_validations(&raw, "test").expect_err("exactly one form required");
            assert!(err.message.contains("exactly one"));
        }
    }

    #[test]
    fn rejects_duplicate_code_for_a_contract() {
        let raw = format!(
            "{VALID}\n{}",
            VALID
                .replace("schema_version = 1", "")
                .replace("code_prefix = \"E6\"", "")
        );
        let err = parse_validations(&raw, "test").expect_err("duplicates are rejected");
        assert!(err.message.contains("duplicate code"));
    }

    #[test]
    fn rejects_unsupported_schema_version() {
        let raw = VALID.replace("schema_version = 1", "schema_version = 2");
        let err = parse_validations(&raw, "test").expect_err("version is checked");
        assert!(err.message.contains("unsupported schema_version"));
    }

    #[test]
    fn instance_matching_covers_suffixed_contracts() {
        let raw = r#"
[[validations]]
contract = "cre.lease_unit"
code = "X1"
message = "m"
check = "term_present"
term = "rent_year"
"#;
        let parsed = parse_validations(raw, "test").expect("parses");
        let v = &parsed[0];
        assert!(v.applies_to("cre.lease_unit"));
        assert!(v.applies_to("cre.lease_unit.tenant_a"));
        // The separator must be a dot, or `cre.debt` would claim
        // `cre.debt_service`.
        assert!(!v.applies_to("cre.lease_unit_other"));
        assert!(!v.applies_to("cre.lease"));
    }

    #[test]
    fn instance_matching_is_unconditional() {
        // This asserted the OPPOSITE until the `match` field was removed:
        // matching defaulted to exact, so a validation reached `cre.lease` and
        // silently skipped `cre.lease.primary` — the form a model must use the
        // moment a deal has two leases.
        let parsed = parse_validations(VALID, "test").expect("parses");
        let lease = parsed
            .iter()
            .find(|v| v.code.starts_with("E6001"))
            .expect("lease rule");
        assert!(lease.applies_to("cre.lease"));
        assert!(lease.applies_to("cre.lease.primary"));
    }

    #[test]
    fn a_leftover_match_declaration_is_rejected_loudly() {
        // A pack written against the old surface must fail to load, not load
        // with the key ignored. Silence is what the field cost us the first
        // time.
        let raw = r#"
[[validations]]
contract = "cre.lease_unit"
match = "instance"
code = "X1"
message = "m"
check = "term_present"
term = "rent_year"
"#;
        let err = parse_validations(raw, "test").expect_err("`match` is gone");
        assert!(err.message.contains("match"), "{}", err.message);
    }
}

#[cfg(test)]
mod ontology_tests {
    use super::*;

    const MINIMAL: &str = r#"
[[entities]]
type_id = "T.Asset.Thing"
family = "asset"
class = "real"
lifecycle = "t.thing"

[[lifecycles]]
lifecycle_id = "t.thing"
initial = "idle"
states = ["idle", "running"]
[[lifecycles.transitions]]
from = "idle"
to = "running"

[[contracts]]
type_id = "T.Contract.Deal"
contract_name = "t.deal"
parties = ["buyer", "seller"]

# An election: a contract whose cash is a payoff the holder takes, so it names
# no lowering rule and is exempt from the rule join.
[[contracts]]
type_id = "T.Contract.Call"
parties = ["grantor", "holder"]
"#;

    /// Built through the real parser rather than a struct literal, so these
    /// tests break if a rule's required shape changes.
    fn rule(name: &str) -> LoweringRule {
        let raw = format!(
            r#"
[[rules]]
id = "r"
contract_name = "{name}"
stream_name = "t.stream"
owner_entity = "${{subject}}"
direction = "inflow"
category = "operating.revenue.other"
amount_expr = "1"
schedule_kind = "every"
schedule_from = "2026-01"
schedule_to = "2026-01"
"#
        );
        parse_lowering_rules(&raw, "test")
            .expect("minimal rule parses")
            .remove(0)
    }

    #[test]
    fn parses_a_coherent_ontology() {
        let o = parse_ontology(MINIMAL, "test", "t").expect("parses");
        assert_eq!(o.entities.len(), 1);
        assert_eq!(o.contract_for_rule("t.deal").unwrap().parties.len(), 2);
        assert!(o.lifecycle("t.thing").unwrap().permits("idle", "running"));
        // The relation is declared one way only, so the reverse is not a legal move.
        assert!(!o.lifecycle("t.thing").unwrap().permits("running", "idle"));
    }

    #[test]
    fn an_asset_must_declare_a_class() {
        let raw = r#"
[[entities]]
type_id = "T.Asset.Thing"
family = "asset"
"#;
        let err = parse_ontology(raw, "test", "t").expect_err("class is required on an asset");
        assert!(err.message.contains("declares no class"), "{}", err.message);
    }

    #[test]
    fn a_party_has_no_class_to_give() {
        let raw = r#"
[[entities]]
type_id = "T.Party.Someone"
family = "party"
class = "real"
"#;
        let err = parse_ontology(raw, "test", "t").expect_err("a party has no class");
        assert!(
            err.message.contains("Only assets have a class"),
            "{}",
            err.message
        );
    }

    /// The whole point of declaring a state space is that an entity is always
    /// in exactly one of its states. An initial state outside the set would
    /// leave it starting nowhere.
    #[test]
    fn initial_state_must_be_one_of_the_declared_states() {
        let raw = r#"
[[lifecycles]]
lifecycle_id = "t.thing"
initial = "somewhere_else"
states = ["idle", "running"]
"#;
        let err = parse_ontology(raw, "test", "t").expect_err("initial must be declared");
        assert!(
            err.message.contains("not one of its states"),
            "{}",
            err.message
        );
    }

    #[test]
    fn a_transition_cannot_name_an_undeclared_state() {
        let raw = r#"
[[lifecycles]]
lifecycle_id = "t.thing"
initial = "idle"
states = ["idle"]
[[lifecycles.transitions]]
from = "idle"
to = "typo"
"#;
        let err = parse_ontology(raw, "test", "t").expect_err("transition target must exist");
        assert!(err.message.contains("`to` is 'typo'"), "{}", err.message);
    }

    #[test]
    fn an_entity_cannot_name_an_undeclared_lifecycle() {
        let raw = r#"
[[entities]]
type_id = "T.Asset.Thing"
family = "asset"
class = "real"
lifecycle = "t.missing"
"#;
        let err = parse_ontology(raw, "test", "t").expect_err("lifecycle must exist");
        assert!(err.message.contains("not declared"), "{}", err.message);
    }

    #[test]
    fn ontology_id_must_match_the_pack_it_lives_in() {
        let raw = "[pack]\nontology_id = \"cre\"\n";
        let err = parse_ontology(raw, "test", "energy").expect_err("id must match the pack");
        assert!(
            err.message.contains("belongs to pack 'energy'"),
            "{}",
            err.message
        );
    }

    /// The join that keeps the vocabulary and the arithmetic from drifting.
    #[test]
    fn every_contract_type_must_name_a_rule_that_exists() {
        let o = parse_ontology(MINIMAL, "test", "t").unwrap();
        let err = validate_ontology_against_rules(&o, &[rule("t.something_else")], "test")
            .expect_err("a type with no rule produces no cash");
        assert!(
            err.message.contains("which the pack does not declare"),
            "{}",
            err.message
        );
    }

    /// An election is exempt from the join by design — it has no rule to name.
    #[test]
    fn an_election_needs_no_lowering_rule() {
        let o = parse_ontology(MINIMAL, "test", "t").unwrap();
        assert_eq!(o.elections().count(), 1);
        validate_ontology_against_rules(&o, &[rule("t.deal")], "test")
            .expect("an election is not a missing rule");
    }

    #[test]
    fn every_rule_must_have_a_contract_type() {
        let o = parse_ontology(MINIMAL, "test", "t").unwrap();
        let err = validate_ontology_against_rules(&o, &[rule("t.deal"), rule("t.orphan")], "test")
            .expect_err("a rule with no type has no counterparties");
        assert!(err.message.contains("t.orphan"), "{}", err.message);
    }

    /// A pack that declares no ontology keeps loading exactly as before.
    #[test]
    fn no_ontology_is_not_an_error() {
        let empty = PackOntology::default();
        assert!(empty.is_empty());
        assert!(validate_ontology_against_rules(&empty, &[rule("anything")], "test").is_ok());
    }
}

#[cfg(test)]
mod ontology_shipped_packs {
    use super::*;

    /// The shipped packs must load with a coherent ontology. This is the test
    /// that would have caught the drift the ontology exists to fix: every
    /// contract type joined to a rule, every lifecycle total, every entity
    /// typed.
    #[test]
    fn every_shipped_pack_has_a_coherent_ontology() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(|p| p.parent())
            .expect("workspace root")
            .join("packs");
        let registry = PackRegistry::load_from_dir(&root).expect("packs load");

        for name in ["cre", "energy", "credit", "opco"] {
            let pack = registry.pack(name).unwrap_or_else(|| panic!("{name} pack"));
            let ontology = &pack.ontology;
            assert!(
                !ontology.is_empty(),
                "{name} declares no ontology; every shipped pack should say what it models"
            );
            assert!(
                ontology.entities.iter().any(|e| e.family == "asset"),
                "{name} declares no asset"
            );
            assert!(
                ontology.entities.iter().any(|e| e.family == "party"),
                "{name} declares no party — a contract needs someone to be between"
            );
            // Every rule is joined to a type and back; load would have failed
            // otherwise, so this asserts the join is non-trivial.
            // Lowered contract types are one-to-one with rules; elections
            // (options) name no rule and are counted separately.
            let lowered = ontology
                .contracts
                .iter()
                .filter(|c| c.contract_name.is_some())
                .count();
            assert_eq!(
                lowered,
                pack.lowering_rules
                    .iter()
                    .map(|r| r.contract_name.as_str())
                    .collect::<BTreeSet<_>>()
                    .len(),
                "{name}: lowered contract types and lowering rules should be one-to-one"
            );
            assert!(
                ontology.elections().next().is_some(),
                "{name} declares no option type — every pack has at least one election"
            );
        }
    }
}

#[cfg(all(test, feature = "embedded-packs"))]
mod ontology_embedded_parity {
    use super::*;

    /// Embedded packs list their files explicitly, so a new pack file is
    /// invisible to the embedded build until someone remembers to add it —
    /// and the failure is silent, because a missing entrypoint loads as empty
    /// rather than erroring. The ontology was missing exactly this way.
    ///
    /// This pins the two loaders to agree, so the next file added cannot
    /// diverge quietly.
    #[test]
    fn embedded_and_filesystem_packs_carry_the_same_ontology() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(|p| p.parent())
            .expect("workspace root")
            .join("packs");
        let from_disk = PackRegistry::load_from_dir(&root).expect("packs load from disk");
        let embedded = PackRegistry::load_embedded().expect("packs load embedded");

        for name in ["cre", "energy", "credit", "opco"] {
            let disk = from_disk.ontology(name).expect("disk ontology");
            let emb = embedded.ontology(name).expect("embedded ontology");
            assert!(
                !emb.is_empty(),
                "{name}: embedded pack carries no ontology — is it missing from the include list?"
            );
            assert_eq!(disk, emb, "{name}: embedded and on-disk ontologies differ");
        }
    }
}

#[cfg(test)]
mod lifecycle_guard_tests {
    use super::*;

    /// A pack edge may carry a guard — the same machine a model declares,
    /// tailored to the domain (`docs/28` §6.1). Guard-less edges stay
    /// permissions, which every shipped pack edge is.
    #[test]
    fn a_pack_transition_carries_its_guard() {
        let toml = r#"
[[lifecycles]]
lifecycle_id = "t.unit"
initial = "leased"
states = ["leased", "delinquent"]

[[lifecycles.transitions]]
from = "leased"
to = "delinquent"
guard = "series_sum(\"t.rent\", time.t - 1, time.t - 1) < 50"

[[lifecycles.transitions]]
from = "delinquent"
to = "leased"
"#;
        #[derive(serde::Deserialize)]
        struct Doc {
            lifecycles: Vec<OntologyLifecycle>,
        }
        let doc: Doc = toml::from_str(toml).expect("guarded lifecycle parses");
        let lc = &doc.lifecycles[0];
        assert_eq!(lc.transitions.len(), 2);
        assert!(lc.transitions[0]
            .guard
            .as_deref()
            .unwrap()
            .contains("series_sum"));
        assert!(lc.transitions[1].guard.is_none());
    }
}
