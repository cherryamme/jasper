use crate::splitter::SplitType;
use bio::io::fastq::{Reader, Record};
use flate2::read::MultiGzDecoder;
use flume::{unbounded, Sender, Receiver};
use log::info;
use std::ffi::OsStr;
use std::{
    fs::File,
    io::{BufReader, Read},
    path::PathBuf,
};
use std::time::Instant;
use std::collections::HashSet;

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
        let readinfo = ReadInfo::new(record.unwrap());
        rtx.send(readinfo).expect("Error sending");
    }
}






#[derive(Debug)]
pub struct ReadInfo {
    pub record: Record,
    pub split_type_vec: Vec<SplitType>,
    pub outfile: String,
    pub strand_orient: String,
    pub read_type: String,
    pub match_types: Vec<String>,
    pub match_names: Vec<String>,
    pub record_id: String,
    pub write_to_fq: bool,
    pub out_record: Record,
    pub read_len: usize,
}
impl ReadInfo {
    pub fn new(record: Record) -> ReadInfo {
        let readinfo = ReadInfo {
            record: record.clone(),
            split_type_vec: Vec::new(),
            outfile: String::new(),
            strand_orient: String::from("unknown"),
            read_type: String::from("valid"),
            match_types: Vec::new(),
            match_names: Vec::new(),
            record_id: String::new(),
            write_to_fq: false,
            out_record: Record::new(),
            read_len: record.seq().len(),
        };
        readinfo
    }
    pub fn update(&mut self, pattern_match: &Vec<String>, write_type: &String, trim_n: usize,  min_length: usize) {
        self.update_match_names(pattern_match);
        self.update_out_filename(write_type);
        self.update_read_type(min_length,trim_n);
        // debug!("read1: {}", self.to_tsv());
        // debug!("read1_self: {:?}", self);
        self.update_write_to_fq(trim_n);
    }   
    fn update_match_names(&mut self,pattern_match: &Vec<String>){
        let mut strand_values: Vec<String> = Vec::new();
        for (i, split_type) in self.split_type_vec.iter().enumerate() {
            match pattern_match.get(i) {
                // "dual" < "single" < "unknown"
                Some(element) if element >= &String::from(split_type.patter_match) => {
                    self.match_types.push(split_type.pattern_type.clone());
                    self.match_names.push(split_type.pattern_name.clone());
                }
                _ => {
                    self.match_types.push(String::from("unknown"));
                    self.match_names.push(String::from("unknown"));
                    self.read_type = "unknown".to_string();
                }
            }
            strand_values.push(split_type.pattern_strand.clone());

        }
        while self.match_names.len() <3 {
            self.match_names.push(String::from("default"));
        }
        while self.match_types.len() <3 {
            self.match_types.push(String::from("default"));
        }
        let mut unique_values: HashSet<_> = strand_values.drain(..).collect();
        unique_values.remove("unknown");
        if unique_values.len() == 1 {
            self.strand_orient = unique_values.into_iter().next().unwrap();
        }
    }
    fn update_out_filename(&mut self, write_type: &String){
        if write_type == "type" {
            let mut reversed_names = self.match_types.clone();
            reversed_names.reverse();
            self.outfile = reversed_names.join("/");
            self.record_id = self.match_types.join("%")
        } else {
            let mut reversed_names = self.match_names.clone();
            reversed_names.reverse();
            self.outfile = reversed_names.join("/");
            self.record_id = self.match_names.join("%")
        }

    }
    fn update_read_type(&mut self, min_length: usize, trim_n: usize){
        if self.read_len <= min_length {
            self.read_type = "filtered".to_string();
        }
        let cutleft = self.split_type_vec[trim_n].left_matcher.ystart;
        let mut cutright = self.split_type_vec[trim_n].right_matcher.yend;
        if cutright == 0 {
            cutright = self.read_len
        }
        if cutleft > cutright {
            self.read_type = "unknown".to_string();
            self.write_to_fq = false;
        }
    }
    fn update_write_to_fq(&mut self,trim_n: usize) {
        if self.read_type == "valid" {
            self.write_to_fq = true;
            let cutleft;
            let mut cutright;
            if trim_n < self.split_type_vec.len() {
                cutleft = self.split_type_vec[trim_n].left_matcher.ystart;
                cutright = self.split_type_vec[trim_n].right_matcher.yend;
            } else {
                cutleft = 0;
                cutright = self.read_len;
            }
            if cutright == 0 {
                cutright = self.read_len
            }
            self.out_record= Record::with_attrs(&format!("{}%{}%{}", self.record.id(),self.strand_orient,self.record_id), None, &self.record.seq()[cutleft..cutright], &self.record.qual()[cutleft..cutright]);
        }
    }
    pub fn to_tsv(&self) -> String {
        let mut split_type_info =
            String::from(format!("{}\t{}", self.record.id(), self.read_len));
        for split_type in self.split_type_vec.iter() {
            split_type_info += format!("\t{}", split_type.to_info(),).as_str();
        }
        return split_type_info;
    }
    // pub fn filter_read
}
