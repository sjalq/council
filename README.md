# Council

Multi-agent code analysis with orthogonal constraints. Spawn multiple Claude instances, each analyzing your codebase through a different expert lens (Goldratt, Musk, Knuth, Dijkstra, etc.), then synthesize their insights into actionable recommendations.

## Quick Start

Install with one command:

```bash
# Clone and install globally
git clone https://github.com/yourusername/council.git
cd council
./council.rs --install
```

That's it! Now you can use `council` from anywhere.

## Requirements

- **Rust Script**: `cargo install rust-script`
- **Claude CLI**: Install from https://docs.anthropic.com/claude/docs/claude-code

## Usage

### Basic Examples

```bash
# Analyze with 5 members (default), show synthesis only
council "Review the auth module for security issues"

# Use 8 members for more diverse perspectives
council -n 8 "How can we optimize the database queries?"

# Show all individual analyses + synthesis
council --all "Evaluate the error handling strategy"

# Use faster model (haiku) for quicker feedback
council -m haiku "Quick review of the API endpoint"

# Skip synthesis, see only individual analyses
council --no-synthesize "Check the performance bottlenecks"
```

### CLI Options

```
Usage: council [OPTIONS] <TASK>

Arguments:
  <TASK>  Task description for the council to analyze

Options:
  -n, --num <NUM>          Number of council members [default: 5]
  -t, --timeout <TIMEOUT>  Timeout per member in seconds [default: 600]
  -m, --model <MODEL>      Model to use (sonnet, opus, haiku)
      --no-synthesize      Skip synthesis phase (synthesis runs by default)
      --all                Show all individual analyses (default: synthesis only)
      --install            Install council globally
  -h, --help               Print help
```

### Example Output

When you run `council "Review the auth module"`, you'll see:

```
============================================================
                 COUNCIL OF CLAUDES
============================================================

  Members: 5
  Timeout: 600s per member
  Synthesize: yes
  Task: Review the auth module

  Member #1: THE_GOAL_GOLDRATT (mandatory)
  Member #2: URGENCY_MUSK (mandatory)
  Member #3: COMPLEXITY_KNUTH
  Member #4: TYPES_CZAPLICKI
  Member #5: ERRORS_DIJKSTRA

============================================================

[Spawning] Member #1: THE_GOAL_GOLDRATT
[Spawning] Member #2: URGENCY_MUSK
[Spawning] Member #3: COMPLEXITY_KNUTH
[Spawning] Member #4: TYPES_CZAPLICKI
[Spawning] Member #5: ERRORS_DIJKSTRA
[Completed] Member #2: URGENCY_MUSK
[Completed] Member #1: THE_GOAL_GOLDRATT
[Completed] Member #3: COMPLEXITY_KNUTH
[Completed] Member #5: ERRORS_DIJKSTRA
[Completed] Member #4: TYPES_CZAPLICKI

============================================================
     ALL 5 MEMBERS COMPLETED (45.3s)
============================================================

============================================================
              RUNNING SYNTHESIS...
============================================================

============================================================
           SYNTHESIS & RECOMMENDATIONS
============================================================

[Synthesized recommendations appear here with:
 - Executive summary
 - Consolidated findings
 - Prioritized action plan (P0/P1/P2)
 - Risks & trade-offs
 - Implementation roadmap]

============================================================
        TOTAL TIME: 52.1s (members: 45.3s, synthesis: 6.8s)
============================================================
```

## Available Constraints (Expert Lenses)

Each council member analyzes through one specialized lens. Two are always included:

### Mandatory Constraints

- **the_goal_goldratt**: Global goal & constraint identification (Theory of Constraints)
- **urgency_musk**: 10x improvements, deletion opportunities, first principles

### Optional Constraints (randomly selected)

- **complexity_knuth**: Algorithmic complexity & data structures
- **types_czaplicki**: Type safety & impossible states
- **errors_dijkstra**: Correctness, formal verification, invariants
- **simplicity_hickey**: Complexity vs complecting, separation of concerns
- **waste_ohno**: Lean/TPS waste elimination
- **devex_spolsky**: Developer experience, leaky abstractions
- **tests_beck**: Test coverage & TDD
- **taste_torvalds**: Code taste & what to delete
- **pragmatic_carmack**: Shipping readiness & state management
- **refactor_fowler**: Code smells & refactoring patterns
- **firstprinciples_feynman**: Fundamental physics constraints
- **delete_muratori**: Compression-oriented programming, delete abstractions
- **crash_armstrong**: Let it crash philosophy, supervision trees
- **data_acton**: Memory layout, cache behavior, data-oriented design

## How It Works

1. Council spawns N concurrent Claude instances (default: 5)
2. Each instance analyzes your task through a different constraint lens
3. All analyses complete in parallel
4. A synthesis phase consolidates insights into one actionable recommendation
5. Output includes specific file:line recommendations with priority levels

## Tips

- Use `-n 8` or more for complex architectural decisions
- Use `--all` when you want to see individual expert perspectives
- Use `-m haiku` for quick feedback on smaller changes
- Use `--no-synthesize` when you want raw expert opinions without consolidation
- Increase `--timeout` for larger codebases (default: 10 minutes)

## License

MIT
