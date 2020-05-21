use rayon::prelude::*;
use sqlparser::ast::Statement;
use sqlparser::parser::Parser;
use std::collections::HashMap;
use std::fs;
use walkdir::{Error, WalkDir};

use serde_derive::Deserialize;

#[derive(Deserialize, Debug)]
struct PowerSqlConfig {
    project: Project,
}
#[derive(Deserialize, Debug)]
struct Project {
    name: String,
    models: Vec<String>,
}

#[derive(Debug)]
struct PowerSqlDialect {}

impl sqlparser::dialect::Dialect for PowerSqlDialect {
    fn is_identifier_start(&self, ch: char) -> bool {
        // Ref (@) or normal identifier
        (ch == '@') || (ch >= 'a' && ch <= 'z') || (ch >= 'A' && ch <= 'Z')
    }

    fn is_identifier_part(&self, ch: char) -> bool {
        // ANSI SQL
        (ch >= 'a' && ch <= 'z')
            || (ch >= 'A' && ch <= 'Z')
            || (ch >= '0' && ch <= '9')
            || ch == '_'
    }
}

pub fn main() -> Result<(), Error> {
    let root_dir = "examples/project_1/";

    // Load project
    let contents = fs::read_to_string(format!("{}{}", root_dir, "powersql.toml"))
        .expect("No powersql.toml file found");
    let config: PowerSqlConfig = toml::from_str(&contents).unwrap();

    let dialect = PowerSqlDialect {};
    let mut models = vec![];

    for dir in config.project.models {
        for entry in WalkDir::new(format!("{}{}", root_dir, dir)) {
            let entry = entry.unwrap();
            if let Some(ext) = entry.path().extension() {
                {
                    if ext == "sql" {
                        let e = entry.clone();
                        models.push(e);
                    }
                }
            }
        }
    }

    let asts: HashMap<String, Vec<Statement>> = models
        .par_iter()
        .map(|x| {
            let sql = fs::read_to_string(x.path()).unwrap();

            let ast = Parser::parse_sql(&dialect, sql).unwrap();

            (x.path().to_str().unwrap().to_string(), ast)
        })
        .collect();

    println!("{} models", asts.len());
    println!("{:?}", asts);

    Ok(())
}
