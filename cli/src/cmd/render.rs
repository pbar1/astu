use tabled::settings::Style;
use tabled::Table;
use tabled::Tabled;

pub fn markdown_table<T: Tabled>(rows: Vec<T>) -> String {
    let mut table = Table::new(rows);
    table.with(Style::markdown());
    table.to_string()
}

pub fn print_markdown_table<T: Tabled>(rows: Vec<T>) {
    println!("{}", markdown_table(rows));
}

pub fn print_section_table<T: Tabled>(title: &str, rows: Vec<T>) {
    println!("{title}");
    if rows.is_empty() {
        println!("(no rows)");
        println!();
        return;
    }
    print_markdown_table(rows);
    println!();
}
