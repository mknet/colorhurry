//! Generates the Swift and Kotlin types for the Color Hurry bridge.

use std::path::PathBuf;

use anyhow::Result;
use clap::{Parser, ValueEnum};
use color_hurry_core::ColorHurry;
use crux_core::type_generation::facet::{BoltFfi, Config, TypeRegistry};

#[derive(Copy, Clone, PartialEq, Eq, ValueEnum)]
enum Language {
    Swift,
    Kotlin,
}

#[derive(Parser)]
struct Args {
    #[arg(short, long, value_enum)]
    language: Language,
    #[arg(short, long)]
    output_dir: PathBuf,
}

fn main() -> Result<()> {
    let args = Args::parse();

    let typegen = TypeRegistry::new()
        .register_app::<ColorHurry>()?
        .build()?
        // Where `boltffi pack` puts `CoreFfi`, so the generated `Core` only
        // needs an effect handler.
        .boltffi(BoltFfi::new().swift("Shared").kotlin());

    match args.language {
        Language::Swift => {
            let mut builder = Config::builder("App", &args.output_dir);
            builder.platform(".iOS(.v16)");
            typegen.swift(&builder.build())?;
        }
        Language::Kotlin => {
            let config = Config::builder("com.colorhurry.shared", &args.output_dir).build();
            typegen.kotlin(&config)?;
        }
    }

    Ok(())
}
