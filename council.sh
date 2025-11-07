#!/bin/bash
# Council - Spawn multiple Claude instances to analyze and provide suggestions
# Based on telebot's monitor pattern
#
# Usage: ./council.sh [num_claudes] "task description"
# Example: ./council.sh 4 "Review the authentication module and suggest improvements"

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
MAGENTA='\033[0;35m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Default number of Claudes
NUM_CLAUDES=${1:-4}
TASK="${2:-}"

# Validate inputs
if [ -z "$TASK" ]; then
    echo -e "${RED}Error: Task description required${NC}"
    echo ""
    echo "Usage: $0 [num_claudes] \"task description\""
    echo ""
    echo "Example:"
    echo "  $0 4 \"Review the authentication module and suggest improvements\""
    echo ""
    exit 1
fi

# Check if NUM_CLAUDES is a valid number
if ! [[ "$NUM_CLAUDES" =~ ^[0-9]+$ ]] || [ "$NUM_CLAUDES" -lt 1 ]; then
    echo -e "${RED}Error: Number of Claudes must be a positive integer${NC}"
    exit 1
fi

# Validate claude CLI is available
if ! command -v claude &> /dev/null; then
    echo -e "${RED}Error: 'claude' CLI not found in PATH${NC}"
    echo ""
    echo "Please install Claude Code first:"
    echo "  https://docs.anthropic.com/claude/docs/claude-code"
    echo ""
    exit 1
fi

# Get current working directory
WORK_DIR=$(pwd)
TEMP_DIR=$(mktemp -d)
OUTPUT_DIR="$TEMP_DIR/outputs"
mkdir -p "$OUTPUT_DIR"

# Kill all child processes
kill_children() {
    # Kill all spawned Claude processes (and their children via process groups)
    for pid in "${PIDS[@]}"; do
        if kill -0 "$pid" 2>/dev/null; then
            # Try graceful termination first, kill process group to catch children
            kill -TERM -$pid 2>/dev/null || kill -TERM $pid 2>/dev/null || true
        fi
    done

    # Give them 3 seconds to cleanup
    sleep 3

    # Force kill any remaining
    for pid in "${PIDS[@]}"; do
        if kill -0 "$pid" 2>/dev/null; then
            # Kill process group first, fallback to individual PID
            kill -KILL -$pid 2>/dev/null || kill -KILL $pid 2>/dev/null || true
        fi
    done
}

# Cleanup function (for abnormal exits: Ctrl+C or termination)
cleanup() {
    local exit_code=$?

    # Kill children on interrupt/termination
    kill_children

    # Clean up temp files if they still exist
    if [ -d "$TEMP_DIR" ]; then
        rm -rf "$TEMP_DIR" 2>/dev/null || true
    fi

    exit $exit_code
}
trap cleanup INT TERM

# Timestamp function
timestamp() {
    date '+%H:%M:%S'
}

# Log functions
info() {
    echo -e "${GREEN}[$(timestamp)]${NC} $1"
}

warn() {
    echo -e "${YELLOW}[$(timestamp)]${NC} $1"
}

error() {
    echo -e "${RED}[$(timestamp)]${NC} $1"
}

# Council member colors for better visual distinction
COLORS=("$CYAN" "$MAGENTA" "$BLUE" "$GREEN" "$YELLOW" "$RED")

get_color() {
    local index=$1
    local color_index=$((index % ${#COLORS[@]}))
    echo "${COLORS[$color_index]}"
}

# Header
echo ""
echo -e "${GREEN}═══════════════════════════════════════════════════════════════${NC}"
echo -e "${GREEN}                    COUNCIL OF CLAUDES${NC}"
echo -e "${GREEN}═══════════════════════════════════════════════════════════════${NC}"
echo ""
echo -e "  ${CYAN}Working Directory:${NC} $WORK_DIR"
echo -e "  ${CYAN}Council Members:${NC} $NUM_CLAUDES"
echo -e "  ${CYAN}Task:${NC} $TASK"
echo ""
echo -e "${GREEN}═══════════════════════════════════════════════════════════════${NC}"
echo ""

# Hybrid Constraint + Persona System
# Constraints ensure orthogonal analysis dimensions
# Personas add flavor and specific questioning styles
declare -A CONSTRAINT_PROMPTS
CONSTRAINT_PROMPTS=(
    [complexity_knuth]="CONSTRAINT: Analyze ONLY algorithmic complexity, time/space efficiency, and data structure choices. Ignore architecture, style, features.

PERSONA: Think like Donald Knuth - mathematical rigor, prove correctness, optimize for algorithmic elegance.

KEY QUESTIONS: What's the time complexity? Can we prove this terminates? Is there a more fundamental algorithm? Are we using optimal data structures?"

    [types_czaplicki]="CONSTRAINT: Analyze ONLY type safety, API design, and preventing impossible states. Ignore implementation details and performance.

PERSONA: Think like Evan Czaplicki (Elm) - make impossible states impossible, design APIs where misuse is a compile error.

KEY QUESTIONS: What runtime failures could types prevent? Where can users misuse this API? How can we encode invariants in types?"

    [errors_dijkstra]="CONSTRAINT: Analyze ONLY error handling, edge cases, failure modes, and correctness. Ignore happy paths and features.

PERSONA: Think like Edsger Dijkstra - correctness by construction, debugging should be unnecessary.

KEY QUESTIONS: What happens when X fails? What invariants must hold? Can we prove this handles all cases? What edge cases are missed?"

    [scale_goldratt]="CONSTRAINT: Analyze ONLY scalability from 1 to 1M users, bottlenecks, and system constraints. Ignore current behavior.

PERSONA: Think like Eliyahu Goldratt (Theory of Constraints) - find the ONE bottleneck limiting throughput.

KEY QUESTIONS: What breaks at 10x load? 100x? Where's the constraint? What optimizations don't address the bottleneck?"

    [simplicity_hickey]="CONSTRAINT: Analyze ONLY complexity, complecting (intertwining), and separation of concerns. Ignore features and performance.

PERSONA: Think like Rich Hickey - Simple (one braid) vs Easy (familiar). Choose simple even when hard.

KEY QUESTIONS: What are we complecting? Can we separate these concerns? Is this genuinely simple or just easy/familiar?"

    [waste_ohno]="CONSTRAINT: Analyze ONLY waste, unnecessary work, and value flow. Ignore features and cleverness.

PERSONA: Think like Taiichi Ohno (Toyota Production System) - eliminate the 7 wastes (waiting, overproduction, defects, over-processing, motion, transport, inventory, unused talent).

KEY QUESTIONS: What's waste here? Where does value flow? What work adds no value? What's inventory hiding problems?"

    [devex_spolsky]="CONSTRAINT: Analyze ONLY developer experience, API usability, error messages, and leaky abstractions. Ignore internals.

PERSONA: Think like Joel Spolsky - abstractions leak, prioritize developer experience, make the common case obvious.

KEY QUESTIONS: Where does this abstraction leak? Is the common case obvious? Are error messages helpful? Can this be misused?"

    [tests_beck]="CONSTRAINT: Analyze ONLY test coverage, missing edge cases, test quality, and testability. Ignore existing code quality.

PERSONA: Think like Kent Beck (TDD) - make it work, make it right, make it fast (in that order). Let design emerge from tests.

KEY QUESTIONS: What's untested? What edge cases are missing? Are tests brittle? Does the design emerge from tests?"

    [taste_torvalds]="CONSTRAINT: Analyze ONLY code taste, unnecessary complexity, and what should be deleted. Ignore features and requirements.

PERSONA: Think like Linus Torvalds - good taste is knowing what to leave out. Bad code is bad regardless of function.

KEY QUESTIONS: Does this have taste? Is this needlessly complex? What should we delete? Would I be embarrassed to show this?"

    [pragmatic_carmack]="CONSTRAINT: Analyze ONLY shipping readiness, premature abstraction, and pragmatic trade-offs. Ignore theoretical perfection.

PERSONA: Think like John Carmack - elegant code that ships. Beware premature abstraction.

KEY QUESTIONS: Will this actually ship? Is this abstraction premature? What's the pragmatic path that doesn't sacrifice quality?"

    [refactor_fowler]="CONSTRAINT: Analyze ONLY code smells, refactoring opportunities, and pattern applications. Ignore new features.

PERSONA: Think like Martin Fowler - name the pattern, know when to apply vs avoid.

KEY QUESTIONS: What's the code smell? Which refactoring applies? What's the simplest transformation? When should we NOT use this pattern?"

    [firstprinciples_feynman]="CONSTRAINT: Analyze ONLY fundamental physics/reality constraints vs arbitrary tradition. Ignore current implementation.

PERSONA: Think like Richard Feynman - break down to fundamentals, explain simply or you don't understand it.

KEY QUESTIONS: What are the actual physical constraints? Can I explain this to a child? What am I pretending to understand? What's physics vs convention?"
)

# Constraint keys for assignment (each guarantees orthogonal analysis)
CONSTRAINT_KEYS=(complexity_knuth types_czaplicki errors_dijkstra scale_goldratt simplicity_hickey waste_ohno devex_spolsky tests_beck taste_torvalds pragmatic_carmack refactor_fowler firstprinciples_feynman)

# Randomly assign unique constraints to council members
assign_constraints() {
    local num_needed=$1
    # Shuffle constraint keys and take first N (ensures no duplicates)
    printf '%s\n' "${CONSTRAINT_KEYS[@]}" | shuf | head -n "$num_needed"
}

# Create the enhanced prompt for each Claude with assigned constraint
create_council_prompt() {
    local member_id=$1
    local constraint_key=$2
    local constraint_framework="${CONSTRAINT_PROMPTS[$constraint_key]}"

    cat <<EOF
You are Council Member #$member_id in a collaborative analysis team of $NUM_CLAUDES Claude instances.

═══════════════════════════════════════════════════════════════
YOUR ANALYSIS CONSTRAINT: ${constraint_key^^}
═══════════════════════════════════════════════════════════════

$constraint_framework

CRITICAL ENFORCEMENT:
1. You MUST focus EXCLUSIVELY on your constraint dimension - this is not optional
2. IGNORE all other aspects not covered by your constraint
3. Label EVERY insight with "[${constraint_key}]:" to maintain focus
4. If your analysis could have come from any other constraint, you're doing it WRONG
5. Other council members cover different dimensions - TRUST them and go DEEP on yours
6. Your analysis will be REJECTED if it's generic advice that ignores your constraint

Quality over quantity: 5 constraint-specific insights > 20 generic observations

═══════════════════════════════════════════════════════════════

YOUR ROLE AND OPERATIONAL CONSTRAINTS:
- You are in READ-ONLY mode: Use ALL analysis tools (Read, Grep, Glob, Bash) but CANNOT use Write, Edit, or NotebookEdit
- Your purpose: Provide detailed, constraint-focused suggestions to a master agent who will implement changes
- Explore the codebase thoroughly within your constraint dimension

YOUR TASK:
$TASK

YOUR OUTPUT REQUIREMENTS:
1. Executive summary (2-3 sentences) from your constraint's perspective ONLY
2. Detailed analysis broken into sections, each labeled with [${constraint_key}]
3. Specific recommendations with:
   - File paths and line numbers (format: file.ext:123)
   - Exact code changes (before/after or patches)
   - Rationale explicitly tied to your constraint dimension
4. Risks, edge cases, trade-offs within your constraint area
5. Brief notes on any out-of-scope critical issues

Remember: All $NUM_CLAUDES members have DIFFERENT, ORTHOGONAL constraints. Your unique value is depth in ONE dimension, not breadth across many.

Begin your constraint-focused analysis now.
EOF
}

# Assign unique constraints to each council member
info "Assigning analysis constraints to council members..."
ASSIGNED_CONSTRAINTS=($(assign_constraints $NUM_CLAUDES))

# Display assignments
for i in $(seq 0 $((NUM_CLAUDES-1))); do
    CONSTRAINT="${ASSIGNED_CONSTRAINTS[$i]}"
    info "  Member #$((i+1)): ${CONSTRAINT^^}"
done
echo ""

# Spawn Claude processes
info "Spawning $NUM_CLAUDES council members with orthogonal constraints..."
echo ""

declare -a PIDS
declare -a OUTPUT_FILES

for i in $(seq 1 $NUM_CLAUDES); do
    OUTPUT_FILE="$OUTPUT_DIR/member_$i.txt"
    OUTPUT_FILES+=("$OUTPUT_FILE")

    CONSTRAINT="${ASSIGNED_CONSTRAINTS[$((i-1))]}"
    PROMPT=$(create_council_prompt $i "$CONSTRAINT")
    COLOR=$(get_color $((i-1)))

    # Spawn Claude in background, capturing output
    # Simplified pipeline to avoid pipe buffer exhaustion and stdout contention
    # Using head to limit output to 100K lines as safety measure
    (
        claude \
            -p "$PROMPT" \
            --model "claude-sonnet-4-5-20250929" \
            --dangerously-skip-permissions \
            --output-format "text" \
            2>&1 | head -n 100000 > "$OUTPUT_FILE"

        # Log completion to stdout without blocking pipes
        echo -e "${COLOR}[Member #$i ($CONSTRAINT) completed]${NC}"
    ) &

    PID=$!
    PIDS+=($PID)

    info "Spawned Member #$i with $CONSTRAINT constraint (PID: $PID)"

    # Small delay to avoid overwhelming the system
    sleep 0.5
done

echo ""
info "All council members spawned. Waiting for analysis to complete..."
echo ""
echo -e "${YELLOW}═══════════════════════════════════════════════════════════════${NC}"
echo -e "${YELLOW}              COUNCIL MEMBERS ANALYZING...${NC}"
echo -e "${YELLOW}═══════════════════════════════════════════════════════════════${NC}"
echo ""

# Monitor completion in real-time
FAILED=0
COMPLETED=0
declare -a MEMBER_STATUS
for i in $(seq 0 $((NUM_CLAUDES-1))); do
    MEMBER_STATUS[$i]="running"
done

START_TIME=$(date +%s)
LAST_STATUS_UPDATE=0
MAX_TIMEOUT=${COUNCIL_TIMEOUT:-1800}  # Default 30 minutes, configurable via env var

while [ $COMPLETED -lt $NUM_CLAUDES ]; do
    # Check each member's status
    for i in $(seq 0 $((NUM_CLAUDES-1))); do
        if [ "${MEMBER_STATUS[$i]}" == "running" ]; then
            PID=${PIDS[$i]}
            MEMBER_NUM=$((i+1))

            if ! kill -0 "$PID" 2>/dev/null; then
                # Process has finished, capture exit code (only called once per process)
                wait $PID 2>/dev/null
                EXIT_CODE=$?

                if [ $EXIT_CODE -eq 0 ]; then
                    info "Member #$MEMBER_NUM completed successfully"
                    MEMBER_STATUS[$i]="completed"
                else
                    error "Member #$MEMBER_NUM failed (exit code: $EXIT_CODE)"
                    MEMBER_STATUS[$i]="failed"
                    FAILED=$((FAILED+1))
                fi
                COMPLETED=$((COMPLETED+1))
            fi
        fi
    done

    # Progress update every 5 seconds if not all complete
    if [ $COMPLETED -lt $NUM_CLAUDES ]; then
        sleep 5

        # Check for timeout
        ELAPSED=$(($(date +%s) - START_TIME))
        if [ $ELAPSED -ge $MAX_TIMEOUT ]; then
            error "Council execution timed out after ${MAX_TIMEOUT}s ($(($MAX_TIMEOUT / 60))m)"
            warn "Killing remaining processes..."

            # Mark remaining running processes as timed out
            for i in $(seq 0 $((NUM_CLAUDES-1))); do
                if [ "${MEMBER_STATUS[$i]}" == "running" ]; then
                    MEMBER_STATUS[$i]="timeout"
                    FAILED=$((FAILED+1))
                    COMPLETED=$((COMPLETED+1))
                fi
            done

            kill_children
            break
        fi

        # Show periodic status update every 30 seconds
        if [ $((ELAPSED - LAST_STATUS_UPDATE)) -ge 30 ]; then
            ACTIVE=$((NUM_CLAUDES - COMPLETED))
            MINUTES=$((ELAPSED / 60))
            SECONDS=$((ELAPSED % 60))
            info "Still analyzing... ($COMPLETED/$NUM_CLAUDES completed, ${MINUTES}m ${SECONDS}s elapsed)"
            LAST_STATUS_UPDATE=$ELAPSED
        fi
    fi
done

echo ""
echo -e "${GREEN}═══════════════════════════════════════════════════════════════${NC}"
echo -e "${GREEN}              COUNCIL ANALYSIS COMPLETE${NC}"
echo -e "${GREEN}═══════════════════════════════════════════════════════════════${NC}"
echo ""

# Summary
if [ $FAILED -eq 0 ]; then
    info "All $NUM_CLAUDES council members completed successfully"
else
    warn "$FAILED out of $NUM_CLAUDES members failed"
fi

echo ""
echo -e "${GREEN}═══════════════════════════════════════════════════════════════${NC}"
echo -e "${GREEN}                  GENERATING AGGREGATE REPORT${NC}"
echo -e "${GREEN}═══════════════════════════════════════════════════════════════${NC}"
echo ""

# Generate and output aggregate report
# If synthesis enabled, skip individual reports and only show synthesis
# If synthesis disabled, show all individual reports
if [ "${COUNCIL_SYNTHESIZE:-0}" -eq 0 ]; then
    echo ""
    echo "═══════════════════════════════════════════════════════════════"
    echo "                    COUNCIL ANALYSIS REPORT"
    echo "═══════════════════════════════════════════════════════════════"
    echo ""
    echo "Task: $TASK"
    echo "Working Directory: $WORK_DIR"
    echo "Council Members: $NUM_CLAUDES"
    echo "Timestamp: $(date '+%Y-%m-%d %H:%M:%S')"
    echo ""
    echo "═══════════════════════════════════════════════════════════════"
    echo ""

    for i in $(seq 1 $NUM_CLAUDES); do
        OUTPUT_FILE="$OUTPUT_DIR/member_$i.txt"
        CONSTRAINT="${ASSIGNED_CONSTRAINTS[$((i-1))]}"
        echo ""
        echo "───────────────────────────────────────────────────────────────"
        echo "        MEMBER #$i ANALYSIS [${CONSTRAINT^^} CONSTRAINT]"
        echo "───────────────────────────────────────────────────────────────"
        echo ""
        if [ -f "$OUTPUT_FILE" ]; then
            cat "$OUTPUT_FILE"
        else
            echo "[No output captured]"
        fi
        echo ""
    done

    echo ""
    echo "═══════════════════════════════════════════════════════════════"
    echo "                       END OF REPORT"
    echo "═══════════════════════════════════════════════════════════════"
    echo ""
else
    # Synthesis mode - skip individual displays, go straight to synthesis
    echo ""
    echo "═══════════════════════════════════════════════════════════════"
    echo "                    COUNCIL SYNTHESIS MODE"
    echo "═══════════════════════════════════════════════════════════════"
    echo ""
    info "Generating consolidated synthesis from $NUM_CLAUDES council members..."
    info "(Individual analyses available in temp directory until completion)"
    echo ""
fi

# Optional synthesis step (enabled via --synthesize flag or COUNCIL_SYNTHESIZE=1)
if [ "${COUNCIL_SYNTHESIZE:-0}" -eq 1 ]; then
    echo ""
    echo "═══════════════════════════════════════════════════════════════"
    echo "                    SYNTHESIS & RECOMMENDATIONS"
    echo "═══════════════════════════════════════════════════════════════"
    echo ""

    info "Synthesizing insights from all $NUM_CLAUDES council members..."
    echo ""

    # Collect all member outputs
    ALL_ANALYSES=""
    for i in $(seq 1 $NUM_CLAUDES); do
        OUTPUT_FILE="$OUTPUT_DIR/member_$i.txt"
        CONSTRAINT="${ASSIGNED_CONSTRAINTS[$((i-1))]}"
        if [ -f "$OUTPUT_FILE" ]; then
            ALL_ANALYSES="$ALL_ANALYSES

═══════════════════════════════════════════════════════════════
MEMBER #$i: ${CONSTRAINT^^}
═══════════════════════════════════════════════════════════════

$(cat "$OUTPUT_FILE")
"
        fi
    done

    # Create synthesis prompt
    SYNTHESIS_PROMPT="You are a master synthesizer analyzing insights from $NUM_CLAUDES council members who each analyzed through different constraints.

YOUR TASK:
Synthesize the following analyses into ONE coherent, actionable recommendation.

ORIGINAL TASK:
$TASK

COUNCIL ANALYSES:
$ALL_ANALYSES

YOUR SYNTHESIS REQUIREMENTS:

1. EXECUTIVE SUMMARY (3-4 sentences)
   - What's the core issue?
   - What's the recommended solution?
   - What's the expected impact?

2. CONSOLIDATED FINDINGS
   - Identify common themes across multiple constraints
   - Highlight unique insights from specific constraints
   - Resolve any conflicting recommendations (explain which to prioritize and why)

3. PRIORITIZED ACTION PLAN
   - List specific changes in priority order (P0/P1/P2)
   - For each item: file:line, what to change, why, expected impact
   - Include concrete code snippets where applicable

4. RISKS & TRADE-OFFS
   - What are we trading off?
   - What could go wrong?
   - How to mitigate?

5. IMPLEMENTATION ROADMAP
   - What order to tackle changes?
   - What dependencies exist?
   - Estimated effort (hours/days)?

Be concise but specific. The goal is ONE clear path forward, not multiple options.
Focus on ACTIONABLE recommendations with clear next steps.

Begin your synthesis now."

    # Run synthesis
    SYNTHESIS_OUTPUT=$(claude \
        -p "$SYNTHESIS_PROMPT" \
        --model "claude-sonnet-4-5-20250929" \
        --dangerously-skip-permissions \
        --output-format "text" \
        2>&1)

    echo "$SYNTHESIS_OUTPUT"
    echo ""
    echo "═══════════════════════════════════════════════════════════════"
    echo "                    END OF SYNTHESIS"
    echo "═══════════════════════════════════════════════════════════════"
    echo ""
fi

# Clean up temp files now that we're done
rm -rf "$TEMP_DIR" 2>/dev/null || true
