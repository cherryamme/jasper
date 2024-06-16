use crate::myers::myers_best;
use crate::myers::SearchPattern;
use std::collections::HashMap;
use bio::io::fastq::Record;
use crate::pattern::{PatternArg,PatternArgs};
use flume::Receiver;
use std::cmp::min;
use crate::fastq::ReadInfo;
use std::thread;

#[derive(Debug)]
struct ReadChunk {
    left: (usize, usize),
    right: (usize, usize),
}



#[derive(Debug)]
pub struct SplitType {
    pub patter_match: &'static str,
    pub pattern_name: String,
    pub pattern_type: String,
    pattern_strand: String,
    left_matcher: Matcher,
    right_matcher: Matcher,
}
impl SplitType {
    pub fn to_info(&self) -> String{
        return format!(
            "{}\t{}\t{}\t{}:({},{},{},{});({},{},{},{})",
            self.patter_match,
            self.pattern_name,
            self.pattern_type,
            self.pattern_strand,
            self.left_matcher.pattern,
            self.left_matcher.score,
            self.left_matcher.ystart,
            self.left_matcher.yend,
            self.right_matcher.pattern,
            self.right_matcher.score,
            self.right_matcher.ystart,
            self.right_matcher.yend,
        ).to_string();
    }
    
}

#[derive(Debug)]
pub struct Matcher {
    // single match score
    pattern: String,
    score: i32,
    ystart: usize,
    yend: usize,
    status: bool,
}

fn find_matcher(pattern: &HashMap<String, String>, search_pattern: &mut SearchPattern, orient: &'static str) -> Matcher {
    let mut matcher = Matcher {
        pattern: String::from(""),
        score: 99,
        ystart: 0,
        yend: 0,
        status: false,
    };
    for (key, value) in pattern.iter() {
        search_pattern.pattern = value.as_bytes().to_vec();
        if search_pattern.pattern_pos && orient == "left" {
            search_pattern.start =  search_pattern.end.checked_sub(search_pattern.pattern.len() + 6).unwrap_or(0);
        } else if search_pattern.pattern_pos && orient == "right" {
            search_pattern.end = min(search_pattern.text_len, search_pattern.start + search_pattern.pattern.len() + 6 );
        }
        // debug!("search_pattern: {:?}", search_pattern);
        let result = myers_best(&*search_pattern);
        // debug!("get result: {:?}", result);
        if result.is_none() {
            // debug!("no match found continue");
            continue;
        } else if result.unwrap().0 < matcher.score {
            matcher.pattern = key.to_string();
            matcher.score = result.unwrap().0;
            matcher.ystart = result.unwrap().1;
            matcher.yend = result.unwrap().2;
            matcher.status = true;
            // debug!("get better matcher: {:?}", matcher);
        }
    }
    matcher
}

fn anno_pattern_type(
    left_matcher: &Matcher,
    right_matcher: &Matcher,
    pattern_type_dict: &HashMap<String, (String, String, String)>,
    pattern_maxdist: i32,
    // exact_match: bool
) -> (&'static str, String, String, String) {
    let mut pattern_name = String::from("unknown");
    let mut pattern_type = String::from("unknown");
    let mut pattern_strand = String::from("unknown");
    let (patter_match, key) = get_match_key(left_matcher, right_matcher, pattern_maxdist);
    for (dict_key, value) in pattern_type_dict {
        if dict_key.contains(&key) {
            pattern_name = value.0.clone();
            pattern_type = value.1.clone();
            pattern_strand = value.2.clone();
            break;
        }
    }

    (patter_match, pattern_name, pattern_type, pattern_strand)
}

fn get_match_key(
    left_matcher: &Matcher,
    right_matcher: &Matcher,
    pattern_maxdist: i32,
) -> (&'static str, String) {
    let score_diff = right_matcher.score - left_matcher.score;
    if score_diff.abs() <= pattern_maxdist {
        return (
            "dual",
            format!("{}_{}", left_matcher.pattern, right_matcher.pattern),
        );
    }
    if score_diff > 0 {
        ("left", format!("{}_", left_matcher.pattern))
    } else {
        ("right", format!("_{}", right_matcher.pattern))
    }
}

fn splitter(record: &Record, readchunk: &ReadChunk, patternarg1: &PatternArg) -> SplitType {
    let patterndb = &patternarg1.pattern_db;
    let mut search_pattern = SearchPattern {
        dist_ratio: patternarg1.pattern_errate.0,
        text: &record.seq(),
        pattern: Vec::new(),
        text_len: record.seq().len(),
        start: readchunk.left.0,
        end: readchunk.left.1,
        pattern_pos: patternarg1.pattern_pos,
    };
    let left_matcher = find_matcher(&patterndb.f_patterns, &mut search_pattern,"left");
    // search right pattern
    search_pattern.start = readchunk.right.0;
    search_pattern.end = readchunk.right.1;
    search_pattern.dist_ratio = patternarg1.pattern_errate.1;
    // debug!("search text is {:?}",String::from_utf8(search_pattern.text.clone()));
    let right_matcher = find_matcher(&patterndb.r_patterns, &mut search_pattern,"right");
    // right_matcher start and end need to be adjusted
    // right_matcher.ystart += readchunk.right.0;
    // right_matcher.yend += readchunk.right.0;
    // debug!("left matcher: {:?}", left_matcher);
    // debug!("right matcher: {:?}", right_matcher);
    let (patter_match, pattern_name, pattern_type, pattern_strand) =
        anno_pattern_type(&left_matcher, &right_matcher, &patterndb.pattern_type, 10);

    let split_type = SplitType {
        patter_match: patter_match,
        pattern_name: pattern_name,
        pattern_type: pattern_type,
        pattern_strand: pattern_strand,
        left_matcher: left_matcher,
        right_matcher: right_matcher,
    };
    // debug!("read1: {:?}", split_type);
    return split_type
}

pub fn splitter_vec(record: &Record, patternargs: &PatternArgs) -> Vec<SplitType>{
    let mut split_type_vec = Vec::new();
    let mut readchunk = ReadChunk {
        left: (0, patternargs.window_size[0]),
        right: (record.seq().len() - patternargs.window_size[1], record.seq().len()),
    };
    for patternarg in patternargs.pattern_vec.iter() {
        // let pattern_db = &patternarg.pattern_db;
        // let pattern_match = &patternarg.pattern_match;
        // let pattern_pos = patternarg.pattern_pos;
        // let pattern_shift = patternarg.pattern_shift;
        // let pattern_errate = patternarg.pattern_errate;
        // let pattern_maxdist = patternarg.pattern_maxdist;
        let split_type = splitter(&record, &readchunk, &patternarg);
        // debug!("split_type: {:?}", split_type);
        if patternarg.pattern_pos {
            let left = split_type.left_matcher.ystart.clone();
            let right = split_type.right_matcher.yend.clone();
        
            if split_type.left_matcher.status {
                readchunk.left = (left+3,left+3);
            }
        
            if split_type.right_matcher.status {
                readchunk.right = (right-3 ,right-3 );
            }
        }
        split_type_vec.push(split_type);
    }
    return split_type_vec
}



pub fn splitter_receiver(rrx: Receiver<ReadInfo>, patternargs: &PatternArgs, threads: usize) -> (Receiver<ReadInfo>,Vec<thread::JoinHandle<()>>) {
	let (stx, srx) = flume::unbounded();
	let mut handles = vec![];
	for _ in 0..threads {
		let rrx = rrx.clone();
		let stx = stx.clone();
		let patternargs = patternargs.clone();
		let handle = thread::spawn(move || {
			for readinfo in rrx.iter() {
				let mut matched_reads = readinfo;
				matched_reads.split_type_vec =  splitter_vec(&matched_reads.record, &patternargs );
                matched_reads.to_name(patternargs.pattern_match.clone());
				// info!("read1: {}", matched_reads.to_tsv());
				stx.send(matched_reads).expect("splitter send error");
			}
		});
		handles.push(handle);
	};
	
	(srx,handles)
}