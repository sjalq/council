# Council - Multi-Agent Code Analysis

Spawn multiple Claude instances with **orthogonal analysis constraints** to get diverse perspectives on your code.

## The Problem

Single-agent analysis produces generic results. Multiple agents with the same prompt produce 70% overlapping insights.

## The Solution

**Constraint-based diversity**: Each council member analyzes through ONE specific lens:
- `complexity_knuth`: Algorithmic complexity & data structures
- `types_czaplicki`: Type safety & API design
- `errors_dijkstra`: Error handling & edge cases
- `scale_goldratt`: Scalability & bottlenecks
- `simplicity_hickey`: Complexity & separation of concerns
- `waste_ohno`: Waste & value flow (Lean/TPS)
- `devex_spolsky`: Developer experience & leaky abstractions
- `tests_beck`: Test coverage & testability
- `taste_torvalds`: Code taste & unnecessary complexity
- `pragmatic_carmack`: Shipping readiness & pragmatic trade-offs
- `refactor_fowler`: Code smells & refactoring patterns
- `firstprinciples_feynman`: Fundamental constraints vs tradition

## Usage

### Default Behavior (Smart + Synthesis)
```bash
./council.sh 4 "Review the authentication module for security issues"
```

- **Auto-selects** 3 most relevant constraints via Haiku (2-3s overhead)
- **Adds 1 random** constraint for serendipity (catch blind spots)
- **Synthesis only**: Shows consolidated recommendations (not individual analyses)
- **Total time**: 2-5 minutes depending on codebase size

### See All Individual Analyses
```bash
./council.sh --all 4 "Review the authentication module"
```

- Same auto-selection as default
- **Shows all 4 individual analyses** before synthesis
- Use when you want to see reasoning behind recommendations
- Useful for learning constraint-thinking patterns

### Fully Random Selection
```bash
./council.sh --random 4 "Review the authentication module"
```

- **All 4 constraints** picked randomly (old behavior)
- Use when exploring unfamiliar code (maximize serendipity)
- No Haiku pre-pass overhead

### All Flags Combined
```bash
./council.sh --random --all 4 "Review auth"
```

- Fully random selection + show all individual analyses
- Maximum transparency, maximum serendipity

## Features

- ✅ **Smart selection**: Auto-picks relevant constraints, adds 1 random for serendipity
- ✅ **Synthesis by default**: Get one clear path forward, not competing opinions
- ✅ **Genuine diversity**: Constraints are structurally orthogonal (20-30% overlap vs 70% baseline)
- ✅ **Parallel execution**: Fast wall-clock time (2-5 min for 4 agents)
- ✅ **Real-time updates**: Shows which members completed as they finish
- ✅ **Automatic cleanup**: Temp files removed after completion
- ✅ **Timeout protection**: 30-minute default (configurable via `COUNCIL_TIMEOUT`)

## Requirements

- Claude Code CLI installed
- Bash 4.0+
- `shuf` command (usually pre-installed)

## How It Works

1. **Auto-selects constraints**: Haiku picks (N-1) relevant constraints (2-3s), adds 1 random for serendipity
2. **Spawns agents**: Each Claude instance gets a different constraint prompt
3. **Enforces focus**: "REJECTED if generic" threat + labeling requirements
4. **Synthesizes**: Final agent consolidates insights into one actionable plan

## Example Output

```
Member #1: TASTE_TORVALDS
"Delete 60% of this code - it's verbose theater"

Member #2: WASTE_OHNO
"75% overproduction waste - spawning 4 instances to read the same file"

Member #3: COMPLEXITY_KNUTH
"This algorithm is O(n²) where O(n log n) exists"

Member #4: ERRORS_DIJKSTRA
"What happens when the API returns 429? No retry logic found"
```

## When to Use What?

The council evaluated these options and gave guidance:

### Default (Auto + Synthesis) - Best for shipping velocity
**Use when:**
- Reviewing code before committing
- Making architectural decisions
- Refactoring complex modules
- You want one clear path forward fast

**Why it works:**
- Auto-selection ensures relevant perspectives (no wasted slots)
- 1 random constraint catches blind spots
- Synthesis eliminates decision paralysis
- **Council's verdict**: 3-5 min saved per session vs full random

### --all flag - Best for learning
**Use when:**
- You want to understand different thinking patterns
- Learning how constraints lead to insights
- Need to see reasoning behind synthesis
- Teaching others constraint-based analysis

### --random flag - Best for exploration
**Use when:**
- Exploring unfamiliar codebase
- You don't know what you don't know
- Maximum serendipity desired
- Avoiding confirmation bias

**Example**: Task "Review auth" gets `waste_ohno` randomly → discovers "You're hashing passwords 3 times (overproduction)". Auto-selection would never pick this, yet it's the most valuable insight.

### --random --all - Maximum transparency
**Use when:**
- Debugging the council itself
- Research/academic use
- Want to see everything raw
- Teaching how the tool works

## Development History

Built through iterative council self-analysis:
- **Iteration 0**: Generic prompts → 70% overlap
- **Iteration 1**: Persona-only (think like Musk/Einstein) → Still 60-70% overlap
- **Iteration 2**: Hybrid constraint+persona → 20-30% overlap ✅
- **Iteration 3**: Added optional synthesis agent (council debated its value!)
- **Iteration 4**: Made synthesis default + added auto-selection (N-1 auto + 1 random)
  - Council evaluation: 3-2 split on defaults, unanimous that auto-selection should exist
  - Hybrid approach wins: relevance + serendipity
  - CLI params only (no env vars)

## License

MIT
