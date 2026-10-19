use opti_bakery::{SolveResult, explain_result, format_cents, solve_plan};
use opti_domain::BakeryPlan;
use std::{env, fs, path::Path, process::ExitCode};

fn main() -> ExitCode {
    match run(env::args().skip(1).collect()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("error: {message}");
            ExitCode::from(1)
        }
    }
}

fn run(args: Vec<String>) -> Result<(), String> {
    match args.first().map(String::as_str) {
        Some("demo") => {
            print_result(&solve_plan(&opti_bakery::demo_plan()).map_err(|error| error.to_string())?)
        }
        Some("validate") => {
            let path = required_path(&args)?;
            let plan = read_plan(path)?;
            plan.validate().map_err(|error| error.to_string())?;
            println!("valid: {}", path.display());
        }
        Some("solve") => {
            let path = required_path(&args)?;
            let plan = read_plan(path)?;
            print_result(&solve_plan(&plan).map_err(|error| error.to_string())?);
        }
        _ => print_help(),
    }
    Ok(())
}

fn required_path(args: &[String]) -> Result<&Path, String> {
    args.get(1)
        .map(Path::new)
        .ok_or_else(|| "a JSON plan path is required".to_owned())
}

fn read_plan(path: &Path) -> Result<BakeryPlan, String> {
    let contents = fs::read_to_string(path)
        .map_err(|error| format!("could not read {}: {error}", path.display()))?;
    serde_json::from_str(&contents).map_err(|error| {
        format!(
            "could not parse {} as a bakery plan: {error}",
            path.display()
        )
    })
}

fn print_result(result: &SolveResult) {
    println!("Bakery: {}", result.bakery_name);
    println!(
        "Estimated contribution profit: {}",
        format_cents(result.objective_profit_cents)
    );
    println!();
    println!("Production plan:");
    for decision in &result.decisions {
        println!(
            "  {:<20} {:>4} units  ({})",
            decision.product_name,
            decision.units,
            format_cents(decision.profit_cents)
        );
    }
    println!();
    println!("Resources:");
    for resource in &result.resources {
        let marker = if resource.is_bottleneck {
            " [LIMITING]"
        } else {
            ""
        };
        println!(
            "  {:<20} {:>5} / {:<5} {}{}",
            resource.resource_name,
            resource.used_quantity,
            resource.available_quantity,
            resource.unit,
            marker
        );
    }
    println!();
    println!("Explanation:");
    println!("{}", explain_result(result));
}

fn print_help() {
    println!("OptiAI bakery planning CLI");
    println!();
    println!("Commands:");
    println!("  demo                 Solve the built-in bakery example");
    println!("  validate <file.json> Validate a bakery plan without solving it");
    println!("  solve <file.json>    Validate and solve a bakery plan");
}
