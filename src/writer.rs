use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use flate2::write::GzEncoder;
use flate2::Compression;
use log::info;
use std::io::Result;
use std::path::Path;
use std::fs::create_dir_all;
use crate::fastq::ReadInfo;
use std::io::BufWriter;
use std::thread;
use flume::{Receiver, Sender, unbounded};
pub struct WriterManager {
    writers: HashMap<String, Sender<ReadInfo>>,
    outdir: String,
}

impl WriterManager {
    pub fn new(outdir: String) -> std::io::Result<Self> {
        create_dir_all(&outdir)?; // create the output directory if it does not exist
        info!("Creating writer manager, start writing...");
        Ok(WriterManager {
            writers: HashMap::new(),
            outdir,
        })
    }

    pub fn write(&mut self, readinfo: ReadInfo) -> Result<()> {
        let file_key = readinfo.read_names.join("_");
        if !self.writers.contains_key(&file_key) {
            let (tx, rx) = unbounded();
            let file = File::create(Path::new(&self.outdir).join(format!("{}.fq.gz", file_key)))?;
            let encoder = GzEncoder::new(file, Compression::default());
            let writer = BufWriter::with_capacity(10_000_000, encoder);
            self.start_writing_thread(writer, rx);
            self.writers.insert(file_key.clone(), tx);
        }
        self.writers.get(&file_key).unwrap().send(readinfo).expect("readinfo to writer send fail");
        Ok(())
    }

    fn start_writing_thread(&self, mut writer: BufWriter<GzEncoder<File>>, rx: Receiver<ReadInfo>) {
        thread::spawn(move || {
            for readinfo in rx.iter() {
                let id = format!("{}%{}", readinfo.record.id(), readinfo.read_names.join("_"));
                let seq = std::str::from_utf8(readinfo.record.seq()).expect("Not a valid UTF-8 sequence");
                let qual = std::str::from_utf8(readinfo.record.qual()).expect("Not a valid UTF-8 sequence");
                let record_str = format!("@{}\n{}\n+\n{}\n", id, seq, qual);
                write!(writer, "{}", record_str).unwrap();
            }
        });
    }
}


pub fn write_log_file(logger: Vec<String>, outdir: &String) -> Result<()> {
    let dir_path = Path::new(outdir);
    create_dir_all(&dir_path)?;
    info!("Writing logger to reads_log.gz");
    let file_path = dir_path.join("reads_log.gz");
    let file = File::create(file_path)?;
    let mut encoder = GzEncoder::new(file, Compression::default());
    for line in logger {
        encoder.write_all(line.as_ref())?;
        encoder.write_all(b"\n")?;
    }
    let _ = encoder.finish()?;
    Ok(())
}


// pub fn write_fq_gz(readinfo: ReadInfo, outdir: &String, encoders: &HashMap<String, GzEncoder<File>) {
//     let file_key = readinfo.read_names.join("_");
//     let pattern = readinfo.read_names.clone();
//     let mut path = PathBuf::from(outdir);
//     for part in pattern.into_iter().take(pattern.len() - 1) {
//         path.push(part);
//     }
//     create_dir_all(&path)?;
//     let filename = pattern.last().unwrap();
//       //你需要自己实现这个函数，以根据数据确定应该写入哪个文件
//     let encoder = encoders.entry(file_key.clone())
//         .or_insert_with(|| {
//             let f = File::create(file_name).unwrap();
//             GzEncoder::new(f, Compression::default())
//         });

// }