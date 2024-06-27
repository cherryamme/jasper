use csv;
use log::info;
use std::collections::HashMap;
use crate::args::Args;
use crate::utils::reverse_complement;


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
        let pattern_db = self.loading_pattern_db(pattern_db_file);
        self.loading_pattern(
            pattern_file,
            pattern_db,
        );
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





pub fn get_patterns(inputargs: &Args) -> PatternArgs {
    info!("loading pattern db file({})...", &inputargs.pattern_db_file);
    let mut patternargs = PatternArgs::new(inputargs);
    for i in 0..inputargs.pattern_files.len() {
        let mut patterndb = PatternDB::new();
        patterndb.get_pattern(&inputargs.pattern_db_file, &inputargs.pattern_files[i]);
        let patternarg = PatternArg {
            pattern_db: patterndb,
            pattern_pos: true,
            pattern_errate: patternargs.pattern_errate[i].clone(),
            pattern_maxdist: patternargs.pattern_maxdist[i].clone(),
            pattern_shift: patternargs.pattern_shift[i].clone(),
        };
        patternargs.pattern_vec.push(patternarg);
    };
    return patternargs;
}



#[test]
pub fn test(){
    use clap::Parser;
    let args = crate::args::Args::parse();
    let search_patterns = get_patterns(&args);
    info!("{:?}", search_patterns);
}
