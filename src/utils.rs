use log::info;

pub fn reverse_complement(seq: &str) -> String {
    let mut complement = vec![' '; seq.len()];
    for (i, c) in seq.chars().enumerate() {
        complement[seq.len() - 1 - i] = match c {
            'A' => 'T',
            'T' => 'A',
            'C' => 'G',
            'G' => 'C',
            _ => panic!("Invalid character: {}", c),
        };
    }
    complement.into_iter().collect::<String>()
}


pub struct ProcessInfo {
    start_time: std::time::Instant,
    end_time: std::time::Instant,
    process_num: u32,
    info_num: u32,
}
impl ProcessInfo {
    pub fn new(info_num: u32) -> ProcessInfo {
        ProcessInfo {
            start_time: std::time::Instant::now(),
            end_time: std::time::Instant::now(),
            process_num: 0,
            info_num: info_num,
        }
    }
    pub fn info(&mut self){
        self.process_num+=1;
        if self.process_num % self.info_num ==0 {
            self.end_time = std::time::Instant::now();
            let elapsed = self.end_time.duration_since(self.start_time);
            let rate = self.process_num as f64 / elapsed.as_secs_f64();
            info!("Processed {} reads. speed: {:.2} reads/s", self.process_num, rate);
            self.start_time = std::time::Instant::now();
            self.process_num = 0;
        }
    }
    
}