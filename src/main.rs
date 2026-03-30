mod cli;
mod config;
mod executor;
mod repl;
mod store;
mod ui;

use clap::Parser;

use cli::{Cli, Command};
use cli::output::format_result;
use cli::parse::{build_search_filters, parse_confidence, parse_importance, parse_tags};
use executor::remember::RememberArgs;
use store::models::MemoryType;

fn main() {
    let cli = Cli::parse();

    if cli.interactive {
        repl::Repl::new().run();
        return;
    }

    let command = match cli.command {
        Some(cmd) => cmd,
        None => {
            println!("MatyMemory v{}", env!("CARGO_PKG_VERSION"));
            println!("Use -i for interactive mode, or a subcommand. Use --help for more info.");
            return;
        }
    };

    let db_path = config::resolve_db_path(cli.db.as_deref());
    let store = match store::Store::open(&db_path) {
        Ok(s) => s,
        Err(e) => {
            if cli.json {
                let err = serde_json::json!({"error": e.to_string()});
                eprintln!("{}", serde_json::to_string_pretty(&err).unwrap());
            } else {
                eprintln!("Error opening database: {e}");
            }
            std::process::exit(1);
        }
    };

    let result = run_command(&store, command);
    match result {
        Ok(cr) => {
            let output = format_result(&cr, cli.json, cli.quiet);
            if !output.is_empty() {
                println!("{output}");
            }
        }
        Err(e) => {
            if cli.json {
                let err = serde_json::json!({"error": e.to_string()});
                eprintln!("{}", serde_json::to_string_pretty(&err).unwrap());
            } else {
                eprintln!("Error: {e}");
            }
            std::process::exit(1);
        }
    }
}

fn run_command(
    store: &store::Store,
    command: Command,
) -> store::error::Result<executor::CommandResult> {
    match command {
        Command::Remember {
            content,
            memory_type,
            tags,
            importance,
            confidence,
            actor,
            session,
            model,
            reason,
        } => {
            let mt = match memory_type {
                Some(t) => t.parse()?,
                None => MemoryType::Semantic,
            };
            let tag_list = tags.map(|t| parse_tags(&t)).unwrap_or_default();
            let imp = parse_importance(importance.unwrap_or(0.5))?;
            let conf = parse_confidence(confidence.unwrap_or(0.8))?;
            executor::execute_remember(store, RememberArgs {
                content,
                memory_type: mt,
                tags: tag_list,
                importance: imp,
                confidence: conf,
                actor,
                session_id: session,
                model_id: model,
                write_reason: reason,
            })
        }
        Command::Recall { query, memory_type, status, tag, limit, offset } => {
            let filters = build_search_filters(
                query.as_deref(),
                memory_type.as_deref(),
                status.as_deref(),
                tag.as_deref(),
                limit,
                offset,
            )?;
            executor::execute_recall(store, filters)
        }
        Command::Get { id } => executor::execute_get(store, &id),
        Command::Inspect { id } => executor::execute_inspect(store, &id),
        Command::Update { id, content, importance, confidence } => {
            if let Some(v) = importance { parse_importance(v)?; }
            if let Some(v) = confidence { parse_confidence(v)?; }
            executor::execute_update(store, &id, content.as_deref(), importance, confidence)
        }
        Command::Archive { id } => executor::execute_archive(store, &id),
        Command::Invalidate { id } => executor::execute_invalidate(store, &id),
        Command::Supersede { old_id, new_id } => {
            executor::execute_supersede(store, &old_id, &new_id)
        }
        Command::Forget { id } => executor::execute_forget(store, &id),
        Command::Pin { id } => executor::execute_pin(store, &id),
        Command::Tag { id, tags } => {
            let tag_list = parse_tags(&tags);
            let tag_refs: Vec<&str> = tag_list.iter().map(|s| s.as_str()).collect();
            executor::execute_add_tags(store, &id, &tag_refs)
        }
        Command::Untag { id, tags } => {
            let tag_list = parse_tags(&tags);
            let tag_refs: Vec<&str> = tag_list.iter().map(|s| s.as_str()).collect();
            executor::execute_remove_tags(store, &id, &tag_refs)
        }
        Command::Relate { from_id, to_id, relation_type } => {
            executor::execute_relate(store, &from_id, &to_id, &relation_type)
        }
        Command::List { memory_type, status, tag, limit, offset } => {
            let filters = build_search_filters(
                None,
                memory_type.as_deref(),
                status.as_deref(),
                tag.as_deref(),
                limit,
                offset,
            )?;
            executor::execute_list(store, filters)
        }
        Command::Stats => executor::execute_stats(store),
    }
}
