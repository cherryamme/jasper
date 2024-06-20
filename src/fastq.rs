use crate::splitter::SplitType;
use bio::io::fastq::{Reader, Record};
use flate2::read::MultiGzDecoder;
use flume::{unbounded, Sender, Receiver};
use log::info;
use std::ffi::OsStr;
use std::{
    fs::File,
    io::{BufReader, Read, Write},
    path::PathBuf,
};
use std::time::Instant;
use std::fs::create_dir_all;
use std::path::Path;
const BUFSIZE: usize = 10 * 1024 * 1024;

fn is_gz(path: &PathBuf) -> bool {
    match path.extension().and_then(OsStr::to_str) {
        Some(ext) => ext == "gz",
        None => false,
    }
}

pub fn spawn_reader(files: Vec<String>) -> Receiver<ReadInfo> {
    let (rtx, rrx) = unbounded();
    std::thread::spawn(move || {
        let start_time = Instant::now();

        if files.is_empty() {
            info!("no input file, loading from stdin...");
            let stdin_handle = std::io::stdin();
            process_file(stdin_handle, &rtx, None);
        } else {
            for file in files {
                let path = PathBuf::from(&file);
                if path.exists() {
                    let raw_handle = File::open(&path)
                        .expect(format!("Error opening input: {}", path.display()).as_str());
                    process_file(raw_handle, &rtx, Some(path));
                } else {
                    panic!("File {} does not exist", path.display());
                }
            }
        }

        let elapsed_time = start_time.elapsed();
        info!("Loading Reads data done! Time elapsed: {:.4?}", elapsed_time)
    });
    rrx
}

fn process_file<R: Read + 'static>(handle: R, rtx: &Sender<ReadInfo>, path: Option<PathBuf>) {
    let buf_handle = BufReader::with_capacity(BUFSIZE, handle);
    let maybe_decoder_handle = {
        if let Some(path) = path {
            if is_gz(&path) {
                info!("loading gzip file:{:?}", path);
                Box::new(MultiGzDecoder::new(buf_handle)) as Box<dyn Read>
            } else {
                info!("loading fastq file:{:?}", path);
                Box::new(buf_handle) as Box<dyn Read>
            }
        } else {
            Box::new(buf_handle) as Box<dyn Read>
        }
    };
    let fastq_reader = Reader::new(maybe_decoder_handle);
    for record in fastq_reader.records() {
        let readinfo = ReadInfo {
            record: record.unwrap(),
            split_type_vec: Vec::new(),
            read_names: Vec::new(),
            read_name: Vec::new(),
        };
        rtx.send(readinfo).expect("Error sending");
    }
}






#[derive(Debug)]
pub struct ReadInfo {
    pub record: Record,
    pub split_type_vec: Vec<SplitType>,
    pub read_names: Vec<String>,
    pub read_name: Vec<String>,
}
impl ReadInfo {
    pub fn to_tsv(&self) -> String {
        let mut split_type_info =
            String::from(format!("{}\t{}", self.record.id(), self.record.seq().len()));
        for split_type in self.split_type_vec.iter() {
            split_type_info += format!("\t{}", split_type.to_info(),).as_str();
        }
        return split_type_info;
    }
    pub fn to_name(&mut self, pattern_match: Vec<String>) {
        let mut result_vec: Vec<String> = Vec::new();
        for (i, split_type) in self.split_type_vec.iter().enumerate() {
            match pattern_match.get(i) {
                Some(element) if element >= &String::from(split_type.patter_match) => {
                    result_vec.push(split_type.pattern_type.clone());
                }
                _ => {
                    result_vec.push(String::from("unknown"));
                }
            }
        }
        self.read_names = result_vec.clone();
        // result_vec
    }
}

use std::collections::HashMap;
pub struct ReadCounter {
    pub counter: HashMap<String, u32>,
    pub names: Vec<String>,

}
impl ReadCounter {
    pub fn new() -> ReadCounter {
        ReadCounter {
            counter: HashMap::new(),
            names: vec!["total".to_string(), "valid".to_string(), "unknown".to_string()],
        }
    }
    pub fn counter_read(&mut self, pattern_match: &Vec<String>) {
        *self.counter.entry("total".to_string()).or_insert(0) += 1;
        let key = match pattern_match.contains(&"unknown".to_string()) {
            true => "unknown".to_string(),
            false => {
                *self.counter.entry("valid".to_string()).or_insert(0) += 1;
                pattern_match.join("_")
            }
        };
        if !self.names.contains(&key) {
            self.names.push(key.clone());
        }
        *self.counter.entry(key).or_insert(0) += 1;
    }

    pub fn write_to_tsv(&self, outdir: &String) -> std::io::Result<()> {
        let dir_path = Path::new(outdir);
        create_dir_all(&dir_path)?;
        info!("Writing split info to tsv");
    
        let file_path = dir_path.join("split_info.tsv");
        let mut file = File::create(file_path)?;
        write!(file, "{}\n", self.names.join("\t"))?;
        // Write keys and values
        for name in &self.names {
            if let Some(value) = self.counter.get(name) {
                write!(file, "{}\t", value)?;
            }
        }
        write!(file, "\n")?;
        Ok(())
    }
}