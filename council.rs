#!/usr/bin/env rust-script
//! ```cargo
//! [dependencies]
//! tokio = { version = "1", features = ["full"] }
//! clap = { version = "4", features = ["derive"] }
//! rand = "0.8"
//! colored = "2"
//! ```

use clap::Parser;
use colored::*;
use rand::seq::SliceRandom;
use std::time::Duration;
use tokio::process::Command;
use tokio::sync::mpsc;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

const MAX_OUTPUT_BYTES: usize = 500_000; // 500KB per member

#[derive(Parser, Debug)]
#[command(name = "council")]
#[command(about = "Spawn multiple Claude instances to analyze with orthogonal constraints")]
struct Args {
    /// Task description for the council to analyze
    task: Option<String>,

    /// Number of council members (default: 5)
    #[arg(short = 'n', long, default_value_t = 5)]
    num: usize,

    /// Timeout per member in seconds (default: 600)
    #[arg(short, long, default_value_t = 600)]
    timeout: u64,

    /// Model to use (sonnet, opus, haiku)
    #[arg(short, long)]
    model: Option<String>,

    /// Skip synthesis phase (synthesis runs by default)
    #[arg(long)]
    no_synthesize: bool,

    /// Show all individual analyses (default: synthesis only)
    #[arg(long)]
    all: bool,

    /// Install council globally to ~/.cargo/bin
    #[arg(long)]
    install: bool,
}

fn install_globally() -> Result<(), Box<dyn std::error::Error>> {
    // Get path to this script
    let exe_path = std::env::current_exe()?;
    let script_dir = exe_path.parent().ok_or("Cannot find script directory")?;

    // Find the actual .rs source file
    let script_path = if exe_path.extension().map(|e| e == "rs").unwrap_or(false) {
        exe_path.clone()
    } else {
        // When run via rust-script, find the original .rs file
        let possible_paths = [
            script_dir.join("council.rs"),
            std::path::PathBuf::from("council.rs"),
            std::path::PathBuf::from("scripts/rust/council.rs"),
        ];
        possible_paths.into_iter()
            .find(|p| p.exists())
            .ok_or("Cannot find council.rs source file")?
    };

    // Install to ~/.cargo/bin (where cargo install puts things)
    let home = std::env::var("HOME").or_else(|_| std::env::var("USERPROFILE"))?;
    let cargo_bin = std::path::PathBuf::from(home).join(".cargo").join("bin");

    // Create ~/.cargo/bin if it doesn't exist
    std::fs::create_dir_all(&cargo_bin)?;

    let install_path = cargo_bin.join("council");

    println!("{}", "Installing council...".green());
    println!("  Source: {}", script_path.display());
    println!("  Target: {}", install_path.display());

    // Copy the file
    std::fs::copy(&script_path, &install_path)?;

    // Make it executable (Unix only)
    #[cfg(unix)]
    {
        std::fs::set_permissions(&install_path, std::fs::Permissions::from_mode(0o755))?;
    }

    println!();
    println!("{}", "Successfully installed!".green().bold());
    println!();
    println!("Run: {} \"your task\"", "council".cyan());
    println!();
    println!("{}", "Note:".yellow());
    println!("  Make sure ~/.cargo/bin is in your PATH");
    println!("  If 'council' is not found, add this to your ~/.bashrc or ~/.zshrc:");
    println!("    export PATH=\"$HOME/.cargo/bin:$PATH\"");

    Ok(())
}

struct Constraint {
    name: &'static str,
    prompt: &'static str,
    mandatory: bool,
}

const CONSTRAINTS: [Constraint; 16] = [
    Constraint {
        name: "the_goal_goldratt",
        prompt: r#"CONSTRAINT: Analyze ONLY by first identifying the GLOBAL GOAL (the ultimate output/outcome the system exists to produce), then THE constraint limiting it, and how to exploit/elevate that constraint. Ignore non-constraints.

PERSONA: Think like Eliyahu Goldratt (Theory of Constraints) - First ask: What is the GLOBAL GOAL? (not local optimization, but the whole system's purpose). Then find the ONE constraint limiting throughput toward that goal. Any improvement not at the constraint is an illusion. Five Focusing Steps: 1) Identify 2) Exploit 3) Subordinate 4) Elevate 5) Repeat.

KEY QUESTIONS: What is the GLOBAL GOAL this system exists to achieve? What's the ONE constraint preventing more of that global output? Are we optimizing locally while ignoring global throughput? How do we exploit the constraint? What should we subordinate to it?"#,
        mandatory: true,
    },
    Constraint {
        name: "urgency_musk",
        prompt: r#"CONSTRAINT: Analyze ONLY what enables 10x faster iteration, deletion opportunities, and shipping urgency. Ignore perfection and process.

PERSONA: Think like Elon Musk - first principles physics, delete ruthlessly, ship with urgency, iterate fast.

KEY QUESTIONS: What can we delete entirely? What's the fastest path to shipping? Are we solving the right problem or optimizing the wrong thing? What would 10x this?"#,
        mandatory: true,
    },
    Constraint {
        name: "complexity_knuth",
        prompt: r#"CONSTRAINT: Analyze ONLY algorithmic complexity, data structure choices, and when optimization matters. Ignore architecture and style.

PERSONA: Think like Donald Knuth - "Premature optimization is the root of all evil." Focus on the critical 3% where performance matters, not the 97% that doesn't. Prove correctness first.

KEY QUESTIONS: What's the actual time/space complexity? Is this in the critical 3% that matters? Are we optimizing prematurely? What's the simplest correct algorithm first?"#,
        mandatory: false,
    },
    Constraint {
        name: "types_czaplicki",
        prompt: r#"CONSTRAINT: Analyze ONLY type safety, API design, and preventing impossible states. Ignore implementation details and performance.

PERSONA: Think like Evan Czaplicki (Elm) - make impossible states impossible, design APIs where misuse is a compile error.

KEY QUESTIONS: What runtime failures could types prevent? Where can users misuse this API? How can we encode invariants in types?"#,
        mandatory: false,
    },
    Constraint {
        name: "errors_dijkstra",
        prompt: r#"CONSTRAINT: Analyze ONLY correctness, formal verification, error handling, and invariants. Ignore performance and features.

PERSONA: Think like Edsger Dijkstra - correctness by construction, not debugging into correctness. "Program testing can show the presence of bugs, but never their absence." Prove it correct.

KEY QUESTIONS: What invariants must hold? Can we prove this is correct? What happens when X fails? How do we know this terminates? What can we eliminate to simplify proof?"#,
        mandatory: false,
    },
    Constraint {
        name: "simplicity_hickey",
        prompt: r#"CONSTRAINT: Analyze ONLY complexity, complecting (intertwining), and separation of concerns. Ignore features and performance.

PERSONA: Think like Rich Hickey - Simple (one braid) vs Easy (familiar). Choose simple even when hard.

KEY QUESTIONS: What are we complecting? Can we separate these concerns? Is this genuinely simple or just easy/familiar?"#,
        mandatory: false,
    },
    Constraint {
        name: "waste_ohno",
        prompt: r#"CONSTRAINT: Analyze ONLY waste, unnecessary work, and value flow. Ignore features and cleverness.

PERSONA: Think like Taiichi Ohno (Toyota Production System) - eliminate the 7 wastes (waiting, overproduction, defects, over-processing, motion, transport, inventory, unused talent).

KEY QUESTIONS: What's waste here? Where does value flow? What work adds no value? What's inventory hiding problems?"#,
        mandatory: false,
    },
    Constraint {
        name: "devex_spolsky",
        prompt: r#"CONSTRAINT: Analyze ONLY developer experience, API usability, error messages, and leaky abstractions. Ignore internals.

PERSONA: Think like Joel Spolsky - abstractions leak, prioritize developer experience, make the common case obvious.

KEY QUESTIONS: Where does this abstraction leak? Is the common case obvious? Are error messages helpful? Can this be misused?"#,
        mandatory: false,
    },
    Constraint {
        name: "tests_beck",
        prompt: r#"CONSTRAINT: Analyze ONLY test coverage, missing edge cases, test quality, and testability. Ignore existing code quality.

PERSONA: Think like Kent Beck (TDD) - make it work, make it right, make it fast (in that order). Let design emerge from tests.

KEY QUESTIONS: What's untested? What edge cases are missing? Are tests brittle? Does the design emerge from tests?"#,
        mandatory: false,
    },
    Constraint {
        name: "taste_torvalds",
        prompt: r#"CONSTRAINT: Analyze ONLY code taste, unnecessary complexity, and what should be deleted. Ignore features and requirements.

PERSONA: Think like Linus Torvalds - good taste is knowing what to leave out. Bad code is bad regardless of function.

KEY QUESTIONS: Does this have taste? Is this needlessly complex? What should we delete? Would I be embarrassed to show this?"#,
        mandatory: false,
    },
    Constraint {
        name: "pragmatic_carmack",
        prompt: r#"CONSTRAINT: Analyze ONLY shipping readiness, state management, and pragmatic functional approaches. Ignore theoretical purity.

PERSONA: Think like John Carmack - move toward functional purity to reduce state bugs, but ship pragmatically. "The real enemy is unexpected mutation of state." Pure functions are easier to reason about.

KEY QUESTIONS: Will this actually ship? What state is being mutated unexpectedly? Can we make this function purer without killing performance? Is this abstraction premature or does it reduce state complexity?"#,
        mandatory: false,
    },
    Constraint {
        name: "refactor_fowler",
        prompt: r#"CONSTRAINT: Analyze ONLY code smells, refactoring opportunities, and pattern applications. Ignore new features.

PERSONA: Think like Martin Fowler - name the pattern, know when to apply vs avoid.

KEY QUESTIONS: What's the code smell? Which refactoring applies? What's the simplest transformation? When should we NOT use this pattern?"#,
        mandatory: false,
    },
    Constraint {
        name: "firstprinciples_feynman",
        prompt: r#"CONSTRAINT: Analyze ONLY fundamental physics/reality constraints vs arbitrary tradition. Ignore current implementation.

PERSONA: Think like Richard Feynman - break down to fundamentals, explain simply or you don't understand it.

KEY QUESTIONS: What are the actual physical constraints? Can I explain this to a child? What am I pretending to understand? What's physics vs convention?"#,
        mandatory: false,
    },
    Constraint {
        name: "delete_muratori",
        prompt: r#"CONSTRAINT: Analyze ONLY by identifying what to DELETE entirely - abstractions, layers, dependencies, code. Ignore features and additions.

PERSONA: Think like Casey Muratori (Handmade Hero) - most abstractions are HARMFUL. Compression-oriented programming: understand the problem domain so well you can delete the framework. The best code is NO code. Performance IS correctness.

KEY QUESTIONS: What abstraction can we delete entirely? What dependency can we remove? What layer is pure overhead? What would this look like with ZERO frameworks? Can we replace 10,000 lines of library with 100 lines that do exactly what we need? How many CPU cycles from input to output?"#,
        mandatory: false,
    },
    Constraint {
        name: "crash_armstrong",
        prompt: r#"CONSTRAINT: Analyze ONLY isolation, supervision trees, and embracing failure. Ignore prevention and defensive programming.

PERSONA: Think like Joe Armstrong (Erlang) - Let it crash. Build supervision, not defenses. Isolation > error handling. Most error handling code is waste—just restart the process. Immutability + message passing = simpler systems.

KEY QUESTIONS: What should we let crash instead of handling? Where's our supervision hierarchy? Can we isolate this so failure doesn't propagate? Are we writing defensive code that should be restart logic? What happens if we DELETE all the try-catch blocks? Can we make this stateless so crashes don't matter?"#,
        mandatory: false,
    },
    Constraint {
        name: "data_acton",
        prompt: r#"CONSTRAINT: Analyze ONLY memory layout, cache behavior, and data transformation pipelines. Ignore object models and abstractions.

PERSONA: Think like Mike Acton (Insomniac Games) - OOP is an expensive disaster. Structure code around memory access patterns, not abstractions. Data is all there is. The purpose of all programs is to transform data from one form to another.

KEY QUESTIONS: What's the cache miss rate? Are we storing arrays of structs or structs of arrays? Does this data layout match CPU reality? Can we delete the object model entirely? Where does the data come from, where does it go, and what transformations happen? How much memory are we wasting on indirection?"#,
        mandatory: false,
    },
];

fn select_constraints(n: usize) -> Vec<&'static Constraint> {
    let mut rng = rand::thread_rng();

    // Always include ALL mandatory constraints
    let mandatory: Vec<_> = CONSTRAINTS.iter().filter(|c| c.mandatory).collect();
    let others: Vec<_> = CONSTRAINTS.iter().filter(|c| !c.mandatory).collect();

    // Always include all mandatory, even if n is smaller
    let mut selected = mandatory;

    if n > selected.len() {
        let remaining = n - selected.len();
        let mut shuffled: Vec<_> = others.into_iter().collect();
        shuffled.shuffle(&mut rng);
        selected.extend(shuffled.into_iter().take(remaining));
    }

    selected
}

fn create_prompt(constraint: &Constraint, task: &str, num_members: usize) -> String {
    format!(
        r#"You are a council member analyzing with a specific constraint. There are {} council members, each with different orthogonal constraints.

{}

YOUR TASK:
{}

YOUR OUTPUT REQUIREMENTS:
1. Executive summary (2-3 sentences) from your constraint's perspective ONLY
2. Detailed analysis with specific insights labeled [{}]
3. Recommendations with file paths and line numbers where applicable
4. Risks and trade-offs within your constraint area

Quality over quantity - 5 constraint-specific insights > 20 generic observations.
If your analysis could come from any other constraint, you're doing it WRONG."#,
        num_members, constraint.prompt, task, constraint.name
    )
}

fn create_synthesis_prompt(outputs: &[(usize, String, String)], task: &str) -> String {
    let analyses: String = outputs
        .iter()
        .map(|(id, name, text)| {
            format!(
                "═══════════════════════════════════════════════════════════════\nMEMBER #{}: {}\n═══════════════════════════════════════════════════════════════\n\n{}",
                id + 1,
                name.to_uppercase(),
                text
            )
        })
        .collect::<Vec<_>>()
        .join("\n\n");

    format!(
        r#"You are a master synthesizer analyzing insights from {} council members who each analyzed through different constraints.

YOUR TASK:
Synthesize the following analyses into ONE coherent, actionable recommendation.

ORIGINAL TASK:
{}

COUNCIL ANALYSES:
{}

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

Be concise but specific. The goal is ONE clear path forward, not multiple options.
Focus on ACTIONABLE recommendations with clear next steps."#,
        outputs.len(),
        task,
        analyses
    )
}

async fn run_claude(
    prompt: &str,
    timeout_secs: u64,
    model: Option<&str>,
) -> Result<String, String> {
    let result = tokio::time::timeout(Duration::from_secs(timeout_secs), async {
        let mut cmd = Command::new("claude");
        cmd.args(["-p", prompt, "--output-format", "text", "--dangerously-skip-permissions"]);

        if let Some(m) = model {
            cmd.args(["--model", m]);
        }

        cmd.kill_on_drop(true).output().await
    })
    .await;

    match result {
        Ok(Ok(output)) => {
            // Capture both stdout and stderr
            let mut combined = output.stdout;
            if !output.stderr.is_empty() {
                combined.extend_from_slice(b"\n[stderr]: ");
                combined.extend_from_slice(&output.stderr);
            }

            let truncated = &combined[..combined.len().min(MAX_OUTPUT_BYTES)];
            let text = String::from_utf8_lossy(truncated).to_string();
            if combined.len() > MAX_OUTPUT_BYTES {
                Ok(format!(
                    "{}\n\n[Output truncated at {}KB]",
                    text,
                    MAX_OUTPUT_BYTES / 1000
                ))
            } else {
                Ok(text)
            }
        }
        Ok(Err(e)) => Err(format!("Process failed: {}", e)),
        Err(_) => Err(format!("Timed out after {}s", timeout_secs)),
    }
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    // Handle install flag
    if args.install {
        match install_globally() {
            Ok(_) => std::process::exit(0),
            Err(e) => {
                eprintln!("{} {}", "Error:".red().bold(), e);
                std::process::exit(1);
            }
        }
    }

    // Ensure task was provided
    let task = match args.task {
        Some(t) => t,
        None => {
            eprintln!("{}", "Error: <TASK> argument is required".red().bold());
            eprintln!();
            eprintln!("Usage: council [OPTIONS] <TASK>");
            eprintln!("       council --install");
            eprintln!();
            eprintln!("For more information try '--help'");
            std::process::exit(1);
        }
    };

    // Validate Claude CLI exists before spawning N processes
    let claude_check = std::process::Command::new("which")
        .arg("claude")
        .output();

    if !claude_check.map(|o| o.status.success()).unwrap_or(false) {
        eprintln!("{}", "Error: 'claude' CLI not found in PATH".red().bold());
        eprintln!();
        eprintln!("Please install Claude Code first:");
        eprintln!("  https://docs.anthropic.com/claude/docs/claude-code");
        std::process::exit(1);
    }

    let constraints = select_constraints(args.num);
    let num_members = constraints.len();

    // Print header
    println!();
    println!("{}", "=".repeat(60).green());
    println!("{}", "                 COUNCIL OF CLAUDES".green().bold());
    println!("{}", "=".repeat(60).green());
    println!();
    println!("  {}: {}", "Members".cyan(), num_members);
    println!("  {}: {}s per member", "Timeout".cyan(), args.timeout);
    if let Some(ref m) = args.model {
        println!("  {}: {}", "Model".cyan(), m);
    }
    println!("  {}: {}", "Synthesize".cyan(), if args.no_synthesize { "no" } else { "yes" });
    println!("  {}: {}", "Task".cyan(), &task[..task.len().min(50)]);
    println!();

    // Show constraint assignments
    for (i, constraint) in constraints.iter().enumerate() {
        let marker = if constraint.mandatory {
            " (mandatory)".yellow()
        } else {
            "".normal()
        };
        println!("  Member #{}: {}{}", i + 1, constraint.name.to_uppercase().blue(), marker);
    }

    println!();
    println!("{}", "=".repeat(60).green());
    println!();

    let (tx, mut rx) = mpsc::channel::<(usize, String, String)>(num_members);
    let start_time = std::time::Instant::now();

    // Spawn all council members
    for (i, constraint) in constraints.iter().enumerate() {
        let tx = tx.clone();
        let prompt = create_prompt(constraint, &task, num_members);
        let name = constraint.name.to_string();
        let timeout = args.timeout;
        let model = args.model.clone();

        println!("{} Member #{}: {}", "[Spawning]".yellow(), i + 1, name.to_uppercase().blue());

        tokio::spawn(async move {
            let result = run_claude(&prompt, timeout, model.as_deref()).await;
            let text = result.unwrap_or_else(|e| format!("[Member {} error: {}]", i + 1, e));
            if let Err(e) = tx.send((i, name, text)).await {
                eprintln!("{}", format!("Failed to send result for member {}: {}", i + 1, e).red());
            }
        });
    }

    drop(tx);

    // Collect results
    let mut outputs: Vec<(usize, String, String)> = Vec::with_capacity(num_members);

    while let Some((id, name, text)) = rx.recv().await {
        println!("{} Member #{}: {}", "[Completed]".green(), id + 1, name.to_uppercase().blue());
        outputs.push((id, name, text));
    }

    outputs.sort_by_key(|(id, _, _)| *id);

    let member_elapsed = start_time.elapsed();

    println!();
    println!("{}", "=".repeat(60).green());
    println!(
        "{}",
        format!("     ALL {} MEMBERS COMPLETED ({:.1}s)", num_members, member_elapsed.as_secs_f64()).green().bold()
    );
    println!("{}", "=".repeat(60).green());
    println!();

    // Print individual member outputs only if --all flag is set
    if args.all {
        for (id, name, text) in &outputs {
            println!();
            println!("{}", "-".repeat(60).blue());
            println!("  MEMBER #{}: {}", id + 1, name.to_uppercase().blue().bold());
            println!("{}", "-".repeat(60).blue());
            println!();
            println!("{}", text);
            println!();
        }
    }

    // Run synthesis by default (unless --no-synthesize)
    if !args.no_synthesize {
        println!();
        println!("{}", "=".repeat(60).magenta());
        println!("{}", "              RUNNING SYNTHESIS...".magenta().bold());
        println!("{}", "=".repeat(60).magenta());
        println!();

        let synthesis_prompt = create_synthesis_prompt(&outputs, &task);
        let synthesis_result = run_claude(&synthesis_prompt, args.timeout, args.model.as_deref()).await;

        println!();
        println!("{}", "=".repeat(60).magenta());
        println!("{}", "           SYNTHESIS & RECOMMENDATIONS".magenta().bold());
        println!("{}", "=".repeat(60).magenta());
        println!();

        match synthesis_result {
            Ok(text) => println!("{}", text),
            Err(e) => println!("{}", format!("[Synthesis failed: {}]", e).red()),
        }

        let total_elapsed = start_time.elapsed();
        println!();
        println!("{}", "=".repeat(60).green());
        println!(
            "{}",
            format!(
                "        TOTAL TIME: {:.1}s (members: {:.1}s, synthesis: {:.1}s)",
                total_elapsed.as_secs_f64(),
                member_elapsed.as_secs_f64(),
                (total_elapsed - member_elapsed).as_secs_f64()
            ).green().bold()
        );
        println!("{}", "=".repeat(60).green());
    } else {
        println!("{}", "=".repeat(60).green());
        println!("{}", "                  END OF COUNCIL".green().bold());
        println!("{}", "=".repeat(60).green());
    }
}
