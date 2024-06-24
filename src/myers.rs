use bio::alignment::Alignment;
use bio::pattern_matching::myers::MyersBuilder;

// pub struct SearchPattern<'a>{
    //     pub text: TextSlice<'a>,
    //     pub pattern: Vec<u8>,
    //     pub start: usize,
    //     pub end: usize,
    //     pub dist_ratio: f32,
// }
// impl SearchPattern {
//     pub fn new(text: TextSlice, pattern: Vec<u8>, start: usize, end: usize, dist_ratio: f32) -> SearchPattern {
//         SearchPattern {
    //             text,
    //             pattern,
    //             start,
//             end,
//             dist_ratio,
//         }
//     }
// }
#[derive(Debug)]
pub struct SearchPattern{
    pub raw_text: Vec<u8>,
    pub text: Vec<u8>,
    pub raw_text_len: usize,
    pub pattern: Vec<u8>, // search pattern
    pub dist_ratio: f32, // error ratio
    pub max_dist: u8, // max distance, already trimN
    pub start: usize,
    pub end: usize,
}
impl SearchPattern {
    pub fn new(raw_text: Vec<u8>, dist_ratio: f32) -> SearchPattern {
        SearchPattern {
            raw_text: raw_text.clone(),
            text: Vec::new(),
            raw_text_len: raw_text.len(),
            pattern: Vec::new(),
            dist_ratio: dist_ratio,
            max_dist: 0,
            start: 0,
            end: 0,
            
        }
    }
    pub fn update(&mut self, start_pos:usize, end_pos: usize, pattern: Vec<u8>) {
        let pattern_trim_n_len = String::from_utf8(pattern.clone()).unwrap().trim_matches('N').len() as f32;
        self.max_dist = (pattern_trim_n_len * self.dist_ratio).floor() as u8; 
        self.start = start_pos;
        self.end = end_pos;
        self.text = self.raw_text[self.start..self.end].to_vec();
        self.pattern = pattern;
    }
}

pub fn myers_best(input:&SearchPattern) -> Option<(i32, usize, usize,)>{
    let mut myers = MyersBuilder::new().ambig(b'N', b"ACGT").build_64(input.pattern.clone());
    let mut aln = Alignment::default();
    let mut matches = myers.find_all_lazy(&input.text, input.max_dist);
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
