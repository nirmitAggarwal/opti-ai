# OptiAI

OptiAI is a Rust-first decision modeling assistant. Its product rule is simple:

> AI translates and explains; deterministic optimization engines calculate.

The first vertical slice is an offline bakery production planner. It validates a
typed business model, solves a mixed-integer linear program with HiGHS, and
generates an evidence-backed business explanation.

## Current MVP slice

The current implementation provides:

- A solver-independent bakery domain model.
- Validation for identifiers, recipes, units, and referenced resources.
- Integer production decisions bounded by demand.
- HiGHS-backed LP/MILP solving through `good_lp`.
- Deterministic resource usage and bottleneck reporting.
- A deterministic explanation fallback; no AI or network is required.
- A CLI that solves the built-in example or a JSON plan.
- Offline unit tests and a reproducible Rust toolchain pin.

The desktop UI, natural-language model translator, data importers, scenario
editor, and local model integration are intentionally not in this first code
slice. They will consume the same typed domain and solver contracts.

## Requirements

For local development:

- Rust 1.85.1, selected by `rust-toolchain.toml`.
- Cargo.
- A C++ toolchain, CMake, Clang, and libclang for the bundled HiGHS build.

The repository includes a `Dockerfile` with the required system packages.

## Run the example

```bash
cargo run --offline -p opti-cli -- demo
```

Solve the checked-in JSON fixture:

```bash
cargo run --offline -p opti-cli -- solve fixtures/bakery.json
```

Validate without solving:

```bash
cargo run --offline -p opti-cli -- validate fixtures/bakery.json
```

The example recommends 100 croissants and 8 cakes. Flour is the limiting
resource and the estimated contribution profit is `$250.00`.

## Checks

The one-command offline test entry point is:

```bash
make test
```

Individual checks are also available:

```bash
make fmt-check
make clippy
make test
make build
```

The first build of HiGHS can take several minutes because its native solver
library is compiled locally. Subsequent builds use Cargo's target cache.

## Docker

Build and test in a clean Linux environment:

```bash
docker build -t opti-ai .
docker run --rm opti-ai
```

## Project layout

```text
crates/domain  Solver-independent business model and validation
crates/bakery  Bakery MILP model, HiGHS adapter, and report facts
apps/cli       Command-line user flow
fixtures       Versioned example input data
```

See [`PROJECT_PLAN.md`](PROJECT_PLAN.md) for the product scope, adversarial
review, safety boundaries, and the next implementation phases.
