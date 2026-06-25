//! AONIX command-line interface (sub-phase 1.J).
//!
//! Thin shell over the deterministic library `aonix`. Every technical
//! decision — whether a `.aoncir` is valid, what a circuit outputs, its
//! canonical hash — is computed by the library, never by the CLI. The CLI
//! only reads files, forwards arguments and prints results.
//!
//! Subcommands:
//!
//! ```text
//! aonix validate     <file.aoncir>            parse + validate, print a summary
//! aonix hash         <file.aoncir>            print the canonical blake3 hash
//! aonix canon        <file.aoncir>            print the canonical re-serialization
//! aonix simulate     <file.aoncir> <bits>     simulate one input vector (e.g. 101)
//! aonix truth-table  <file.aoncir>            print the full truth table
//! aonix help                                  show this help
//! ```

use std::process::ExitCode;

use aonix::circuit_model::{Bit, Circuit, InputVector};
use aonix::format::aoncir::{hash_canonical, parse, write};
use aonix::memory::{CircuitKey, MemoryStore, PromotionOutcome};
use aonix::simulation::{simulate, simulate_exhaustive};

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().collect();
    match run(&args) {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("error: {message}");
            ExitCode::FAILURE
        }
    }
}

fn run(args: &[String]) -> Result<(), String> {
    match args.get(1).map(String::as_str) {
        Some("validate") => cmd_validate(args.get(2)),
        Some("hash") => cmd_hash(args.get(2)),
        Some("canon") => cmd_canon(args.get(2)),
        Some("simulate") => cmd_simulate(args.get(2), args.get(3)),
        Some("truth-table") => cmd_truth_table(args.get(2)),
        Some("mem") => cmd_mem(args),
        Some("help") | Some("--help") | Some("-h") | None => {
            print_usage();
            Ok(())
        }
        Some(other) => Err(format!(
            "unknown subcommand {other:?}. Run `aonix-cli help` for the list."
        )),
    }
}

fn load(path: Option<&String>) -> Result<Circuit, String> {
    let path = path.ok_or("missing <file.aoncir> argument")?;
    let text = std::fs::read_to_string(path).map_err(|error| format!("cannot read {path}: {error}"))?;
    parse(&text).map_err(|error| format!("invalid .aoncir: {error}"))
}

fn cmd_validate(path: Option<&String>) -> Result<(), String> {
    let circuit = load(path)?;
    println!(
        "valid: {} input(s), {} output(s), {} signal(s), {} gate(s), {} semantic group(s)",
        circuit.input_count(),
        circuit.output_count(),
        circuit.signal_count(),
        circuit.gate_count(),
        circuit.semantic_group_count(),
    );
    Ok(())
}

fn cmd_hash(path: Option<&String>) -> Result<(), String> {
    let circuit = load(path)?;
    println!("{}", hash_canonical(&circuit));
    Ok(())
}

fn cmd_canon(path: Option<&String>) -> Result<(), String> {
    let circuit = load(path)?;
    print!("{}", write(&circuit));
    Ok(())
}

fn cmd_simulate(path: Option<&String>, bits: Option<&String>) -> Result<(), String> {
    let circuit = load(path)?;
    let bits = bits.ok_or("missing <input-bits> argument, e.g. 101")?;
    let input = InputVector::new(parse_bits(bits)?);
    let output = simulate(&circuit, &input).map_err(|error| error.to_string())?;

    let labelled: Vec<String> = circuit
        .outputs_in_order()
        .iter()
        .zip(output.as_slice())
        .map(|(port, bit)| format!("{}={}", port.identifier.as_str(), bit_char(*bit)))
        .collect();
    println!("out = {}", bits_to_string(output.as_slice()));
    println!("{}", labelled.join("  "));
    Ok(())
}

fn cmd_truth_table(path: Option<&String>) -> Result<(), String> {
    let circuit = load(path)?;
    let input_names: Vec<&str> = circuit
        .inputs_in_order()
        .iter()
        .map(|port| port.identifier.as_str())
        .collect();
    let output_names: Vec<&str> = circuit
        .outputs_in_order()
        .iter()
        .map(|port| port.identifier.as_str())
        .collect();
    let table = simulate_exhaustive(&circuit).map_err(|error| error.to_string())?;

    println!("# {} | {}", input_names.join(" "), output_names.join(" "));
    for (input, output) in table {
        println!(
            "{} | {}",
            bits_to_string(input.as_slice()),
            bits_to_string(output.as_slice())
        );
    }
    Ok(())
}

fn cmd_mem(args: &[String]) -> Result<(), String> {
    match args.get(2).map(String::as_str) {
        Some("list") => mem_list(args.get(3)),
        Some("show") => mem_show(args.get(3), args.get(4), args.get(5)),
        Some("history") => mem_history(args.get(3), args.get(4), args.get(5)),
        Some("promote") => mem_promote(args.get(3), args.get(4), args.get(5), args.get(6)),
        _ => Err(
            "usage: aonix-cli mem <list|show|history|promote> <memory-root> [name] [parameters] [file]".into(),
        ),
    }
}

fn open_store(root: Option<&String>) -> Result<MemoryStore, String> {
    let root = root.ok_or("missing <memory-root> argument")?;
    Ok(MemoryStore::open(root))
}

fn make_key(name: Option<&String>, parameters: Option<&String>) -> Result<CircuitKey, String> {
    let name = name.ok_or("missing <name> argument")?;
    let key = match parameters {
        Some(parameters) => CircuitKey::new(name.clone(), parameters.clone()),
        None => CircuitKey::simple(name.clone()),
    };
    key.map_err(|error| error.to_string())
}

fn mem_list(root: Option<&String>) -> Result<(), String> {
    let store = open_store(root)?;
    let keys = store.list().map_err(|error| error.to_string())?;
    if keys.is_empty() {
        println!("(memory empty)");
    }
    for key in keys {
        let active = store.active_hash(&key).map_err(|error| error.to_string())?;
        let history = store.history_hexes(&key).map_err(|error| error.to_string())?.len();
        println!(
            "{} [{}]  active={}  history={}",
            key.name(),
            key.parameters(),
            active.as_deref().unwrap_or("<none>"),
            history,
        );
    }
    Ok(())
}

fn mem_show(
    root: Option<&String>,
    name: Option<&String>,
    parameters: Option<&String>,
) -> Result<(), String> {
    let store = open_store(root)?;
    let key = make_key(name, parameters)?;
    match store.active(&key).map_err(|error| error.to_string())? {
        Some(circuit) => {
            println!("active hash : {}", hash_canonical(&circuit));
            println!(
                "interface   : {} input(s), {} output(s), {} gate(s)",
                circuit.input_count(),
                circuit.output_count(),
                circuit.gate_count(),
            );
        }
        None => println!("no active version for {} [{}]", key.name(), key.parameters()),
    }
    Ok(())
}

fn mem_history(
    root: Option<&String>,
    name: Option<&String>,
    parameters: Option<&String>,
) -> Result<(), String> {
    let store = open_store(root)?;
    let key = make_key(name, parameters)?;
    let hexes = store.history_hexes(&key).map_err(|error| error.to_string())?;
    if hexes.is_empty() {
        println!("(no history)");
    }
    for hex in hexes {
        println!("blake3:{hex}");
    }
    Ok(())
}

fn mem_promote(
    root: Option<&String>,
    name: Option<&String>,
    parameters: Option<&String>,
    file: Option<&String>,
) -> Result<(), String> {
    let store = open_store(root)?;
    let key = make_key(name, parameters)?;
    let circuit = load(file)?;
    let outcome = store.promote(&key, &circuit).map_err(|error| error.to_string())?;
    match outcome {
        PromotionOutcome::InstalledFirst { hash } => println!("installed first active: {hash}"),
        PromotionOutcome::Replaced { new_hash, archived_hash } => {
            println!("replaced: new={new_hash} archived={archived_hash}")
        }
        PromotionOutcome::AlreadyActive { hash } => println!("already active (dedup): {hash}"),
        PromotionOutcome::RejectedNotBetter { candidate_hash, active_hash } => {
            println!("rejected (not strictly better): candidate={candidate_hash} active={active_hash}")
        }
    }
    Ok(())
}

fn parse_bits(spec: &str) -> Result<Vec<Bit>, String> {
    spec.chars()
        .map(|character| match character {
            '0' => Ok(Bit::ZERO),
            '1' => Ok(Bit::ONE),
            other => Err(format!("invalid bit {other:?}; use only the digits 0 and 1")),
        })
        .collect()
}

fn bit_char(bit: Bit) -> char {
    if bit.is_one() { '1' } else { '0' }
}

fn bits_to_string(bits: &[Bit]) -> String {
    bits.iter().map(|bit| bit_char(*bit)).collect()
}

fn print_usage() {
    println!("AONIX - AND-OR-NOT Integrated eXploration");
    println!();
    println!("Usage: aonix-cli <subcommand> [arguments]");
    println!();
    println!("Subcommands:");
    println!("  validate     <file.aoncir>          parse + validate, print a summary");
    println!("  hash         <file.aoncir>          print the canonical blake3 hash");
    println!("  canon        <file.aoncir>          print the canonical re-serialization");
    println!("  simulate     <file.aoncir> <bits>   simulate one input vector (e.g. 101)");
    println!("  truth-table  <file.aoncir>          print the full truth table");
    println!("  mem list     <root>                            list circuits in the memory store");
    println!("  mem show     <root> <name> [params]            show the active version");
    println!("  mem history  <root> <name> [params]            list archived versions");
    println!("  mem promote  <root> <name> <params> <file>     promote a .aoncir candidate");
    println!("  help                                show this help");
}
