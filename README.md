# Council

Multi-agent code analysis with orthogonal constraints.

## Install

```bash
cargo install rust-script
council --install  # installs to ~/.cargo/bin/council
```

## Usage

```bash
council "Review the auth module"              # 5 members, synthesis only
council -n 8 "question"                       # 8 members
council --all "question"                      # show individual analyses
council -m haiku "question"                   # faster model
council --no-synthesize "question"            # skip synthesis
```

## Constraints

Each member analyzes through one lens (Goldratt + Musk always included):

- `the_goal_goldratt`: Global goal & constraints (mandatory)
- `urgency_musk`: 10x improvements & first principles (mandatory)
- `complexity_knuth`: Algorithmic complexity
- `types_czaplicki`: Type safety & API design
- `errors_dijkstra`: Error handling & edge cases
- `simplicity_hickey`: Separation of concerns
- `waste_ohno`: Lean/TPS waste elimination
- `devex_spolsky`: Developer experience
- `tests_beck`: Test coverage
- `taste_torvalds`: Code taste
- `pragmatic_carmack`: Shipping readiness
- `refactor_fowler`: Code smells
- `firstprinciples_feynman`: Fundamental constraints
- `security_bernstein`: Crypto & security
- `perf_blow`: Performance optimization
- `tic80_limit`: Minimal footprint

## Requirements

- `rust-script` (`cargo install rust-script`)
- Claude CLI installed

## License

MIT
