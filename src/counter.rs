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
        let mut counter = HashMap::new();
        counter.insert("valid".to_string(), 0);
        counter.insert("total".to_string(), 0);
        counter.insert("filtered".to_string(), 0);
        counter.insert("unknown".to_string(), 0);
        counter.insert("fusion".to_string(), 0);
        CounterManager {
            counter: counter,
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
            let primer: String = readinfo.match_names[0].clone();
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
        let fusion = self.counter.get("fusion").unwrap_or(&0);
        let filterd = self.counter.get("filtered").unwrap_or(&0);
        let valid_rate = if *total > 0 {
            100 * *valid / *total
        } else {
            0
        };
        let fusion_rate = if *total > 0 {
            100 * *fusion / *total
        } else {
            0
        };
        let filterd_rate = if *total > 0 {
            100 * *filterd / *total
        } else {
            0
        };
        info!("process {}/{} reads (filtered/total), filtered rate: {:.2} %.", filterd, total, filterd_rate);
        info!("process {}/{} reads (fusion/total), fusion rate: {:.2} %.", fusion, total, fusion_rate);
        info!("process {}/{} reads (valid/total), valid rate: {:.2} %.", valid, total, valid_rate);
    }
    // pub fn write_total_info(&self) {
    //     let mut file = File::create(Path::new(&self.outdir).join("total_info.tsv")).expect("fail to create total_info.tsv");
    //     writeln!(file, "type\tcount").expect("fail to write header");
    //     for (read_type, count) in &self.counter {
    //         writeln!(file, "{}\t{}", read_type, count).expect("fail to write read info");
    //     }
    // }
    pub fn write_total_info(&self) {
        let total_reads = *self.counter.get("total").unwrap_or(&0) as f64;
        let valid_reads = *self.counter.get("valid").unwrap_or(&0) as f64;
        let unkown_reads = *self.counter.get("unknown").unwrap_or(&0) as f64;
        let filtered_reads = *self.counter.get("filtered").unwrap_or(&0) as f64;
        let fusion_reads = *self.counter.get("fusion").unwrap_or(&0) as f64;

        let valid_rate = if total_reads > 0.0 {
            valid_reads / total_reads * 100.0
        } else {
            0.0
        };
        let unkown_rate = if total_reads > 0.0 {
            unkown_reads / total_reads * 100.0
        } else {
            0.0
        };
        let filtered_rate = if total_reads > 0.0 {
            filtered_reads / total_reads * 100.0
        } else {
            0.0
        };
        let fusion_rate = if total_reads > 0.0 {
            fusion_reads / total_reads * 100.0
        } else {
            0.0
        };

        let mut file = File::create(Path::new(&self.outdir).join("total_info.tsv")).expect("fail to create total_info.tsv");
        writeln!(file, "total\tfiltered\tfiltered_rate\tfuison\tfusion_rate\tunkown\tunkown_rate\tvalid\tvalid_rate").expect("fail to write header");

        writeln!(file, "{}\t{}\t{:.2}%\t{}\t{:.2}%\t{}\t{:.2}%\t{}\t{:.2}%", 
            total_reads as u32, 
            filtered_reads as u32, 
            filtered_rate,
            fusion_reads as u32,
            fusion_rate,
            unkown_reads as u32,
            unkown_rate,
            valid_reads as u32, 
            valid_rate, 
        ).expect("fail to write total_info");
    }
}
