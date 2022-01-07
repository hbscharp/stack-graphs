// -*- coding: utf-8 -*-
// ------------------------------------------------------------------------------------------------
// Copyright © 2021, stack-graphs authors.
// Licensed under either of Apache License, Version 2.0, or MIT license, at your option.
// Please see the LICENSE-APACHE or LICENSE-MIT files in this distribution for license details.
// ------------------------------------------------------------------------------------------------

use std::path::PathBuf;

use anyhow::anyhow;
use anyhow::Context;
use anyhow::Error;
use clap::Parser;
use tree_sitter::Language;
use tree_sitter_config::Config;
use tree_sitter_loader::LanguageConfiguration;
use tree_sitter_loader::Loader;

/// Parses and analyzes a source file, producing a stack graph from it
///
/// This command uses a tree-sitter grammar to parse a source file, and then uses a TSG file to
/// construct a stack graph from the parsed syntax tree.
///
/// We will try to automatically find the correct grammar to use to parse the file using
/// tree-sitter's usual auto-detection rules.  If we cannot auto-detect a grammar, or if you want
/// to override our choice, you can use the --scope option to manually specify one.
///
/// We will try to automatically find the TSG file containing the stack graph construction rules
/// for the language.  Stack graph TSG files are typically found in the language's grammar repo,
/// under the path ‘queries/stack-graphs.tsg’.  If the grammar repo doesn't contain that file, or
/// if you want to override the stack graph construction rules, use the --tsg option to manually
/// specify a TSG file.
#[derive(Parser)]
pub struct Command {
    /// The source file to analyze
    source_file: PathBuf,

    /// The TSG file containing the stack graph construction rules.
    #[clap(long)]
    tsg: Option<PathBuf>,

    /// The scope of the tree-sitter grammar to use
    #[clap(long)]
    scope: Option<String>,
}

impl Command {
    pub fn run(&self) -> Result<(), Error> {
        let mut loader = Loader::new()?;
        let language_config = self.select_language(&mut loader)?;
        println!(
            "Create a stack graph {}",
            language_config.root_path.display()
        );
        Ok(())
    }

    // TODO: Some version of this should be upstreamed to tree-sitter-loader
    fn select_language<'a>(&'a self, loader: &'a mut Loader) -> Result<(Language, PathBuf), Error> {
        let config = Config::load()?;
        let loader_config = config.get()?;
        loader.find_all_languages(&loader_config)?;

        if let Some(scope) = &self.scope {
            let scope_config = loader
                .language_configuration_for_scope(scope)
                .with_context(|| format!("Failed to load language for scope '{}'", scope))?;
            match scope_config {
                Some((lang, config)) => return Ok((lang, config.root_path.clone())),
                None => return Err(anyhow!("Unknown scope '{}'", scope)),
            }
        }

        let path_config = loader
            .language_configuration_for_file_name(&self.source_file)
            .with_context(|| {
                format!(
                    "Failed to load language for file name {}",
                    self.source_file.display(),
                )
            })?;
        if let Some((lang, config)) = path_config {
            return Ok((lang, config.root_path));
        }

        let current_dir = std::env::current_dir()?;
        let pwd_configs = loader
            .find_language_configurations_at_path(&current_dir)
            .with_context(|| "Failed to load language in current directory")?;
        if let Some(config) = pwd_configs.first() {
            return Ok(config);
        }

        Err(anyhow!("No language found"))
    }
}
