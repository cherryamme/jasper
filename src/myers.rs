use bio::alignment::{Alignment,TextSlice};
use bio::pattern_matching::myers::MyersBuilder;
use log::warn;

#[warn(unused_imports)]
#[warn(unused_variables)]
#[derive(Debug)]
pub struct SearchPattern<'a>{
    pub dist_ratio: f32,
    pub text: TextSlice<'a>,
    pub text_len: usize,
    pub pattern: Vec<u8>,
    pub start: usize,
    pub end: usize,
    pub pattern_pos: bool,
}


pub fn myers_best(input:&SearchPattern) -> Option<(i32, usize, usize,)>{
    let pattern_trim_n_len = String::from_utf8(input.pattern.clone()).unwrap().trim_matches('N').len() as f32;
    let max_dist =  (pattern_trim_n_len * input.dist_ratio).floor() as u8; 
    let mut myers = MyersBuilder::new().ambig(b'N', b"ACGT").build_64(input.pattern.clone());
    let mut aln = Alignment::default();
    let text = input.text[input.start..input.end].to_owned();
    let mut matches = myers.find_all_lazy(&text, max_dist);
    // first, find the best hit
    match matches.by_ref().min_by_key(|&(_, dist)| dist) {
        Some((best_end, _)) => {
            matches.alignment_at(best_end, &mut aln);
            // println!("{}", aln.pretty(&input.pattern, &input.text, 80));
            Some((aln.score, aln.ystart+input.start, aln.yend+ input.start))
        },
        None => None,
    }
}
