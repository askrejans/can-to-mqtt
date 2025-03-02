use prettytable::{Table, row};

pub struct VehicleTable {
    table: Table,
}

impl VehicleTable {
    pub fn new() -> Self {
        let table = Table::new();
        VehicleTable { table }
    }

    pub fn add_row(&mut self, parameter: &str, value: &str, unit: &str) {
        self.table.add_row(row![parameter, value, unit]);
    }

    pub fn display(&self) {
        self.table.printstd();
    }

    pub fn clear(&mut self) {
        self.table = Table::new();
    }
}