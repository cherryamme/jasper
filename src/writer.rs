use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use flate2::write::GzEncoder;
use flate2::Compression;
use std::io::Result;
use std::path::Path;
use std::fs::create_dir_all;
use crate::fastq::ReadInfo;
use std::io::BufWriter;

pub struct WriterManager {
    writers:  HashMap<String, BufWriter<GzEncoder<File>>>,
    outdir: String,
}

impl WriterManager {
    pub fn new(outdir: String) -> std::io::Result<Self> {
        create_dir_all(&outdir)?; // create the output directory if it does not exist
        Ok(WriterManager {
            writers: HashMap::new(),
            outdir,
        })
    }

    // pub async fn write(&mut self, readinfo: ReadInfo) -> Result<(), Box<dyn std::error::Error>> {
    //     let file_key = readinfo.read_names.join("_");
    //     if !self.writers.contains_key(&file_key) {
    //         let file = File::create(Path::new(&self.outdir).join(format!("{}.fq.gz", file_key))).await?;
    //         let encoder = GzEncoder::new(file, Compression::default());
    //         let writer = BufWriter::with_capacity(10_000_000, encoder);
    //         self.writers.insert(file_key.clone(), writer);
    //     }                                                                            
    //     let id = format!("{}%{}",readinfo.record.id(),file_key);
    //     let seq = std::str::from_utf8(readinfo.record.seq()).expect("Not a valid UTF-8 sequence");
    //     let qual = std::str::from_utf8(readinfo.record.qual()).expect("Not a valid UTF-8 sequence");
    //     let record_str = format!("@{}\n{}\n+\n{}\n", id, seq, qual);
        
    //     if let Some(writer) = self.writers.get_mut(&file_key) {
    //         writer.write_all(record_str.as_bytes()).await?;
    //     }
        
    //     Ok(())
    // }
    pub fn write(&mut self, readinfo: ReadInfo) -> Result<()> {
        let file_key = readinfo.read_names.join("_");
        if !self.writers.contains_key(&file_key) {
            let file = File::create(Path::new(&self.outdir).join(format!("{}.fq.gz", file_key)))?;
            let encoder = GzEncoder::new(file, Compression::default());
            let writer = BufWriter::with_capacity(10_000_000, encoder);
            self.writers.insert(file_key.clone(), writer);
        }                                                                            
        let id = format!("{}%{}",readinfo.record.id(),file_key);
        let seq = std::str::from_utf8(readinfo.record.seq()).expect("Not a valid UTF-8 sequence");
        let qual = std::str::from_utf8(readinfo.record.qual()).expect("Not a valid UTF-8 sequence");
        let record_str = format!("@{}\n{}\n+\n{}\n", id, seq, qual);
        write!(self.writers.get_mut(&file_key).unwrap(), "{}", record_str)?;
        Ok(())
    }
    // pub fn multithread_write(&mut self, readinfo: ReadInfo) -> Result<()> {
        //     let file_key = readinfo.read_names.join("_");
        //     let outdir = self.outdir.clone();
        //     let writer_manager = Arc::clone(&self.writers);
        
        //     if !writer_manager.lock().unwrap().contains_key(&file_key) {
            //         let file = File::create(Path::new(&outdir).join(format!("{}.fq.gz", file_key))).unwrap();
    //         let encoder = GzEncoder::new(file, Compression::default());
    //         let writer = BufWriter::with_capacity(10_000_000, encoder);
    //         writer_manager.lock().unwrap().insert(file_key.clone(), writer);
    //     }
    //     let writer_manager_cloned = Arc::clone(&writer_manager);
    //     thread::spawn(move || {
        //         let id = format!("{}%{}",readinfo.record.id(),file_key);
        //         let seq = std::str::from_utf8(readinfo.record.seq()).expect("Not a valid UTF-8 sequence");
    //         let qual = std::str::from_utf8(readinfo.record.qual()).expect("Not a valid UTF-8 sequence");
    //         let record_str = format!("@{}\n{}\n+\n{}\n", id, seq, qual);

    //         write!(writer_manager_cloned.lock().unwrap().get_mut(&file_key).unwrap(), "{}", record_str).unwrap();
    //     });

    //     Ok(())
    // }
}


pub fn write_log_file(logger: Vec<String>, outdir: &String) -> Result<()> {
    let dir_path = Path::new(outdir);
    create_dir_all(&dir_path)?;
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