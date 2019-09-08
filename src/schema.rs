#[derive(Debug)]
enum Datatype {
    Integer = 1,
}

#[derive(Debug)]
struct Column<'a> {
    name: &'a str,
    datatype: Datatype,
}

#[derive(Debug)]
struct Table<'a> {
    name: &'a str,
    columns: Vec<Column<'a>>,
}
