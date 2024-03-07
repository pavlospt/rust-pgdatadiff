use crate::diff::table::query::table_types::{
    IncludedExcludedTables, TableMode, TableName, TableOffset, TablePosition, TablePrimaryKeys,
};
use crate::diff::types::SchemaName;
use std::fmt::Display;

pub enum TableQuery {
    AllTablesForSchema(SchemaName, IncludedExcludedTables),
    CountRowsForTable(SchemaName, TableName),
    FindPrimaryKeyForTable(TableName),
    HashQuery(
        SchemaName,
        TableName,
        TablePrimaryKeys,
        TablePosition,
        TableOffset,
    ),
}

impl Display for TableQuery {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AllTablesForSchema(schema_name, included_excluded_tables) => {
                let inclusion_exclusion_statement = match included_excluded_tables.table_mode() {
                    None => "".to_string(),
                    Some(table_mode) => match table_mode {
                        TableMode::Include => included_excluded_tables.inclusion_statement(),
                        TableMode::Exclude => included_excluded_tables.exclusion_statement(),
                    },
                };

                write!(
                    f,
                    r#"
                SELECT table_name
                FROM information_schema.tables
                WHERE table_schema = '{}'
                {}
                "#,
                    schema_name.name(),
                    inclusion_exclusion_statement
                )
            }
            // https://stackoverflow.com/questions/7943233/fast-way-to-discover-the-row-count-of-a-table-in-postgresql
            TableQuery::CountRowsForTable(schema_name, table_name) => {
                write!(
                    f,
                    "SELECT count(*) FROM {}.{}",
                    schema_name.name(),
                    table_name.name()
                )
            }
            TableQuery::FindPrimaryKeyForTable(table_name) => write!(
                f,
                // language=postgresql
                r#"
                SELECT a.attname
                FROM   pg_index i
                JOIN   pg_attribute a ON a.attrelid = i.indrelid
                                     AND a.attnum = ANY(i.indkey)
                WHERE  i.indrelid = '{}'::regclass
                AND    i.indisprimary"#,
                table_name.name()
            ),
            TableQuery::HashQuery(
                schema_name,
                table_name,
                table_primary_keys,
                table_position,
                table_offset,
            ) => {
                write!(
                    f,
                    r#"
                    SELECT md5(array_agg(md5((t.*)::varchar))::varchar)
                    FROM (
                        SELECT *
                        FROM {}.{}
                        ORDER BY {} limit {} offset {}
                    ) AS t
                    "#,
                    schema_name.name(),
                    table_name.name(),
                    table_primary_keys.keys(),
                    table_offset.offset(),
                    table_position.position(),
                )
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_display_all_tables_for_schema_with_included_tables() {
        let schema_name = SchemaName::new("public");
        let included_tables = vec!["table1".to_string(), "table2".to_string()];
        let excluded_tables: Vec<String> = vec![];
        let included_excluded_tables =
            IncludedExcludedTables::new(included_tables, excluded_tables);
        let query = TableQuery::AllTablesForSchema(schema_name, included_excluded_tables);
        let expected = r#"
                SELECT table_name
                FROM information_schema.tables
                WHERE table_schema = 'public'
                AND table_name IN ('table1','table2')
                "#;
        assert_eq!(expected, query.to_string());
    }

    #[test]
    fn test_display_all_tables_for_schema_with_excluded_tables() {
        let schema_name = SchemaName::new("public");
        let included_tables: Vec<String> = vec![];
        let excluded_tables = vec!["table1", "table2"];
        let included_excluded_tables =
            IncludedExcludedTables::new(included_tables, excluded_tables);
        let query = TableQuery::AllTablesForSchema(schema_name, included_excluded_tables);
        let expected = r#"
                SELECT table_name
                FROM information_schema.tables
                WHERE table_schema = 'public'
                AND table_name NOT IN ('table1','table2')
                "#;
        assert_eq!(expected, query.to_string());
    }

    #[test]
    fn test_display_count_rows_for_table() {
        let schema_name = SchemaName::new("public".to_string());
        let table_name = TableName::new("table1".to_string());
        let query = TableQuery::CountRowsForTable(schema_name, table_name);
        let expected = "SELECT count(*) FROM public.table1";
        assert_eq!(expected, query.to_string());
    }

    #[test]
    fn test_display_find_primary_key_for_table() {
        let table_name = TableName::new("table1".to_string());
        let query = TableQuery::FindPrimaryKeyForTable(table_name);
        let expected = r#"
                SELECT a.attname
                FROM   pg_index i
                JOIN   pg_attribute a ON a.attrelid = i.indrelid
                                     AND a.attnum = ANY(i.indkey)
                WHERE  i.indrelid = 'table1'::regclass
                AND    i.indisprimary"#;
        assert_eq!(expected, query.to_string());
    }

    #[test]
    fn test_display_hash_query() {
        let schema_name = SchemaName::new("public".to_string());
        let table_name = TableName::new("table1".to_string());
        let table_primary_keys = TablePrimaryKeys::new("id".to_string());
        let table_position = TablePosition::new(0);
        let table_offset = TableOffset::new(100);
        let query = TableQuery::HashQuery(
            schema_name,
            table_name,
            table_primary_keys,
            table_position,
            table_offset,
        );
        let expected = r#"
                    SELECT md5(array_agg(md5((t.*)::varchar))::varchar)
                    FROM (
                        SELECT *
                        FROM public.table1
                        ORDER BY id limit 100 offset 0
                    ) AS t
                    "#;
        assert_eq!(expected, query.to_string());
    }
}
