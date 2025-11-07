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

### Basic Usage (Joint Responses)
```bash
./council.sh 4 "Review the authentication module for security issues"
```

- Spawns 4 Claude instances in parallel
- Each gets a random constraint from the pool
- Results shown side-by-side for you to synthesize
- Takes 2-5 minutes depending on codebase size

### With Synthesis (Consolidated Recommendations)
```bash
COUNCIL_SYNTHESIZE=1 ./council.sh 4 "Review the authentication module"
```

- Same as above, PLUS a 5th agent that:
  - Reads all council member analyses
  - Identifies common themes and unique insights
  - Resolves conflicting recommendations
  - Creates prioritized action plan with implementation roadmap
  - Adds ~30-120 seconds extra time

## Features

- ✅ **Genuine diversity**: Constraints are structurally orthogonal (20-30% overlap vs 70% baseline)
- ✅ **Parallel execution**: Fast wall-clock time (5-30 min for 4 agents)
- ✅ **Real-time updates**: Shows which members completed as they finish
- ✅ **Automatic cleanup**: Temp files removed after completion
- ✅ **Timeout protection**: 30-minute default (configurable via `COUNCIL_TIMEOUT`)

## Requirements

- Claude Code CLI installed
- Bash 4.0+
- `shuf` command (usually pre-installed)

## How It Works

1. **Assigns constraints**: Randomly selects N unique constraints from the pool
2. **Spawns agents**: Each Claude instance gets a different constraint prompt
3. **Enforces focus**: "REJECTED if generic" threat + labeling requirements
4. **Aggregates**: Combines all analyses into one report

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

## Which Approach to Use?

The council evaluated this question itself and reached different conclusions based on constraints:

**Use Joint Responses (default) when:**
- You understand your codebase context and can prioritize yourself
- You want to see the raw trade-offs between different perspectives
- You want to learn constraint-thinking mental models
- Precision at file:line level matters
- Cost/time optimization is important

**Use Synthesis (COUNCIL_SYNTHESIZE=1) when:**
- You want one clear path forward immediately
- Multiple conflicting recommendations need arbitration
- You prefer lower cognitive load over raw insights
- Action sequencing and dependency analysis would help
- The extra 30-120 seconds and API cost is acceptable

**The council's verdict:**
- `waste_ohno`: "Synthesis is waste - adds unnecessary processing"
- `complexity_knuth`: "Synthesis reduces cognitive load from O(N·K) to O(S)"

The disagreement itself proves the constraint system works! Try both and see what works for your workflow.

## Development History

Built through iterative council self-analysis:
- **Iteration 0**: Generic prompts → 70% overlap
- **Iteration 1**: Persona-only (think like Musk/Einstein) → Still 60-70% overlap
- **Iteration 2**: Hybrid constraint+persona → 20-30% overlap ✅
- **Iteration 3**: Added optional synthesis agent (council debated its value!)

## License

MIT
