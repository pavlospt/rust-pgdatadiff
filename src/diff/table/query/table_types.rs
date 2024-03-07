#[derive(Clone)]
pub struct TableName(String);

impl TableName {
    pub fn new(name: impl Into<String>) -> Self {
        Self(name.into())
    }

    pub fn name(&self) -> &str {
        &self.0
    }
}

#[derive(Clone)]
pub struct TablePrimaryKeys(String);

impl TablePrimaryKeys {
    pub fn new(keys: impl Into<String>) -> Self {
        Self(keys.into())
    }

    pub fn keys(&self) -> &str {
        &self.0
    }
}

#[derive(Clone)]
pub struct TablePosition(i64);

impl TablePosition {
    pub fn new(position: i64) -> Self {
        Self(position)
    }

    pub fn position(&self) -> i64 {
        self.0
    }
}

#[derive(Clone)]
pub struct TableOffset(i64);

impl TableOffset {
    pub fn new(offset: i64) -> Self {
        Self(offset)
    }

    pub fn offset(&self) -> i64 {
        self.0
    }
}

pub struct IncludedExcludedTables {
    included_tables: Vec<String>,
    excluded_tables: Vec<String>,
}

pub enum TableMode {
    Include,
    Exclude,
}

impl IncludedExcludedTables {
    pub fn new(
        include_tables: Vec<impl Into<String>>,
        exclude_tables: Vec<impl Into<String>>,
    ) -> Self {
        if !include_tables.is_empty() && !exclude_tables.is_empty() {
            panic!("Cannot include and exclude tables at the same time");
        }

        Self {
            included_tables: include_tables.into_iter().map(|t| t.into()).collect(),
            excluded_tables: exclude_tables.into_iter().map(|t| t.into()).collect(),
        }
    }

    pub fn table_mode(&self) -> Option<TableMode> {
        if self.has_included_tables() {
            Some(TableMode::Include)
        } else if self.has_excluded_tables() {
            Some(TableMode::Exclude)
        } else {
            None
        }
    }

    pub fn exclusion_statement(&self) -> String {
        if !self.has_excluded_tables() {
            return String::new();
        }

        let joined_tables = self
            .excluded_tables
            .iter()
            .map(|table| format!("'{}'", table))
            .collect::<Vec<String>>()
            .join(",");

        format!("AND table_name NOT IN ({})", joined_tables)
    }

    pub fn inclusion_statement(&self) -> String {
        if !self.has_included_tables() {
            return String::new();
        }

        let joined_tables = self
            .included_tables
            .iter()
            .map(|table| format!("'{}'", table))
            .collect::<Vec<String>>()
            .join(",");

        format!("AND table_name IN ({})", joined_tables)
    }

    fn has_included_tables(&self) -> bool {
        !self.included_tables.is_empty()
    }

    fn has_excluded_tables(&self) -> bool {
        !self.excluded_tables.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_included_tables_when_include_tables_not_empty() {
        let included_tables = vec!["table1", "table2"];
        let excluded_tables: Vec<String> = vec![];
        let included_excluded_tables =
            IncludedExcludedTables::new(included_tables, excluded_tables);

        assert!(matches!(
            included_excluded_tables.table_mode().unwrap(),
            TableMode::Include
        ));

        assert_eq!(
            included_excluded_tables.inclusion_statement(),
            "AND table_name IN ('table1','table2')"
        );
    }

    #[test]
    fn test_excluded_tables_when_exclude_tables_not_empty() {
        let included_tables: Vec<String> = vec![];
        let excluded_tables = vec!["table1", "table2"];
        let included_excluded_tables =
            IncludedExcludedTables::new(included_tables, excluded_tables);

        assert!(matches!(
            included_excluded_tables.table_mode().unwrap(),
            TableMode::Exclude
        ));

        assert_eq!(
            included_excluded_tables.exclusion_statement(),
            "AND table_name NOT IN ('table1','table2')"
        );
    }

    #[test]
    fn test_when_included_tables_and_excluded_tables_empty() {
        let included_tables: Vec<String> = vec![];
        let excluded_tables: Vec<String> = vec![];
        let included_excluded_tables =
            IncludedExcludedTables::new(included_tables, excluded_tables);

        assert_eq!(included_excluded_tables.inclusion_statement(), "");
    }

    #[test]
    #[should_panic = "Cannot include and exclude tables at the same time"]
    fn test_when_included_tables_and_excluded_tables_are_both_not_empty() {
        let included_tables: Vec<&str> = vec!["table1"];
        let excluded_tables: Vec<&str> = vec!["table2"];
        _ = IncludedExcludedTables::new(included_tables, excluded_tables);
    }
}
