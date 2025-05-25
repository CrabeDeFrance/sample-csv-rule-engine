// Return a vec of rules, one rule per line in original file
use evalexpr::*;
use serde::Deserialize;
use std::{
    io::{Error, ErrorKind, Result},
    path::Path,
};

pub fn read_from_path(file_path: &Path) -> Result<Vec<CompiledRule>> {
    let content = std::fs::read_to_string(file_path)?;
    load_rules(&content)
}

pub fn read_from_str(content: &str) -> Result<Vec<CompiledRule>> {
    load_rules(content)
}

fn load_rules(content: &str) -> Result<Vec<CompiledRule>> {
    let rules: Vec<Rule> = serde_json::from_str(content).map_err(|e| {
        Error::new(
            ErrorKind::InvalidInput,
            format!("Can't deserialize file: {e}"),
        )
    })?;

    let mut compiled_rules = vec![];
    for rule in rules {
        let compiled = build_operator_tree::<DefaultNumericTypes>(&rule.rule).map_err(|e| {
            Error::new(
                ErrorKind::InvalidInput,
                format!("Can't compile rule: {}: {e}", &rule.rule),
            )
        })?; // Do proper error handling here

        compiled_rules.push(CompiledRule { rule, compiled });
    }

    Ok(compiled_rules)
}

#[derive(Deserialize, Clone, Debug)]
pub struct Rule {
    rule: String,
}

pub struct CompiledRule {
    rule: Rule,
    compiled: Node,
}

impl CompiledRule {
    pub fn rule(&self) -> &String {
        &self.rule.rule
    }

    pub fn compiled(&self) -> &Node {
        &self.compiled
    }
}
