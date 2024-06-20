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
    pub treads: usize,
}
#[derive(Debug,Clone)]
pub struct PatternArg {
    pub pattern_db: PatternDB,      // search db
    pub pattern_match: String,  // single or dual
    pub pattern_pos: bool,          // use position or not
    // pub pattern_shift: usize,       // >0 means shift to right, <0 means shift to left
    pub pattern_errate: (f32, f32), // error rate for left and right
    pub pattern_maxdist: usize,     // max distance in matcher for left and right
}


#[derive(Debug,Clone)]
pub struct PatternDB {
    // patterns to find
    pub f_patterns: HashMap<String, String>,
    pub r_patterns: HashMap<String, String>,
    pub pattern_type: HashMap<String,(String, String, String)>,
}

fn loading_pattern_db(file: &str) -> HashMap<String, String> {
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

fn loading_pattern(file: &str, pattern_db: HashMap<String, String>) -> PatternDB {
    // initialize the search pattern
    let mut search_patterns = PatternDB {
        f_patterns: HashMap::new(),
        r_patterns: HashMap::new(),
        pattern_type: HashMap::new(),
    };
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
        search_patterns.f_patterns.insert(f.clone(), f_seq.clone());
        search_patterns.f_patterns.insert(r.clone(), r_seq.clone());
        search_patterns.r_patterns.insert(f.clone(), reverse_complement(&f_seq));
        search_patterns.r_patterns.insert(r.clone(), reverse_complement(&r_seq));
        search_patterns.pattern_type.insert(key_fs.clone(), (key_fs.clone(),name.clone(),"fs".to_string()));
        search_patterns.pattern_type.insert(key_rs.clone(), (key_fs.clone(),name.clone(),"rs".to_string()));
    }
    info!("loading pattern file success({})...", file);
    return search_patterns;
}

pub fn get_pattern(pattern_db_file: &String, pattern_file: &String) -> PatternDB{
    let pattern_db =
        loading_pattern_db(pattern_db_file);
    let patterndb = loading_pattern(
        pattern_file,
        pattern_db,
    );
    return patterndb;
}

pub fn get_patterns(inputargs: &Args) -> PatternArgs {
    info!("loading pattern db file({})...", &inputargs.pattern_db_file);
    let mut patternargs = PatternArgs {
        pattern_vec: Vec::new(),
        pattern_match: inputargs.pattern_match.clone(),
        window_size: inputargs.window_size.clone(),
        treads: inputargs.threads,
    };
    for i in 0..inputargs.pattern_files.len() {
        let patterndb = get_pattern(&inputargs.pattern_db_file, &inputargs.pattern_files[i]);
        let patternarg = PatternArg {
            pattern_db: patterndb,
            pattern_match: inputargs.pattern_match[i].clone(),
            pattern_pos: true,
            // pattern_shift: inputargs.pattern_shift[i].clone(),
            pattern_errate: inputargs.pattern_errate[i].clone(),
            pattern_maxdist: inputargs.pattern_maxdist[i].clone(),
        };
        patternargs.pattern_vec.push(patternarg);
    };
    return patternargs;
}