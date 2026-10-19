# OptiAI Project Plan

## Product vision

OptiAI is a native desktop decision-modeling assistant for businesses that
need optimization but do not have a dedicated data engineering or operations
research team.

The product turns business language and operational data into a reviewed,
structured optimization model. Deterministic algorithms calculate the result.
AI is used as a translator, implementor, question-asker, and explainer—not as
the numerical decision maker.

The product should make the following workflow approachable:

1. A business owner or operations manager describes a decision problem.
2. The application identifies the required data and missing information.
3. The user imports or enters data.
4. The application proposes an objective, constraints, and assumptions.
5. The user reviews and approves the proposed model.
6. A deterministic optimization engine solves it.
7. The application reports the decision and the tradeoffs in business language.
8. The user runs what-if scenarios before acting.

The result must always mean:

> The best feasible result for the approved objective, constraints, data, and
> assumptions.

It must never be presented as an absolute or universal “best business
decision.”

## Initial beachhead

The first useful domain is bakery or small production-business planning.

### Initial problem

Given products, prices, costs, recipes, demand limits, and available resources,
recommend integer production quantities that maximize contribution profit.

### Initial model family

- Linear programming.
- Mixed-integer linear programming.
- Resource allocation.
- Basic production planning.

### Initial solver

HiGHS, accessed through a Rust adapter. The solver is packaged as part of the
application build so end users do not install it separately.

### Initial result

An explainable production plan that includes:

- Recommended quantities.
- Contribution profit.
- Resource usage.
- Remaining resources.
- Limiting resources.
- Validation warnings.
- A scenario-ready structured result.

## MVP acceptance criteria

The first complete MVP should let a user:

1. Create or select a bakery production-planning workspace.
2. Import or enter products and resources.
3. Define price, cost, demand, recipes, capacity, and units.
4. Describe the objective in plain language or select a guided objective.
5. Review the structured objective, constraints, and assumptions.
6. Validate data before solving.
7. Solve a supported LP or MILP with HiGHS.
8. See the recommended production quantities and objective value.
9. See which resources are limiting.
10. Receive an explanation generated from solver facts.
11. Change an input and compare a scenario.
12. Receive a clear error for invalid, infeasible, unsupported, or incomplete
    models.
13. Run the workflow offline without an external AI service.
14. Reproduce the same result from the same model, data, and solver settings.

## Current implementation slice

The repository currently implements the deterministic foundation of the MVP:

- `crates/domain`: serializable bakery model and domain validation.
- `crates/bakery`: HiGHS-backed integer production model and result facts.
- `apps/cli`: validate and solve commands.
- `fixtures/bakery.json`: a repeatable example plan.
- `Makefile`: offline test, format, lint, build, and demo commands.

This foundation deliberately comes before AI and UI. AI cannot safely translate
or explain a model until the model contract and solver result contract are
stable and testable.

## User workflow

### Guided mode

For a bakery, restaurant, small manufacturer, or local retailer:

- Conversation or guided questions.
- Templates.
- Simple editable tables.
- Explicit assumptions.
- Plain-language explanations.
- No solver terminology unless the user asks for it.

### Analyst mode

For operations managers and analysts:

- Data mapping.
- Constraint editing.
- Scenario comparison.
- Historical data checks.
- Solver diagnostics.
- Exportable reports.

### Engineering and enterprise mode

For factories, refineries, logistics teams, and large operations:

- Database and file connectors.
- Schema inspection.
- Unit and type validation.
- Large dataset previews.
- Model versioning.
- Audit history.
- Batch scenarios.
- Solver settings and diagnostics.

The interface should use progressive disclosure. A simple mode must not hide
the objective, important constraints, assumptions, data-quality warnings, or
solver status. Users must be able to open the advanced view without losing the
current model.

## Technical direction

### Desktop application

Use Rust with a native cross-platform UI. Slint is the planned UI technology
because the application should be Rust-first, installable on Linux and
Windows, and not require a browser deployment.

### Workspace structure

```text
opti-ai/
├── Cargo.toml
├── Cargo.lock
├── rust-toolchain.toml
├── Dockerfile
├── Makefile
├── README.md
├── PROJECT_PLAN.md
├── apps/
│   ├── desktop/
│   └── cli/
├── crates/
│   ├── domain/
│   ├── data/
│   ├── modeling/
│   ├── solver-highs/
│   ├── linear-algebra/
│   ├── ai/
│   ├── explanation/
│   ├── storage/
│   └── execution/
├── fixtures/
└── tests/
```

The current repository has the domain and bakery solver slices. Additional
crates should be introduced only when they have a real responsibility and a
testable boundary.

### Canonical model

The solver-independent model must define:

- Variables.
- Variable types.
- Bounds.
- Objective direction.
- Linear expressions.
- Constraints.
- Units.
- Data provenance.
- User-approved assumptions.
- Model version.

The canonical model is the contract between data ingestion, AI translation,
validation, solver adapters, the UI, and explanations.

### Solver adapter

The application should expose a solver interface similar to:

```rust
trait OptimizationSolver {
    fn capabilities(&self) -> SolverCapabilities;
    fn solve(&self, model: &OptimizationModel) -> Result<SolveResult, SolverError>;
}
```

HiGHS is the first implementation. Other engines may be added later without
forcing the domain model or UI to depend on a particular library.

## Data strategy

Initial formats:

- CSV.
- JSON.
- SQLite.

Later formats and connectors:

- Parquet.
- Excel.
- PostgreSQL.
- SQL Server.
- REST APIs.
- Industrial historian systems.
- Cloud object storage.

Large industrial data must not be copied into a chat prompt. The application
should inspect schemas, profile columns, query summaries, and pass only the
information needed to create a model to the AI layer. Full datasets remain in
controlled local or enterprise storage.

## AI responsibilities and safety boundaries

### AI may

- Extract intent from natural language.
- Identify entities and required fields.
- Suggest data mappings.
- Ask for missing information.
- Suggest objectives and constraints for review.
- Generate a typed draft model.
- Identify ambiguity and contradictions.
- Suggest scenarios.
- Turn structured solver facts into business language.

### AI may not

- Directly calculate the optimal decision.
- Silently invent a value.
- Execute arbitrary generated code.
- Modify business data without approval.
- Bypass model validation.
- Claim unsupported numerical facts.
- Hide infeasibility or uncertainty.
- Access unrestricted files or system commands.

The AI must produce a typed intermediate representation. Allowed operations
should include explicit data actions such as selecting columns, converting
units, filtering rows, joining tables, aggregating values, and deriving a
validated field. The system executes those operations through deterministic
Rust code.

### Required review gates

1. AI interpretation review.
2. Data-quality review.
3. Objective and constraint review.
4. Canonical model validation.
5. Solver execution.
6. Result and scenario review.
7. Human approval before export or operational use.

### Prompt injection protection

Imported data is untrusted. Product names, notes, database fields, and files
must not be treated as instructions. The AI layer must:

- Keep system instructions separate from business data.
- Use structured output validation.
- Restrict available tools.
- Prevent arbitrary file access.
- Require approval for data export.
- Log generated actions.
- Reject unknown actions and malformed model structures.

## Explanation architecture

Explanations must be generated from structured facts rather than from a raw
solver log or an unconstrained model response.

The result contract should include:

- Solver status.
- Objective value.
- Decision variable values.
- Constraint utilization.
- Remaining resource quantities.
- Binding constraints.
- Reduced costs or dual values where the adapter supports them.
- Scenario differences.
- Data-quality warnings.
- Model assumptions.
- Source references.

The first release includes a deterministic explanation fallback so the product
is useful even without a language model. A future language model should only
rewrite these facts for the selected audience.

Numerical precision must not be confused with business confidence. Every
report should distinguish:

- User-provided facts.
- Imported facts.
- AI suggestions.
- Default assumptions.
- Solver-derived facts.
- Human-approved decisions.

## Data quality and uncertainty

The application must check:

- Units.
- Data types.
- Missing values.
- Duplicate rows.
- Outliers.
- Date ranges.
- Stale records.
- Referential integrity.
- Currency.
- Time periods.
- Impossible values.
- Category changes.

Results should include warnings such as:

> Demand data is 14 months old. The recommendation may not reflect current
> demand.

The next scenario layer should support:

- Expected case.
- Best case.
- Worst case.
- Demand ranges.
- Cost ranges.
- Supplier delays.
- Capacity disruptions.
- Sensitivity analysis.

The application should show whether a recommendation is stable across tested
scenarios or changes materially when an assumption changes.

## Supported and unsupported optimization

The product must publish a capability matrix instead of implying that HiGHS
solves every business problem.

### Initial supported models

- LP.
- MILP.
- Production planning.
- Resource allocation.
- Assignment.
- Transportation.
- Basic scheduling.
- Inventory planning.

### Later model families

- Nonlinear programming.
- Stochastic optimization.
- Robust optimization.
- Constraint programming.
- Process simulation.
- Global optimization.
- Real-time optimization.

Unsupported requests should receive a specific explanation rather than an
incorrect approximation:

> This problem contains nonlinear relationships that are outside the current
> model capabilities.

## Security, privacy, and governance

The default product posture is local-first:

- Business data remains on the device by default.
- Solving works without a network connection.
- AI is optional for the deterministic core.
- Remote model providers require explicit user configuration.
- Remote execution is a separate, visible execution mode.

The application should eventually provide:

- Encrypted local workspace storage.
- Workspace access controls.
- Audit logs.
- Model and scenario versioning.
- Data retention controls.
- Export controls.
- Clear model and provider identification.
- Reproducible solver settings.

## Packaging and open-source dependencies

End users should not need to install Rust, Cargo, CMake, a C++ compiler, a
database server, or HiGHS separately.

Release artifacts should package the application and its runtime dependencies.
Initial targets are Windows x64 and Linux x64.

The build must maintain:

- `Cargo.lock`.
- `rust-toolchain.toml`.
- License inventory.
- Security advisory checks.
- Third-party notices.
- Versioned solver and model metadata.
- Reproducible build instructions.

The project must define the open-source boundary explicitly. The core model
specification, solver adapters, data transformation engine, templates, and
documentation should not depend on a proprietary service to function.

## Adversarial review and mitigations

### “Best” is underspecified

**Risk:** The user asks for a universally best decision while caring about
multiple unspoken goals.

**Mitigation:** Require explicit objectives, secondary objectives, hard
constraints, soft constraints, planning horizon, and risk tolerance. Use
“best under the approved model” in all reports.

### Natural language can produce the wrong model

**Risk:** A mathematically valid model may represent the wrong business intent.

**Mitigation:** Show the AI interpretation, list ambiguity, require approval,
and make every objective and constraint editable before solving.

### Precise results can hide uncertain data

**Risk:** Exact quantities create false confidence when demand or costs are
uncertain.

**Mitigation:** Add data-quality scores, freshness warnings, scenario analysis,
sensitivity analysis, and stable-versus-sensitive recommendation labels.

### A solver can optimize bad data

**Risk:** Valid syntax does not mean valid operations data.

**Mitigation:** Validate units, dates, duplicates, missing values, ranges,
referential integrity, and source freshness before model construction.

### HiGHS cannot solve every industrial problem

**Risk:** Refinery and industrial users need nonlinear, stochastic, simulation,
or real-time models.

**Mitigation:** Publish capabilities, reject unsupported model types, and keep a
solver adapter boundary for future engines.

### AI explanations can be persuasive but false

**Risk:** The explanation claims the wrong bottleneck or invents a number.

**Mitigation:** Generate explanations from typed result facts and attach source
references. Test numerical consistency.

### Imported business text can contain prompt injection

**Risk:** A file or database field attempts to manipulate the AI.

**Mitigation:** Treat all imported text as untrusted data, use structured
outputs, restrict tools, validate actions, and require export approval.

### Simplification can hide risk

**Risk:** A friendly interface hides assumptions or makes users over-trust the
answer.

**Mitigation:** Use progressive disclosure. Always show objective, assumptions,
key constraints, warnings, solver status, and a path to advanced details.

### Desktop-only execution will not scale to refinery data

**Risk:** Large models and datasets exceed a workstation’s memory or runtime.

**Mitigation:** Keep the desktop model contract independent from execution.
Support optional remote execution later while preserving local operation for
small and private workloads.

### Local AI creates packaging and hardware problems

**Risk:** A bundled model is large and may not run on small computers.

**Mitigation:** Support lightweight and full local models, explicit hardware
capabilities, private hosted providers, and a deterministic non-AI fallback.

### The product becomes too broad

**Risk:** Data engineering, AI, optimization, reporting, and many industries
are attempted at once.

**Mitigation:** Make bakery production planning excellent first. Add future
industries as tested domain packs with their own schemas, constraints,
explanations, and fixtures.

### Correct mathematics may still fail to create business value

**Risk:** Users do not trust, adopt, or maintain the recommendation.

**Mitigation:** Measure time to valid model, recommendation acceptance, waste
reduction, stockout reduction, profit impact, and errors caught—not AI usage or
number of generated messages.

### Open-source dependencies introduce legal and maintenance risk

**Risk:** A bundled solver, native library, or model has incompatible terms or
security issues.

**Mitigation:** Track SPDX licenses, advisories, sources, notices, model terms,
and release versions. Run dependency checks in CI.

## Implementation phases

### Phase 1: Deterministic optimization core

Current phase:

- Canonical bakery data types.
- Domain validation.
- HiGHS adapter.
- Integer production variables.
- Demand bounds.
- Resource constraints.
- Structured solve results.
- Deterministic explanation fallback.
- CLI and JSON fixture.

Exit criteria:

- Valid models solve reproducibly.
- Invalid models fail with actionable errors.
- Objective and resource usage match the solver result.
- Tests pass offline.

### Phase 2: Storage and data entry

- SQLite workspace.
- CSV import.
- Editable product and resource tables.
- Unit checks.
- Data-quality report.
- Model and scenario versioning.

Exit criteria:

- Imported data can be read back without loss.
- Every model field has a source or explicit assumption.
- Invalid data is visible before solving.

### Phase 3: Native desktop workflow

- Create workspace.
- Choose bakery template.
- Enter or import data.
- Review model.
- Solve.
- View recommendations.
- View bottlenecks.
- Compare scenarios.

Exit criteria:

- A nontechnical user can complete the full flow without a terminal.
- The UI never presents a result without solver status and assumptions.

### Phase 4: AI translation

- Natural-language intake.
- Missing-field questions.
- Data mapping suggestions.
- Typed model draft generation.
- Ambiguity detection.
- Human approval gates.

Exit criteria:

- AI output is schema-validated.
- AI cannot execute arbitrary code.
- The same approved model solves identically with or without AI.

### Phase 5: Evidence-backed explanation

- Solver-backed business reports.
- Scenario comparisons.
- Sensitivity analysis.
- Data freshness warnings.
- Report export.

Exit criteria:

- Every numerical claim maps to a structured result fact.
- Reports distinguish facts, assumptions, and suggestions.

### Phase 6: Enterprise execution

- Large file and database connectors.
- Query-based data access.
- Optional remote execution.
- Permissions.
- Audit logs.
- Batch scenarios.
- More advanced model families.

Exit criteria:

- Large datasets are not sent wholesale to the AI.
- Desktop and remote execution use the same canonical model contract.

## Testing strategy

### Mathematical tests

- Known bakery fixtures with expected objective values.
- Expected decision quantities.
- Demand-bound tests.
- Resource-bound tests.
- Invalid model tests.
- Infeasible model tests.
- Unsupported-model tests.
- Constraint satisfaction post-checks.

### Translation tests

- Ambiguous requests.
- Missing data.
- Contradictory requirements.
- Incorrect units.
- Unsupported nonlinear requests.
- Prompt injection in imported data.
- Malformed structured output.

### Explanation tests

- Numerical consistency.
- Correct bottleneck identification.
- Correct scenario differences.
- No unsupported claims.
- Clear uncertainty warnings.

### Packaging tests

- Clean Linux installation.
- Clean Windows installation.
- No Rust or compiler requirement for end users.
- Offline startup.
- Offline solving.
- Version and solver metadata.
- Upgrade and workspace migration.

## Success metrics

The project should measure outcomes rather than model novelty:

- Time from first input to valid optimization model.
- Percentage of models needing manual correction.
- Percentage of recommendations accepted.
- Time saved compared with spreadsheet planning.
- Waste reduction.
- Stockout reduction.
- Contribution-profit improvement.
- Number of data issues detected before decisions.
- Number of infeasible plans prevented.
- User understanding of the recommendation.

## Non-goals for the first release

- Supporting every industry.
- Replacing domain experts.
- Autonomous execution of operational actions.
- Making legal, safety, medical, or financial compliance claims.
- Treating a chatbot response as an optimization result.
- Sending full industrial datasets into a language-model context.
- Adding nonlinear refinery optimization before the supported model contract is
  stable.

## Open product decisions

Before expanding beyond the current solver slice, decide:

1. The first desktop UI toolkit and accessibility requirements.
2. Whether the first AI model is bundled, downloaded, or provider-configured.
3. The initial CSV schema and unit policy.
4. The minimum scenario comparison experience.
5. The first Windows and Linux packaging formats.
6. Whether the first release includes SQLite persistence or keeps the CLI
   fixture-only.
