use std::{
    collections::HashMap,
    error::Error,
    fs::File,
    io,
};

use serde::{
    Deserialize,
    Serialize,
};

type Result<T> = std::result::Result<T, Box<dyn Error>>;

type OptimizedSize = f32;
type GasUsage = u128;

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct Row {
    name: String,
    optimized_size: OptimizedSize,
    total_size: OptimizedSize,
    gas_usage: GasUsage,
    total_gas_usage: GasUsage,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct SizesRow {
    name: String,
    optimized_size: OptimizedSize,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct GasUsageRow {
    name: String,
    gas_usage: GasUsage,
}

#[derive(Debug, Default)]
struct CsvComparator {
    old_sizes: HashMap<String, OptimizedSize>,
    new_sizes: HashMap<String, OptimizedSize>,
    old_gas_usage: HashMap<String, GasUsage>,
    new_gas_usage: HashMap<String, GasUsage>,
}

impl CsvComparator {
    fn new() -> Self {
        Default::default()
    }

    fn write_old_sizes(&mut self, file: File) -> Result<()> {
        read_csv_size(&mut self.old_sizes, file)
    }

    fn write_new_sizes(&mut self, file: File) -> Result<()> {
        read_csv_size(&mut self.new_sizes, file)
    }

    fn write_old_gas_usage(&mut self, file: File) -> Result<()> {
        read_csv_gas(&mut self.old_gas_usage, file)
    }

    fn write_new_gas_usage(&mut self, file: File) -> Result<()> {
        read_csv_gas(&mut self.new_gas_usage, file)
    }

    fn get_diffs(&self) -> Result<Vec<Row>> {
        let mut result = Vec::new();
        let mut all_contracts: HashMap<String, ()> = self
            .old_sizes
            .iter()
            .map(|(k, _)| (k.clone(), ()))
            .collect();
        self.new_sizes.iter().for_each(|(k, _)| {
            all_contracts.insert(k.clone(), ());
        });

        for (contract, _) in all_contracts {
            let def = OptimizedSize::default();
            let opt_size_old = self
                .old_sizes
                .get(&contract)
                .or(Some(&def))
                .unwrap_or_else(|| {
                    panic!("failed getting old_sizes entry for {:?}", contract)
                });
            let opt_size_new = self
                .new_sizes
                .get(&contract)
                .or(Some(&def))
                .unwrap_or_else(|| {
                    panic!("failed getting new_sizes entry for {:?}", contract)
                });
            let opt_size_diff = opt_size_new - opt_size_old;

            let old_gas_usage = self
                .old_gas_usage
                .get(&contract)
                .or(Some(&0))
                .unwrap_or_else(|| {
                    panic!("failed getting old_gas_usage entry for {:?}", contract)
                });
            let new_gas_usage = self
                .new_gas_usage
                .get(&contract)
                .or(Some(&0))
                .unwrap_or_else(|| {
                    panic!("failed getting new_gas_usage entry for {:?}", contract)
                });
            let gas_usage_diff = new_gas_usage - old_gas_usage;

            let row = Row {
                name: contract.to_string(),
                optimized_size: opt_size_diff,
                total_size: *opt_size_new,
                gas_usage: gas_usage_diff,
                total_gas_usage: *new_gas_usage,
            };
            result.push(row);
        }

        Ok(result)
    }
}

fn read_csv_size(map: &mut HashMap<String, OptimizedSize>, file: File) -> Result<()> {
    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(false)
        .trim(csv::Trim::All)
        .from_reader(file);

    for result in rdr.deserialize() {
        let record: SizesRow = result?;
        map.insert(record.name, record.optimized_size);
    }

    Ok(())
}

fn read_csv_gas(map: &mut HashMap<String, GasUsage>, file: File) -> Result<()> {
    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(false)
        .trim(csv::Trim::All)
        .from_reader(file);

    for result in rdr.deserialize() {
        let record: GasUsageRow = result?;
        map.insert(record.name, record.gas_usage);
    }

    Ok(())
}

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let old_sizes = File::open(&args[1])?;
    let new_sizes = File::open(&args[2])?;
    let old_gas_usage = File::open(&args[3])?;
    let new_gas_usage = File::open(&args[4])?;

    let mut comparator = CsvComparator::new();
    comparator.write_old_sizes(old_sizes)?;
    comparator.write_new_sizes(new_sizes)?;
    comparator.write_old_gas_usage(old_gas_usage)?;
    comparator.write_new_gas_usage(new_gas_usage)?;

    let mut wtr = csv::WriterBuilder::new()
        .has_headers(false)
        .from_writer(io::stdout());

    for row in comparator.get_diffs()? {
        let prefix_opt = if row.optimized_size > 0.0 { "+" } else { "" };
        let prefix_gas = if row.gas_usage > 0 { "+" } else { "" };
        let optimized_size = format!("{}{:.2} K", prefix_opt, row.optimized_size);
        let gas_usage = format!("{}{}", prefix_gas, row.gas_usage);
        let total_size = format!("{:.2} K", row.total_size);
        let human_readable_row = (
            row.name,
            optimized_size,
            gas_usage,
            total_size,
            row.total_gas_usage,
        );
        wtr.serialize(human_readable_row)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn entries_existent_only_in_old_csv_must_still_appear_in_diff() {
        // given
        let mut old_sizes: HashMap<String, OptimizedSize> = HashMap::new();
        let new_sizes: HashMap<String, OptimizedSize> = HashMap::new();
        let old_gas_usage: HashMap<String, GasUsage> = HashMap::new();
        let new_gas_usage: HashMap<String, GasUsage> = HashMap::new();
        let optimized_size = 0.1337;
        old_sizes.insert("removed_in_new_csv".to_string(), optimized_size);
        let comparator = CsvComparator {
            old_sizes,
            new_sizes,
            old_gas_usage,
            new_gas_usage,
        };

        // when
        let res = comparator.get_diffs().expect("getting diffs failed");

        // then
        let mut iter = res.iter();
        assert_eq!(
            iter.next().expect("first diff entry must exist"),
            &Row {
                name: "removed_in_new_csv".to_string(),
                optimized_size: optimized_size * -1.0,
                total_size: 0.0,
                gas_usage: 0,
                total_gas_usage: 0,
            }
        );
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn entries_existent_only_in_new_csv_must_still_appear_in_diff() {
        // given
        let old_sizes: HashMap<String, OptimizedSize> = HashMap::new();
        let mut new_sizes: HashMap<String, OptimizedSize> = HashMap::new();
        let old_gas_usage: HashMap<String, GasUsage> = HashMap::new();
        let new_gas_usage: HashMap<String, GasUsage> = HashMap::new();
        let optimized_size = 0.1337;
        new_sizes.insert("not_existent_in_old_csv".to_string(), optimized_size);
        let comparator = CsvComparator {
            old_sizes,
            new_sizes,
            old_gas_usage,
            new_gas_usage,
        };

        // when
        let res = comparator.get_diffs().expect("getting diffs failed");

        // then
        let mut iter = res.iter();
        assert_eq!(
            iter.next().expect("first diff entry must exist"),
            &Row {
                name: "not_existent_in_old_csv".to_string(),
                optimized_size,
                total_size: optimized_size,
                gas_usage: 0,
                total_gas_usage: 0,
            }
        );
        assert_eq!(iter.next(), None);
    }
}
