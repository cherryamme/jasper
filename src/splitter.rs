use crate::fastq::ReadInfo;
use crate::myers::myers_best;
use crate::myers::SearchPattern;
use crate::pattern::{PatternArg, PatternArgs};
use bio::io::fastq::Record;
use flume::Receiver;
use log::info;
use std::cmp::min;
use std::collections::HashMap;
use std::thread;
use std::time::Instant;

#[derive(Debug)]
struct ReadChunk {
    left: usize,
    right: usize,
    pos_mut: bool,
}
impl ReadChunk {
    pub fn new(patternargs: &PatternArgs, readinfo: &ReadInfo) -> Self {
        let left = if patternargs.window_size[0] > readinfo.read_len {
            0
        } else {
            patternargs.window_size[0]
        };

        let right = if patternargs.window_size[1] > readinfo.read_len {
            0
        } else {
            readinfo.read_len - patternargs.window_size[1]
        };

        ReadChunk {
            left: left,
            right: right,
            pos_mut: false,
        }
    }
}

#[derive(Debug)]
pub struct SplitType {
    pub patter_match: &'static str, // single or dual
    pub pattern_name: String,       // pattern name ex:4.2-F_3.7-R
    pub pattern_type: String,       // pattern type ex:alpha
    pub pattern_strand: String,         // strand orientation
    pub left_matcher: Matcher,          // matcher
    pub right_matcher: Matcher,         // matcher
}
impl SplitType {
    pub fn new(left_matcher: Matcher, right_matcher: Matcher) -> Self {
        SplitType {
            patter_match: "unknown",
            pattern_name: String::from("unknown"),
            pattern_type: String::from("unknown"),
            pattern_strand: String::from("unknown"),
            left_matcher: left_matcher,
            right_matcher: right_matcher,
        }
    }
    pub fn to_info(&self) -> String {
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
        )
        .to_string();
    }
    pub fn anno_pattern_type(
        &mut self,
        pattern_type_dict: &HashMap<String, (String, String, String)>,
        pattern_maxdist: i32,
    ) -> (){
        let (patter_match, key) =
            self.get_match_key(pattern_maxdist);
            if key == "_".to_string() || key == "unknown".to_string(){
                return;
            }
            for (dict_key, value) in pattern_type_dict {
                if dict_key.contains(&key) {
                    self.patter_match = patter_match;
                    self.pattern_name = value.0.clone();
                    self.pattern_type = value.1.clone();
                    self.pattern_strand = value.2.clone();
                break;
            }
        }
    }
    pub fn get_match_key(
        &self,
        pattern_maxdist: i32,
    ) -> (&'static str, String) {
        if self.right_matcher.status && self.left_matcher.status {
            let score_diff = self.right_matcher.score - self.left_matcher.score;
            if score_diff.abs() <= pattern_maxdist {
                return (
                    "dual",
                    format!("{}_{}", self.left_matcher.pattern, self.right_matcher.pattern),
                );
            }
            if score_diff > 0 {
                ("left", format!("{}_", self.left_matcher.pattern))
            } else {
                ("right", format!("_{}", self.right_matcher.pattern))
            }
        }else if self.right_matcher.status {
            return (
                "right",
                format!("_{}", self.right_matcher.pattern),
            )
        }else if self.left_matcher.status {
            return (
                "left",
                format!("{}_", self.left_matcher.pattern),
            );
        }else {
            return (
                "unknown",
                String::from("unknown"));
        }

    }
}

#[derive(Debug)]
pub struct Matcher {
    // single match score
    pattern: String,
    score: i32,
    pub ystart: usize,
    pub yend: usize,
    status: bool,
}
impl Matcher {
    pub fn new() -> Self {
        Matcher {
            pattern: String::from(""),
            score: 99,
            ystart: 0,
            yend: 0,
            status: false,
        }
    }
}

fn calculate_start_end(
    start: usize,
    end: usize,
    pattern_shift: usize,
    pattern_len: usize,
    text_len: usize,
    orient: &'static str,
) -> (usize, usize) {
    let mut new_start = start;
    let mut new_end = end;
    match orient {
        "left" => {
            new_start = end.checked_sub(pattern_len).and_then(|x| x.checked_sub(pattern_shift)).unwrap_or(0);
            new_end = min(text_len, new_end + pattern_shift);
        }
        "right" => {
            new_end = min(text_len, new_start + pattern_len + pattern_shift);
            new_start = start.checked_sub(pattern_shift).unwrap_or(0);
        }
        _ => {}
    };
    (new_start, new_end)
}

fn find_matcher(
    rawstart: usize,
    rawend: usize,
    patterndb: &HashMap<String, String>,
    search_pattern: &mut SearchPattern,
    mut_pos: bool,
    pattern_shift: usize,
    orient: &'static str,
) -> Matcher {
    let mut matcher = Matcher::new();
    for (key, value) in patterndb.iter() {
        let pattern = value.as_bytes().to_vec();
        let (start_pos, end_pos) = if mut_pos {
            calculate_start_end(
                rawstart,
                rawend,
                pattern_shift,
                pattern.len(),
                search_pattern.raw_text_len,
                orient,
            )
        } else {
            (rawstart, rawend)
        };
        search_pattern.update(start_pos, end_pos, pattern);

        // debug!("search_pattern: {:?}", search_pattern);
        let result = myers_best(&search_pattern);
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

fn splitter(record: &Record, readchunk: &ReadChunk, patternarg1: &PatternArg) -> SplitType {
    let patterndb = &patternarg1.pattern_db;
    let mut search_pattern =
        SearchPattern::new(record.seq().to_vec(), patternarg1.pattern_errate.0);
    let left_matcher = find_matcher(
        0,
        readchunk.left,
        &patterndb.f_patterns,
        &mut search_pattern,
        readchunk.pos_mut,
        patternarg1.pattern_shift,
        "left",
    );
    // search right pattern
    search_pattern.dist_ratio = patternarg1.pattern_errate.1;
    // debug!("search text is {:?}",String::from_utf8(search_pattern.text.clone()));
    let right_matcher = find_matcher(
        readchunk.right,
        record.seq().len(),
        &patterndb.r_patterns,
        &mut search_pattern,
        readchunk.pos_mut,
        patternarg1.pattern_shift,
        "right",
    );
    // debug!("left matcher: {:?}", left_matcher);
    // debug!("right matcher: {:?}", right_matcher);
    let mut split_type = SplitType::new(left_matcher, right_matcher);
    split_type.anno_pattern_type(&patterndb.pattern_type, 10);
    // debug!("read1: {:?}", split_type);
    return split_type;
}

pub fn splitter_vec(readinfo: &ReadInfo, patternargs: &PatternArgs) -> Vec<SplitType> {
    let mut split_type_vec = Vec::new();
    let mut readchunk = ReadChunk::new(patternargs, readinfo);
    for patternarg in patternargs.pattern_vec.iter() {
        let split_type = splitter(&readinfo.record, &readchunk, &patternarg);
        // debug!("split_type: {:?}", split_type);
        if patternarg.pattern_pos
            && split_type.left_matcher.status
            && split_type.right_matcher.status
        {
            // let right_bound: usize = if right <= record.seq().len() - 30 { right + 30 } else { record.seq().len() };

            readchunk.left = split_type.left_matcher.ystart.clone();
            readchunk.right = split_type.right_matcher.yend.clone();
            readchunk.pos_mut = true
        } else {
            readchunk = ReadChunk::new(patternargs, readinfo);
        }
        split_type_vec.push(split_type);
    }
    return split_type_vec;
}

pub fn splitter_receiver(
    rrx: Receiver<ReadInfo>,
    patternargs: &PatternArgs,
    threads: usize,
) -> Receiver<ReadInfo> {
    let (stx, srx) = flume::unbounded();
    for t in 0..threads {
        let start_time = Instant::now();
        let rrx = rrx.clone();
        let stx = stx.clone();
        let patternargs = patternargs.clone();
        thread::spawn(move || {
            let mut read_count = 0;
            for mut readinfo in rrx.iter() {
                readinfo.split_type_vec = splitter_vec(&readinfo, &patternargs);
                // get split_type_vec annotation
                readinfo.update(&patternargs.pattern_match,&patternargs.write_type,patternargs.trim_n, patternargs.min_length);
                // info!("read1: {}", matched_reads.to_tsv());
                stx.send(readinfo).expect("splitter send error");
                read_count += 1;
            }
            let elapsed_time = start_time.elapsed();
            info!(
                "threads {} process {} reads. Time elapsed: {:.4?}",
                t, read_count, elapsed_time
            )
        });
    }
    srx
}
