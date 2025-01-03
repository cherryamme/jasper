use csv;
use log::info;
use std::collections::HashMap;
use crate::args::Args;
use crate::utils::reverse_complement;
use age::secrecy::SecretString;
use std::fs::File;
use std::io::{Read, Write};

#[derive(Debug,Clone)]
pub struct PatternArgs {
    pub window_size: Vec<usize>,
    pub pattern_match: Vec<String>,
    pub pattern_vec: Vec<PatternArg>,
    pub trim_n: usize,
    pub write_type: String,
    pub pattern_errate: Vec<(f32, f32)>,
    pub pattern_maxdist: Vec<usize>,
    pub pattern_shift: Vec<usize>,
    pub min_length: usize,
    pub id_sep: String,
    pub fusion_db: FusionDB,
    pub fusion_errate: f32,
}
impl PatternArgs {
    pub fn new(inputargs: &Args) -> PatternArgs {
        let mut p = PatternArgs {
            window_size: inputargs.window_size.clone(),
            pattern_match: inputargs.pattern_match.clone(),
            pattern_vec: vec![],
            trim_n: inputargs.trim_n,
            write_type: inputargs.write_type.clone(),
            pattern_errate: inputargs.pattern_errate.clone(),
            pattern_maxdist: inputargs.pattern_maxdist.clone(),
            pattern_shift: inputargs.pattern_shift.clone(),
            min_length: inputargs.min_length,
            id_sep: inputargs.id_sep.clone(),
            fusion_db: FusionDB::new(),
            fusion_errate: inputargs.fusion_errate,
        };
        p.fix_vec();
        return p;
    }
    pub fn fix_vec(&mut self) {
        PatternArgs::resize_to_min_length(&mut self.pattern_match, 5);
        PatternArgs::resize_to_min_length(&mut self.pattern_errate, 5);
        PatternArgs::resize_to_min_length(&mut self.pattern_maxdist, 5);
        PatternArgs::resize_to_min_length(&mut self.pattern_shift, 5);
    }
    fn resize_to_min_length<T: Clone + Default>(vec: &mut Vec<T>, min_length: usize) {
        if vec.len() < min_length {
            let last_elem = vec.last().cloned().unwrap_or_default();
            vec.resize(min_length, last_elem);
        }
    }
}

#[derive(Debug,Clone)]
pub struct PatternArg {
    pub pattern_db: PatternDB,      // search db
    pub pattern_pos: bool,          // use position or not
    // pub pattern_shift: usize,       // >0 means shift to right, <0 means shift to left
    pub pattern_errate: (f32, f32), // error rate for left and right
    pub pattern_maxdist: usize,     // max distance in matcher for left and right
    pub pattern_shift: usize
}

pub fn encrypt_pattern_db(file: &str, passphrase: &str) {
    // Read the file content
    let mut f = File::open(file).expect(&format!("no such file ({}) found", file));
    let mut content = Vec::new();
    f.read_to_end(&mut content).unwrap();

    // Encrypt the content
    let passphrase = SecretString::from(passphrase.to_owned());
    let recipient = age::scrypt::Recipient::new(passphrase.clone());
    let encrypted_data = age::encrypt(&recipient, &content).unwrap();

    // Write the encrypted content to a new file with .age suffix
    let output_file = format!("{}.safe", file);
    let mut out = File::create(&output_file).unwrap();
    out.write_all(&encrypted_data).unwrap();
    info!("Encrypted pattern db file saved to {}", output_file);
}



#[derive(Debug,Clone)]
pub struct PatternDB {
    // patterns to find
    pub f_patterns: HashMap<String, String>,
    pub r_patterns: HashMap<String, String>,
    pub pattern_type: HashMap<String,(String, String, String)>,
}

impl PatternDB {
    fn new() -> PatternDB {
        PatternDB {
            f_patterns: HashMap::new(),
            r_patterns: HashMap::new(),
            pattern_type: HashMap::new(),
        }
    }
    pub fn get_pattern(&mut self, pattern_db_file: &String, pattern_file: &String){
        let pattern_db = self.loading_pattern_db(pattern_db_file,"666666");
        self.loading_pattern(
            pattern_file,
            pattern_db,
        );
    }
    fn loading_pattern_db(&self, file: &str, passphrase: &str) -> HashMap<String, String> {
        let mut pattern_db = HashMap::new();
        let mut content = Vec::new();

        if file.ends_with(".safe") {
            // Decrypt the file
            let passphrase = SecretString::from(passphrase.to_owned());
            let identity = age::scrypt::Identity::new(passphrase);
            let mut encrypted_file = File::open(file).expect(&format!("no such file({}) found", file));
            encrypted_file.read_to_end(&mut content).unwrap();
            let decrypted_data = age::decrypt(&identity, &content[..]).unwrap();
            content = decrypted_data;
        } else {
            // Read the file directly
            let mut f = File::open(file).expect(&format!("no such file({}) found", file));
            f.read_to_end(&mut content).unwrap();
        }

        let cursor = std::io::Cursor::new(content);
        let mut rdr = csv::ReaderBuilder::new()
            .has_headers(false)
            .delimiter(b'\t')
            .from_reader(cursor);

        for result in rdr.records() {
            let recored = result.unwrap();
            let name = &recored[0];
            let seq = &recored[1];
            pattern_db.insert(name.to_string(), seq.to_string());
        }
        return pattern_db;
    }
    fn loading_pattern(&mut self, file: &str, pattern_db: HashMap<String, String>){
        //loading tsv file
        // let file = File::open(file).unwrap();
        let mut rdr = csv::ReaderBuilder::new()
            .has_headers(true)
            .delimiter(b'\t')
            .from_path(file)
            .expect(&format!("no such file({}) found", file));
        for result in rdr.records() {
            let record = result.unwrap();
            let (f, r, name) = (record[0].to_string(), record[1].to_string(), record[2].to_string());
            let key_fs = format!("{}_{}",f,r);
            let key_rs = format!("{}_{}",r,f);
            let f_seq = pattern_db
                .get(&f)
                .expect(&format!("no such pattern({}) in pattern_db", f))
                .to_string();
            let r_seq = pattern_db
                .get(&r)
                .expect(&format!("no such pattern({}) in pattern_db", r))
                .to_string();
            self.f_patterns.insert(f.clone(), f_seq.clone());
            self.f_patterns.insert(r.clone(), r_seq.clone());
            self.r_patterns.insert(f.clone(), reverse_complement(&f_seq));
            self.r_patterns.insert(r.clone(), reverse_complement(&r_seq));
            if key_fs != key_rs {
                self.pattern_type.insert(key_fs.clone(), (key_fs.clone(),name.clone(),"fs".to_string()));
                self.pattern_type.insert(key_rs.clone(), (key_fs.clone(),name.clone(),"rs".to_string()));
            }else {
                self.pattern_type.insert(key_fs.clone(), (key_fs.clone(),name.clone(),"unknown".to_string()));
            }
        }
        info!("loading pattern file success({})...", file);
    }
}

#[derive(Debug,Clone)]
pub struct FusionDB {
    pub fusion_db: HashMap<String, String>,
}    // fusion patterns to find
impl FusionDB {
fn new() -> FusionDB {
    FusionDB {
        fusion_db: HashMap::new(),
    }
}
pub fn is_empty(&self) -> bool {
    self.fusion_db.is_empty()
}

fn loading_pattern_db(&self, file: &str) -> HashMap<String, String> {
    //loading pattern db file
    let mut pattern_db = HashMap::new();
    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(false)
        .delimiter(b'\t')
        .from_path(file)
        .expect(&format!("no such file({}) found", file));
    // info!("loading pattern db file success({})...", file);
    for result in rdr.records() {
        let recored = result.unwrap();
        let name = &recored[0];
        let seq = &recored[1];
        pattern_db.insert(name.to_string(), seq.to_string());
    }
    // debug!("pattern_db is {:?}", pattern_db);
    return pattern_db;
}
fn loading_pattern(&mut self, file: &str, pattern_db: HashMap<String, String>){
    //loading tsv file
    // let file = File::open(file).unwrap();
    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(true)
        .delimiter(b'\t')
        .from_path(file)
        .expect(&format!("no such file({}) found", file));
    for result in rdr.records() {
        let record = result.unwrap();
        let fusion_pattern = record[0].to_string();
        let fusion_seq = pattern_db
            .get(&fusion_pattern)
            .expect(&format!("no such fusion pattern({}) in pattern", fusion_pattern))
            .to_string();
        self.fusion_db.insert(fusion_pattern.clone(), fusion_seq.clone());
    }
}    
pub fn get_pattern(&mut self, pattern_db_file: &String, pattern_file: &String){
    let pattern_db = self.loading_pattern_db(pattern_db_file);
    self.loading_pattern(
        pattern_file,
        pattern_db,
    );
}
}

pub fn get_patterns(inputargs: &Args) -> PatternArgs {
    info!("loading pattern db file({})...", inputargs.pattern_db_file.as_ref().unwrap());
    let mut patternargs = PatternArgs::new(inputargs);

    let mut fusion_db = FusionDB::new();
    if &inputargs.fusion_file != "" {
        fusion_db.get_pattern(&inputargs.pattern_db_file.as_ref().unwrap(), &inputargs.fusion_file);
        patternargs.fusion_db = fusion_db;
    }
    for i in 0..inputargs.pattern_files.as_ref().unwrap().len() {
        let mut patterndb = PatternDB::new();
        patterndb.get_pattern(&inputargs.pattern_db_file.as_ref().unwrap(), &inputargs.pattern_files.as_ref().unwrap()[i]);
        let patternarg = PatternArg {
            pattern_db: patterndb,
            pattern_pos: inputargs.pattern_pos,
            pattern_errate: patternargs.pattern_errate[i].clone(),
            pattern_maxdist: patternargs.pattern_maxdist[i].clone(),
            pattern_shift: patternargs.pattern_shift[i].clone(),
        };
        patternargs.pattern_vec.push(patternarg);
    };
    return patternargs;
}



#[test]
pub fn test0(){
    use clap::Parser;
    let args = crate::args::Args::parse();
    let search_patterns = get_patterns(&args);
    info!("{:?}", search_patterns);
}
#[test]
pub fn test1(){
    pretty_env_logger::init();
    // use clap::Parser;
    // let args = crate::args::Args::parse();
    let mut patterndb = PatternDB::new();
    let db = "/home/jiangchen/project/jasper/example/pattern.db".to_string();
    let file = "/home/jiangchen/project/jasper/example/primer.list".to_string();
    patterndb.get_pattern(&db, &file);
    info!("{:?}", patterndb);
}





#[test]
pub fn test2(){
    pretty_env_logger::init();
    // use clap::Parser;
    // let args = crate::args::Args::parse();
    let mut fusion_db = FusionDB::new();
    let db = "/home/jiangchen/project/jasper/example/pattern.db".to_string();
    let file = "/home/jiangchen/project/jasper/example/fusion.list".to_string();
    fusion_db.get_pattern(&db, &file);
    info!("{:?}", fusion_db);
}
