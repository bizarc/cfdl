// Generator for penzance_deck_2026-09-13.pptx. Rebuild with:
//   npm install pptxgenjs && node deck_gen.js "$PWD/penzance_deck_2026-09-13.pptx"
// Figures are from benchmarks/cre/penzance_highlands and penzance_one_rosslyn as of #340.
const pptxgen = require("pptxgenjs");
const pres = new pptxgen();
pres.layout = "LAYOUT_WIDE"; // 13.33 x 7.5
pres.title = "CFDL for Penzance";

// palette: the private case pages
const INK = "191C1F", MUTED = "5C6570", RULE = "E2DFDA", SURF = "F2F0EC";
const ACC = "1F4E5F", ACCS = "E6EDEF", GOOD = "3F6B4F", WHITE = "FFFFFF", DARK = "14171A", DARKS = "1A1E22", ICE = "79ADBD";
const H = "Cambria", B = "Calibri", M = "Courier New";
const W = 13.33;

function base(dark = false) {
  const s = pres.addSlide();
  s.background = { color: dark ? DARK : WHITE };
  return s;
}
function eyebrow(s, text, y = 0.55, dark = false) {
  s.addText(text.toUpperCase(), { x: 0.7, y, w: 8, h: 0.3, fontFace: B, fontSize: 11, color: dark ? ICE : MUTED, charSpacing: 2, isTextBox: true, margin: 0 });
}
function title(s, text, y = 0.85, dark = false, size = 30) {
  s.addText(text, { x: 0.7, y, w: W - 1.4, h: 1.05, fontFace: H, fontSize: size, color: dark ? "E8E6E2" : INK, isTextBox: true, margin: 0, valign: "top" });
}
function body(s, text, x, y, w, h, opts = {}) {
  s.addText(text, Object.assign({ x, y, w, h, fontFace: B, fontSize: 15, color: INK, isTextBox: true, margin: 0, valign: "top", paraSpaceAfter: 6 }, opts));
}
function foot(s, n, dark = false) {
  s.addText("CFDL · Penzance · September 2026", { x: 0.7, y: 7.0, w: 6, h: 0.3, fontFace: B, fontSize: 9, color: dark ? "969EA6" : MUTED, isTextBox: true, margin: 0 });
  s.addText(String(n), { x: W - 1.4, y: 7.0, w: 0.7, h: 0.3, fontFace: B, fontSize: 9, color: dark ? "969EA6" : MUTED, align: "right", isTextBox: true, margin: 0 });
}
function stat(s, x, y, w, value, label, dark = false) {
  s.addShape(pres.ShapeType.rect, { x, y, w, h: 1.7, fill: { color: dark ? DARKS : SURF }, line: { color: dark ? DARKS : SURF } });
  s.addText(value, { x: x + 0.2, y: y + 0.15, w: w - 0.4, h: 0.8, fontFace: H, fontSize: 30, color: dark ? "E8E6E2" : ACC, isTextBox: true, margin: 0, valign: "middle" });
  s.addText(label, { x: x + 0.2, y: y + 0.95, w: w - 0.4, h: 0.7, fontFace: B, fontSize: 11, color: dark ? "969EA6" : MUTED, isTextBox: true, margin: 0, valign: "top" });
}
function table(s, rows, x, y, w, colW, opts = {}) {
  const fs = opts.fontSize || 12;
  const data = rows.map((r, i) => r.map((c, j) => ({
    text: c,
    options: {
      fontFace: B, fontSize: fs, color: i === 0 ? MUTED : INK, bold: i === 0 || (opts.boldFirstCol && j === 0),
      align: j === 0 ? "left" : "right", valign: "middle",
      fill: { color: i === 0 ? WHITE : (i % 2 === 0 ? WHITE : "FAF9F7") },
      border: [{ type: "none" }, { type: "none" }, { type: "solid", pt: 0.5, color: RULE }, { type: "none" }],
      margin: [4, 6, 4, 6],
    },
  })));
  s.addTable(data, { x, y, w, colW, rowH: opts.rowH || 0.34 });
}
function code(s, text, x, y, w, h) {
  s.addShape(pres.ShapeType.rect, { x, y, w, h, fill: { color: DARKS }, line: { color: DARKS } });
  s.addText(text, { x: x + 0.25, y: y + 0.2, w: w - 0.5, h: h - 0.4, fontFace: M, fontSize: 11.5, color: "E8E6E2", isTextBox: true, margin: 0, valign: "top", lineSpacingMultiple: 1.15 });
}
function note(s, text, x, y, w, h) {
  s.addShape(pres.ShapeType.rect, { x, y, w, h, fill: { color: ACCS }, line: { color: ACCS } });
  s.addText(text, { x: x + 0.25, y: y + 0.15, w: w - 0.5, h: h - 0.3, fontFace: H, fontSize: 14, italic: true, color: ACC, isTextBox: true, margin: 0, valign: "middle" });
}

let n = 0;

// 1 — title
{
  const s = base(true); n++;
  eyebrow(s, "Prepared for Penzance", 2.2, true);
  s.addText("Two buildings, one language", { x: 0.7, y: 2.55, w: 11.5, h: 1.2, fontFace: H, fontSize: 48, color: "E8E6E2", isTextBox: true, margin: 0 });
  s.addText("The Highlands, reconstructed from the public record. One Rosslyn, before it exists. Both written in CFDL, both checked against an independent model, period by period.", { x: 0.7, y: 3.85, w: 9.5, h: 1.2, fontFace: B, fontSize: 18, color: "969EA6", isTextBox: true, margin: 0 });
  s.addText("September 2026", { x: 0.7, y: 6.4, w: 5, h: 0.4, fontFace: B, fontSize: 12, color: ICE, isTextBox: true, margin: 0 });
  s.addNotes("Open on the thesis, not on the tool. The two buildings are the reason we are in the room.");
}

// 2 — the problem
{
  const s = base(); n++;
  eyebrow(s, "The problem");
  title(s, "Every deal model is a spreadsheet nobody else can check");
  const items = [
    ["Rebuilt every time", "Each deal starts from a copied workbook. Nothing carries forward except habit."],
    ["Assumptions hide in cells", "A rent growth rate lives in a formula three sheets deep. The memo says one number; the model runs another."],
    ["No two models agree", "The same building, modeled by two analysts, produces two answers. Neither can be reconciled to the other."],
    ["Nothing is verified", "The model is checked by reading it. A wrong sign in a construction draw survives until someone notices the equity."],
  ];
  items.forEach((it, i) => {
    const y = 2.1 + i * 1.15;
    s.addShape(pres.ShapeType.ellipse, { x: 0.7, y: y + 0.05, w: 0.5, h: 0.5, fill: { color: ACCS }, line: { color: ACCS } });
    s.addText(String(i + 1), { x: 0.7, y: y + 0.05, w: 0.5, h: 0.5, fontFace: H, fontSize: 16, color: ACC, align: "center", valign: "middle", isTextBox: true, margin: 0 });
    s.addText(it[0], { x: 1.4, y, w: 5, h: 0.4, fontFace: B, fontSize: 16, bold: true, color: INK, isTextBox: true, margin: 0 });
    s.addText(it[1], { x: 1.4, y: y + 0.4, w: 6.2, h: 0.7, fontFace: B, fontSize: 13, color: MUTED, isTextBox: true, margin: 0 });
  });
  note(s, "A model should be a document: readable, diffable, and checkable against something other than itself.", 8.2, 2.3, 4.4, 2.0);
  foot(s, n);
}

// 3 — what CFDL is
{
  const s = base(); n++;
  eyebrow(s, "What CFDL is");
  title(s, "The model is the document");
  body(s, "CFDL is a language for cash-flowing assets. A model is a page of text that keeps time, structure and behavior apart. The compiler turns it into a deterministic record; the engine runs it. Same inputs, same version: byte-identical output.", 0.7, 2.0, 5.6, 1.7, { fontSize: 15 });
  body(s, "Every published case is diffed, period by period, against an independent reference: a workbook, a public exhibit, a textbook answer. The numbers are verified, not asserted.", 0.7, 3.7, 5.6, 1.3, { fontSize: 15 });
  stat(s, 0.7, 5.15, 1.75, "40", "published cases");
  stat(s, 2.6, 5.15, 1.75, "304", "golden fixtures");
  stat(s, 4.5, 5.15, 1.8, "4", "domain packs");
  code(s, `entity asset facility : Asset.Financial {
  equity_funded init min(inputs.equity_commitment,
                         curve_value("dev_cost_cum", time.date))
  interest      init 0.0
                next prev.asset.facility.balance
                     * inputs.loan_rate / 12.0
  balance       init max(0.0, curve_value("dev_cost", time.date)
                            - inputs.equity_commitment)
                next ...
}

stream cre.loan_interest on entity container.project outflow {
  schedule every month start from 2018-04 to 2024-12
  category financing.debt.interest_paid
  amount = asset.facility.interest
}`, 6.8, 2.0, 5.85, 4.7);
  s.addText("A construction facility, from The Highlands model: equity first, then draws, with interest capitalized. Every month resolves from the one before it.", { x: 6.8, y: 6.72, w: 5.85, h: 0.4, fontFace: B, fontSize: 9.5, color: MUTED, isTextBox: true, margin: 0 });
  foot(s, n);
}

// 4 — what it expresses
{
  const s = base(); n++;
  eyebrow(s, "What it can express");
  title(s, "The mechanics of a real estate deal, as language");
  const cells = [
    ["Leases, one by one", "Free rent, escalations, expense stops, TI and LC, probability-weighted rollover, downtime. Every period inside a cent of the reference."],
    ["Construction funding", "Equity first, then the facility. Capitalized interest ties to the dollar on One Lincoln Street."],
    ["Debt", "Construction facilities retired from proceeds, permanent takeouts sized off value, balloons booked as reversion so coverage stays honest."],
    ["The split", "Capital accounts per partner, tiered waterfalls, preferred returns that compound, promotes, and a return measured on each partner's own cash."],
    ["Scenarios", "One model, many runs. A strategy switch, a market factor, a downside, each a named run with its own asserted answer."],
    ["Uncertainty", "Distributions on any input. A thousand trials, and the P5 to P95 band, not one blended number that no world produces."],
  ];
  cells.forEach((c, i) => {
    const col = i % 3, row = Math.floor(i / 3);
    const x = 0.7 + col * 4.05, y = 2.05 + row * 2.35;
    s.addShape(pres.ShapeType.rect, { x, y, w: 3.85, h: 2.15, fill: { color: SURF }, line: { color: SURF } });
    s.addText(c[0], { x: x + 0.25, y: y + 0.2, w: 3.35, h: 0.4, fontFace: H, fontSize: 17, color: ACC, isTextBox: true, margin: 0 });
    s.addText(c[1], { x: x + 0.25, y: y + 0.65, w: 3.35, h: 1.4, fontFace: B, fontSize: 12.5, color: INK, isTextBox: true, margin: 0, valign: "top" });
  });
  foot(s, n);
}

// 5 — Highlands: the record
{
  const s = base(); n++;
  eyebrow(s, "The Highlands · SP #445");
  title(s, "A completed deal, rebuilt from the public record only");
  body(s, "Two towers over one podium, for-sale and rental product on a single construction basis, seven of twelve parcels ground-leased from the County, and in-kind public obligations that are pure cost. Land bought September 2011, a 39-month build, and a three-part exit.", 0.7, 2.0, 5.6, 1.6, { fontSize: 14 });
  table(s, [
    ["", "Units", "Tenure", "Exit"],
    ["Pierce", "104", "for sale", "102 recorded closings"],
    ["Evo", "455", "rental", "$334,642,240"],
    ["Aubrey", "331", "rental", "$266,455,000"],
  ], 0.7, 3.7, 5.6, [1.2, 0.8, 1.2, 2.4]);
  s.addText("Sourced, from Arlington County", { x: 6.8, y: 2.0, w: 5.8, h: 0.35, fontFace: B, fontSize: 13, bold: true, color: GOOD, isTextBox: true, margin: 0 });
  body(s, "Program, unit mix, GFA, parking and the quantified obligations from the site plan record. The $67,000,000 land basis from the deed. All 102 condominium closings from the assessment roll. Both tower sale prices. The 5.15% guideline cap, $7,511 per unit expenses and 8% vacancy from the Commercial Guidebook. Rents solved back from each tower's assessed value at the County's own cap.", 6.8, 2.4, 5.8, 1.9, { fontSize: 12.5 });
  s.addText("Assumed, and labeled as such", { x: 6.8, y: 4.4, w: 5.8, h: 0.35, fontFace: B, fontSize: 13, bold: true, color: "8A5A1E", isTextBox: true, margin: 0 });
  body(s, "Construction cost, debt pricing, lease-up pace, and the partnership terms. Each is a stated placeholder in the model, and each is the kind of number only the people who built the deal can replace.", 6.8, 4.8, 5.8, 1.2, { fontSize: 12.5 });
  note(s, "Everything sourced says where it came from. Everything assumed says that it is assumed.", 0.7, 5.5, 5.6, 1.1);
  foot(s, n);
}

// 6 — Highlands: the result
{
  const s = base(); n++;
  eyebrow(s, "The Highlands · the result");
  title(s, "The facility ties to the independent workbook exactly");
  stat(s, 0.7, 2.05, 2.9, "$370.4M", "peak construction debt");
  stat(s, 3.8, 2.05, 2.9, "$48.4M", "capitalized interest, to the cent");
  stat(s, 6.9, 2.05, 2.9, "11.02%", "levered return over the hold");
  stat(s, 10.0, 2.05, 2.63, "2.05x", "multiple on invested capital");
  body(s, "The engine and an independent spreadsheet read the same frozen inputs and tie by construction, not by transcription. The return agrees to the sixth decimal. Every month is asserted, not only the lifetime totals, because placing cash at the close of a period instead of the open moves the return from 11.02% to 11.15% while every total stays the same.", 0.7, 3.95, 7.4, 1.7, { fontSize: 14 });
  body(s, "Both towers sold in lease-up in May 2022, a year after delivery. The model carries the ramp rather than a stabilized year, because that is what happened.", 0.7, 5.6, 7.4, 1.0, { fontSize: 14 });
  s.addShape(pres.ShapeType.rect, { x: 8.6, y: 3.95, w: 4.0, h: 2.7, fill: { color: SURF }, line: { color: SURF } });
  s.addText("Why no NPV", { x: 8.85, y: 4.1, w: 3.5, h: 0.35, fontFace: H, fontSize: 16, color: ACC, isTextBox: true, margin: 0 });
  s.addText("The deal is financed, so any present value is levered and needs a cost of equity. No source states one. Across nine rates the NPV swings from +$84M to −$46M while the return and the multiple do not move, because they are solved from the cash.", { x: 8.85, y: 4.5, w: 3.5, h: 2.0, fontFace: B, fontSize: 12, color: INK, isTextBox: true, margin: 0, valign: "top" });
  foot(s, n);
}

// 7 — Highlands: the split
{
  const s = base(); n++;
  eyebrow(s, "The Highlands · the split");
  title(s, "What each partner receives, on placeholder terms");
  body(s, "Contributions move each partner's capital account on the dates the facility draws equity. Cash accrues to the venture and is split once, at the last condominium closing in June 2024, through seven tiers. The distribution allocates exactly what the venture holds and leaves nothing behind.", 0.7, 2.0, 5.9, 1.5, { fontSize: 13.5 });
  table(s, [
    ["", "The Baupost Group", "Penzance"],
    ["Share of capital", "90%", "10%"],
    ["Contributed", "167,620,753", "18,624,528"],
    ["Distributed", "328,472,612", "54,134,181"],
    ["Share of profit", "81.9%", "18.1%"],
    ["Multiple", "1.96x", "2.91x"],
    ["Return", "8.12%", "12.76%"],
  ], 0.7, 3.65, 5.9, [2.3, 1.9, 1.7], { boldFirstCol: false });
  s.addText("The tiers", { x: 7.1, y: 2.0, w: 5.5, h: 0.35, fontFace: H, fontSize: 16, color: ACC, isTextBox: true, margin: 0 });
  table(s, [
    ["Tier", "Paid to", "Amount"],
    ["Capital", "both, pro rata", "186,245,281"],
    ["8% preference", "both, pro rata", "108,175,391"],
    ["20% promote", "Penzance", "17,637,224"],
    ["Residual", "90 / 10", "70,548,897"],
  ], 7.1, 2.45, 5.5, [1.7, 1.8, 2.0]);
  note(s, "The parties are real. The 8% preference, 90/10 split and 20% promote are not; they stand in for terms in no public source. Replacing three rates recomputes every figure on this slide.", 7.1, 4.6, 5.5, 1.6);
  foot(s, n);
}

// 8 — One Rosslyn: fact and forecast
{
  const s = base(); n++;
  eyebrow(s, "One Rosslyn · SP #419");
  title(s, "The inverse case: the program is fact, the economics are forecast");
  body(s, "Three towers over one podium, 845 units, entitled July 2025 and unbuilt. Land recorded November 2023 at $52,000,000, which is 13.6% below the County's own guideline basis for the rental units. The affordable-housing obligations are timed to each tower's certificate of occupancy, so their timing is entitlement fact.", 0.7, 2.0, 5.9, 1.8, { fontSize: 13.5 });
  table(s, [
    ["", "Units", "Stories", "Tenure"],
    ["NE Tower", "73", "23", "for sale"],
    ["NW Tower", "311", "27", "rental"],
    ["S Tower", "461", "30", "rental"],
  ], 0.7, 3.95, 5.9, [1.7, 1.2, 1.3, 1.7]);
  stat(s, 7.1, 2.0, 2.65, "$3,789", "rent per unit per month, derived by the County's method");
  stat(s, 9.95, 2.0, 2.68, "$255/sf", "per sf of building, anchored to Highlands' actual spend");
  s.addText("How the rent is derived", { x: 7.1, y: 3.9, w: 5.5, h: 0.35, fontFace: H, fontSize: 16, color: ACC, isTextBox: true, margin: 0 });
  body(s, "A comparable's assessed value times the guideline loaded cap gives the assessor's own net operating income. Add back guideline expenses, gross up for vacancy. Nothing in it is an opinion about the market.", 7.1, 4.3, 5.5, 1.2, { fontSize: 13 });
  note(s, "Read the two cases together. One shows what a model does when the facts constrain it. The other shows what it does when they do not.", 0.7, 5.55, 5.9, 1.1);
  foot(s, n);
}

// 9 — One Rosslyn: two strategies
{
  const s = base(); n++;
  eyebrow(s, "One Rosslyn · the strategies");
  title(s, "The holding period is the deal");
  body(s, "One model carries two exits over one set of facts. The hold is the base case: stabilize, refinance into permanent debt at 60% of value, hold five years. The sale is the comparison, kept because it is what the same partners chose four blocks away. The engine derives each exit from the twelve months of income after it, over the County's cap.", 0.7, 2.0, 5.5, 1.9, { fontSize: 13.5 });
  table(s, [
    ["", "A · sell in lease-up", "B · hold five years"],
    ["Exit", "2031-10", "2037-06"],
    ["Construction cost", "555,953,894", "555,953,894"],
    ["Capitalized interest", "58,451,635", "70,146,419"],
    ["Permanent loan", "—", "347,217,832"],
    ["Exit value", "567,404,298", "691,127,415"],
    ["Exit per unit", "734,980", "895,243"],
    ["Net to equity", "28,591,606", "223,799,127"],
    ["Equity multiple", "1.09x", "1.74x"],
    ["Return", "2.02%", "6.57%"],
  ], 6.6, 2.0, 6.0, [2.2, 1.9, 1.9], { rowH: 0.36 });
  note(s, "Land, building, cost and basis are identical in both columns. Scenario A's derived exit of $734,980 per unit sits within 0.1% of what Evo actually sold for.", 0.7, 4.1, 5.5, 1.4);
  body(s, "Both scenarios tie to the independent workbook to under a dollar.", 0.7, 5.7, 5.5, 0.5, { fontSize: 12, color: MUTED });
  foot(s, n);
}

// 10 — One Rosslyn: the scenario set (chart)
{
  const s = base(); n++;
  eyebrow(s, "One Rosslyn · the scenario set");
  title(s, "Six named runs, bracketed by Rosslyn's own history");
  body(s, "The market factor scales the exit. 1.00 is the County's guideline basis. The two observations available bracket it: The Highlands sold at +32% to this basis in 2022, Central Place at −11.6% in 2026. Each strategy runs at all three, and each run pins its own answer.", 0.7, 2.0, 5.2, 1.7, { fontSize: 13.5 });
  s.addChart(pres.ChartType.bar, [
    { name: "Equity multiple", labels: ["Sell · 2026 discount", "Sell · guideline", "Sell · 2022 premium", "Hold · 2026 discount", "Hold · guideline", "Hold · 2022 premium"], values: [0.877, 1.094, 1.695, 1.474, 1.738, 2.470] },
  ], {
    x: 6.3, y: 1.9, w: 6.4, h: 4.6, barDir: "bar",
    chartColors: [ICE, ICE, ICE, ACC, ACC, ACC],
    showValue: true, dataLabelPosition: "outEnd", dataLabelFormatCode: "0.00\"x\"", dataLabelFontSize: 11, dataLabelColor: INK, dataLabelFontFace: B,
    catAxisLabelColor: INK, catAxisLabelFontSize: 11, catAxisLabelFontFace: B, valAxisLabelColor: MUTED, valAxisLabelFontSize: 10, valAxisLabelFontFace: B,
    valGridLine: { color: RULE, size: 0.5 }, catGridLine: { style: "none" }, valAxisMinVal: 0, valAxisMaxVal: 3,
    showLegend: false, showTitle: true, title: "Equity multiple on the deal's cash, by run", titleColor: MUTED, titleFontSize: 12, titleFontFace: B,
  });
  table(s, [
    ["Run", "Market factor", "Return"],
    ["Hold, guideline (base)", "1.000", "6.57%"],
    ["Hold, 2026 discount", "0.884", "4.70%"],
    ["Hold, 2022 premium", "1.321", "10.44%"],
    ["Sell, guideline", "1.000", "2.02%"],
    ["Sell, 2026 discount", "0.884", "−2.90%"],
    ["Sell, 2022 premium", "1.321", "12.00%"],
  ], 0.7, 3.85, 5.2, [2.6, 1.3, 1.3], { fontSize: 11.5, rowH: 0.32 });
  foot(s, n);
}

// 11 — One Rosslyn: the split
{
  const s = base(); n++;
  eyebrow(s, "One Rosslyn · the split");
  title(s, "At the guideline basis, neither strategy clears the preference");
  body(s, "The same seven tiers as The Highlands, split once per strategy. Capital and the preference come back pro rata, so below the promote the partners are pari passu. The hold returns 6.57% on the deal's cash against an 8% preference: the profit is paid out as preference and stops short of it. The promote is zero under both strategies, and both partners earn the same return.", 0.7, 2.0, 5.7, 2.0, { fontSize: 13.5 });
  table(s, [
    ["", "Sell", "Hold"],
    ["Distributed", "278,260,488", "473,468,010"],
    ["Preference owed", "115,348,264", "297,775,808"],
    ["Preference paid", "28,591,606", "223,799,127"],
    ["Promote", "0", "0"],
    ["Multiple, both partners", "1.11x", "1.90x"],
    ["Return, both partners", "2.00%", "6.22%"],
  ], 6.9, 2.0, 5.7, [2.5, 1.6, 1.6]);
  s.addText("Where the promote starts", { x: 6.9, y: 4.65, w: 5.7, h: 0.35, fontFace: H, fontSize: 16, color: ACC, isTextBox: true, margin: 0 });
  table(s, [
    ["Priced where The Highlands traded", "Promote", "Baupost", "Penzance"],
    ["Hold", "29,575,044", "9.64%", "13.43%"],
    ["Sell", "19,076,024", "10.74%", "17.78%"],
  ], 6.9, 5.05, 5.7, [2.5, 1.2, 1.0, 1.0], { fontSize: 11.5 });
  note(s, "The promote is not an assumption about the deal. It is what the structure pays once the deal clears the preference, and the record says whether it does.", 0.7, 4.3, 5.7, 1.4);
  foot(s, n);
}

// 12 — what we know and what only you know
{
  const s = base(); n++;
  eyebrow(s, "What is sourced, and what is not");
  title(s, "The record fixes half the answer. You hold the other half.");
  const cols = [
    ["In the record", GOOD, ["Program, unit mix, GFA, parking", "Land basis, from the deed", "Every condominium closing", "Both tower sale prices", "County cap, expenses, vacancy", "Public obligations and their timing", "Four recorded Arlington trades"]],
    ["Derived, by a stated method", ACC, ["Rent per unit, from assessed value at the County's cap", "Construction cost, anchored to Highlands and escalated by BLS", "Exit value, from forward income over the cap", "The market factor's bracket, from Rosslyn's own trades"]],
    ["Assumed, and labeled", "8A5A1E", ["Construction cost and duration", "Debt pricing and leverage", "Lease-up pace", "Condominium pricing", "The partnership terms", "The hold length"]],
  ];
  cols.forEach((c, i) => {
    const x = 0.7 + i * 4.05;
    s.addText(c[0], { x, y: 2.05, w: 3.85, h: 0.4, fontFace: H, fontSize: 17, color: c[1], isTextBox: true, margin: 0 });
    s.addText(c[2].map((t, k) => ({ text: t, options: { bullet: true, breakLine: k < c[2].length - 1 } })), { x, y: 2.55, w: 3.85, h: 3.2, fontFace: B, fontSize: 13, color: INK, isTextBox: true, margin: 0, valign: "top", paraSpaceAfter: 6 });
  });
  note(s, "Every figure in the third column is one you can replace. When you do, both cases recompute, and the checks that tie them to the record still have to pass.", 0.7, 5.75, 11.9, 0.95);
  foot(s, n);
}

// 13 — how it reaches the desk
{
  const s = base(); n++;
  eyebrow(s, "How it reaches your desk");
  title(s, "The model is the record. Excel stays the view.");
  const rows = [
    ["Playground", "In the browser, no install. Edit a model, run it, read the statement: revenue to NOI to DSCR to sale proceeds, monthly to annual, exported to CSV."],
    ["Python and API", "Results as data frames for the analyst; a server for saved runs and for the systems that consume them."],
    ["Editor", "VS Code with diagnostics: a misspelled account or an undeclared input is named at the line, before anything runs."],
    ["The Academy", "learn.cfdl.dev: 28 chapters and 23 runnable exercises, ending in a ground-up development with a construction facility, a permanent takeout, a promote waterfall and a downside scenario."],
    ["Migration and audit", "An existing workbook, rebuilt in CFDL and reconciled to it period by period. The discrepancies are the deliverable."],
  ];
  rows.forEach((r, i) => {
    const y = 2.05 + i * 0.92;
    s.addText(r[0], { x: 0.7, y, w: 2.6, h: 0.8, fontFace: H, fontSize: 16, color: ACC, isTextBox: true, margin: 0, valign: "top" });
    s.addText(r[1], { x: 3.4, y, w: 9.2, h: 0.8, fontFace: B, fontSize: 13, color: INK, isTextBox: true, margin: 0, valign: "top" });
    if (i < rows.length - 1) s.addShape(pres.ShapeType.line, { x: 0.7, y: y + 0.84, w: 11.9, h: 0, line: { color: RULE, width: 0.5 } });
  });
  foot(s, n);
}

// 14 — what we are asking
{
  const s = base(true); n++;
  eyebrow(s, "Next", 0.55, true);
  title(s, "What we would like from Penzance", 0.85, true);
  const asks = [
    ["Review both cases", "Both are marked pending practitioner review. Your corrections to cost, pace and debt pricing turn placeholders into a record."],
    ["The terms, under NDA", "So the split is yours rather than a stand-in, and the return per partner means what it says."],
    ["A third building", "An operating asset of your choosing, so the lease and rollover machinery runs on your rent roll rather than a textbook's."],
    ["A pilot", "One analyst, one deal, one quarter. Start with the migration of a model you already trust, and reconcile the two."],
  ];
  asks.forEach((a, i) => {
    const col = i % 2, row = Math.floor(i / 2);
    const x = 0.7 + col * 6.1, y = 2.2 + row * 2.2;
    s.addShape(pres.ShapeType.rect, { x, y, w: 5.85, h: 2.0, fill: { color: DARKS }, line: { color: DARKS } });
    s.addText(a[0], { x: x + 0.3, y: y + 0.25, w: 5.25, h: 0.45, fontFace: H, fontSize: 20, color: "E8E6E2", isTextBox: true, margin: 0 });
    s.addText(a[1], { x: x + 0.3, y: y + 0.8, w: 5.25, h: 1.1, fontFace: B, fontSize: 13.5, color: "969EA6", isTextBox: true, margin: 0, valign: "top" });
  });
  s.addText("You have the record. We have the machine that checks it. Pick the building.", { x: 0.7, y: 6.55, w: 11.9, h: 0.4, fontFace: H, fontSize: 16, italic: true, color: ICE, isTextBox: true, margin: 0 });
  foot(s, n, true);
}

const out = process.argv[2];
pres.writeFile({ fileName: out }).then(() => console.log("wrote", out, n, "slides"));
