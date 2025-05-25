use evalexpr::*;
use std::io::Result;

use crate::{csv::Csv, rule::CompiledRule};

pub struct Match {
    pub csv: Vec<String>,
    pub rule: String,
}

pub fn process_file<T>(mut csv: Csv<T>, rules: Vec<CompiledRule>) -> Result<Vec<Match>>
where
    T: std::io::Read,
{
    let mut results = vec![];

    for result in csv.reader.records() {
        let record = result?;

        // setup context with custom functions
        let mut context: HashMapContext<DefaultNumericTypes> = context_map! {
            "string_contains" => Function::new(|argument| {
                let arguments = argument.as_tuple()?;

                if let (Value::String(a), Value::String(b)) = (&arguments[0], &arguments[1]) {
                    Ok(Value::Boolean(a.contains(b)))
                } else {
                    Ok(Value::Boolean(false))
                }
            }),
        }
        .unwrap(); // Do proper error handling here

        // fill the evaluation context with record values
        record.iter().enumerate().for_each(|(idx, s)| {
            // try to convert as number if possible, because operators +/- ... don't work on strings
            let value: Value<_> = if let Ok(f) = s.parse::<f64>() {
                Value::Float(f)
            } else {
                Value::from(s)
            };

            context.set_value(csv.headers[idx].clone(), value).unwrap();
        });

        // apply each rule on values
        rules.iter().for_each(|rule| {
            if rule.compiled().eval_with_context(&context) == Ok(Value::from(true)) {
                // store matching rules in results
                results.push(Match {
                    csv: record.iter().map(|s| s.to_owned()).collect(),
                    rule: rule.rule().to_owned(),
                });
            }
        });
    }
    Ok(results)
}
