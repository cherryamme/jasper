use std::collections::HashMap;
use std::fs::File;
use std::path::Path;
use log::info;
use crate::fastq::ReadInfo;
use std::io::Write;

pub struct CounterManager {
    pub counter: HashMap<String, u32>,
    pub validname_counter: HashMap<String, HashMap<String, HashMap<String, u32>>>,
    pub validtype_counter: HashMap<String, HashMap<String, HashMap<String, u32>>>,
    outdir: String,
}
impl CounterManager {
    pub fn new(outdir: String) -> CounterManager {
		info!("Creating counter manager, start counting...");
        CounterManager {
            counter: HashMap::new(),
            validname_counter: HashMap::new(),
            validtype_counter: HashMap::new(),
            // names: vec!["total".to_string(),"filtered".to_string(), "unknown".to_string(), "valid".to_string()],
            outdir: outdir,
        }
    }
    pub fn counter_read(&mut self, readinfo: &ReadInfo) {
        *self.counter.entry("total".to_string()).or_insert(0) += 1;
        *self.counter.entry(readinfo.read_type.clone()).or_insert(0) += 1;
        if readinfo.read_type == "valid" {
            let primer = readinfo.match_names[0].clone();
            let index = readinfo.match_names[1].clone();
            let barcode = readinfo.match_names[2].clone();
            let primer_type = readinfo.match_types[0].clone();
            let index_type = readinfo.match_types[1].clone();
            let barcode_type = readinfo.match_types[2].clone();
            let barcode_map = self.validname_counter.entry(barcode).or_insert_with(HashMap::new);
            let barcodetype_map = self.validtype_counter.entry(barcode_type).or_insert_with(HashMap::new);
            let index_map = barcode_map.entry(index).or_insert_with(HashMap::new);
            let indextype_map = barcodetype_map.entry(index_type).or_insert_with(HashMap::new);
            *indextype_map.entry(primer_type).or_insert(0) += 1;
            *index_map.entry(primer).or_insert(0) += 1;
        }
    }
    pub fn write_valid_info(&self) {
        for (barcode, index_map) in &self.validname_counter {
            let mut file = File::create(Path::new(&self.outdir).join(format!("{}_validname.tsv",barcode))).expect("fail to create valid_info.tsv");
            writeln!(file, "barcode\tindex\tprimer\tcount").expect("fail to write header");
            for (index, primer_map) in index_map {
                for (primer, count) in primer_map {
                    writeln!(file, "{}\t{}\t{}\t{}", barcode, index, primer, count).expect("fail to write valid_info");
                }
            }
        }
        for (barcode, index_map) in &self.validtype_counter {
            let mut file = File::create(Path::new(&self.outdir).join(format!("{}_validtype.tsv",barcode))).expect("fail to create valid_info.tsv");
            writeln!(file, "barcode\tindex\tprimer\tcount").expect("fail to write header");
            for (index, primer_map) in index_map {
                for (primer, count) in primer_map {
                    writeln!(file, "{}\t{}\t{}\t{}", barcode, index, primer, count).expect("fail to write valid_info");
                }
            }
        }
    }
    pub fn info(&self){
        let valid = self.counter.get("valid").unwrap_or(&0);
        let total = self.counter.get("total").unwrap_or(&0);
        let valid_rate = if *total > 0 {
            100 * *valid / *total
        } else {
            0
        };
        info!("process {}/{} reads (valid/total), valid rate: {:.2} %.", valid, total, valid_rate);
    }

}
