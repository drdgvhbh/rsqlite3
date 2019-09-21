pub use super::database::data::{Column, ColumnSet, DataType, Schema, TableValue};

#[derive(Debug)]
pub enum Ast {
    MetaCommand(MetaCommand),
    SQLStatement(SQLStatement),
}

#[derive(Debug)]
pub enum MetaCommand {
    Exit,
}

#[derive(Debug)]
pub enum SQLStatement {
    Create(Schema),
    Insert(Insertion),
    Select(Selection),
}

#[derive(Debug)]
pub struct Selection {
    pub table_name: String,
    pub columns: ColumnSet,
}

impl Selection {
    pub fn new(table_name: &str, columns: ColumnSet) -> Selection {
        Selection {
            table_name: table_name.to_string(),
            columns,
        }
    }
    pub fn validate(&self) -> Result<(), String> {
        return Ok(());
    }
}
/*
#[derive(Debug)]
pub struct TableSchema {
    pub name: String,
    pub columns: Vec<Column>,
}

impl TableSchema {
    pub fn new(name: &str, columns: Vec<Column>) -> TableSchema {
        TableSchema {
            name: name.to_string(),
            columns,
        }
    }

    pub fn table_name(&self) -> String {
        self.name.clone()
    }

    pub fn columns(&self) -> Vec<Column> {
        return self.columns.clone();
    }

    pub fn validate(&self) -> Result<(), String> {
        let mut column_names = HashSet::new();
        let mut has_primary_key = false;
        for c in &self.columns {
            if column_names.contains(&c.name) {
                return Err(format!("duplicate column name: {}", c.name));
            }
            if c.is_primary_key && has_primary_key {
                return Err(format!(
                    "table \"{}\" has more than one primary key",
                    self.name
                ));
            }
            if c.is_primary_key {
                has_primary_key = true
            }
            column_names.insert(c.name.clone());
        }
        Ok(())
    }
}
 */
#[derive(Debug)]
pub struct Insertion {
    pub table_name: String,
    pub column_names: Option<Vec<String>>,
    pub values: Vec<TableValue>,
}

impl Insertion {
    pub fn new(
        table_name: &str,
        column_names: Option<Vec<String>>,
        values: Vec<TableValue>,
    ) -> Insertion {
        return Insertion {
            table_name: table_name.to_string(),
            column_names: column_names.map(|column_names| {
                column_names
                    .iter()
                    .map(|column_name| column_name.clone())
                    .collect()
            }),
            values,
        };
    }
    pub fn validate(&self) -> Result<(), String> {
        return self
            .column_names
            .as_ref()
            .map(|column_names| {
                if self.values.len() != column_names.len() {
                    return Err(format!(
                        "{} values for {} columns",
                        self.values.len(),
                        column_names.len()
                    ));
                }

                Ok(())
            })
            .map_or_else(|| Ok(()), |r| r);
    }
}
