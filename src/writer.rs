use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use flate2::write::GzEncoder;
use flate2::Compression;
use log::info;
use std::io::Result;
use std::path::Path;
// use bio::io::fastq::Writer;
use std::fs::create_dir_all;
use crate::fastq::ReadInfo;
use std::io::BufWriter;
use std::thread;
use flume::{Receiver, Sender, unbounded};
pub struct WriterManager {
    writers: HashMap<String, Sender<ReadInfo>>,
    outdir: String,
    pub logger: Vec<String>,
    handles: Vec<thread::JoinHandle<()>>,
}

impl WriterManager {
    pub fn new(outdir: String) -> WriterManager {
        info!("Creating writer manager, start writing...");
        WriterManager {
            writers: HashMap::new(),
            outdir,
            logger: Vec::new(),
            handles: Vec::new(),
        }
    }

    pub fn write(&mut self, readinfo: ReadInfo) -> Result<()> {
        if !readinfo.write_to_fq {
            return Ok(());
        }
        let outfile = readinfo.outfile.clone();
        if !self.writers.contains_key(&outfile) {
            let (tx, rx) = unbounded();
            let filepath = Path::new(&self.outdir).join(format!("{}.fq.gz", outfile));
            let filedir = filepath.parent().unwrap();
            create_dir_all(&filedir).expect("fail to create output directory");
            let file = File::create(&filepath).expect("fail to create output fq.gz");
            let encoder = GzEncoder::new(file, Compression::default());
            let writer = BufWriter::with_capacity(1_000_000, encoder);
            self.start_writing_thread(writer, rx);
            self.writers.insert(outfile.clone(), tx);
        }
        self.writers.get(&outfile).unwrap().send(readinfo).expect("readinfo to writer send fail");
        Ok(())
    }

    fn start_writing_thread(&mut self, mut writer: BufWriter<GzEncoder<File>>, rx: Receiver<ReadInfo>) {
        let handle = thread::spawn(move || {
            for readinfo in rx.iter() {
                let id =  readinfo.out_record.id();
                let seq = std::str::from_utf8(readinfo.out_record.seq()).expect("Not a valid UTF-8 sequence");
                let qual = std::str::from_utf8(readinfo.out_record.qual()).expect("Not a valid UTF-8 sequence");
                let record_str = format!("@{}\n{}\n+\n{}\n", id, seq, qual);
                write!(writer, "{}", record_str).unwrap();
            }
        });
        self.handles.push(handle);
    }

    pub fn write_log_file(&self, outdir: &String) -> Result<()> {
        let dir_path = Path::new(outdir);
        create_dir_all(&dir_path)?;
        info!("Writing logger to reads_log.gz");
        let file_path = dir_path.join("reads_log.gz");
        let file = File::create(file_path)?;
        let mut encoder = GzEncoder::new(file, Compression::default());
        for line in &self.logger {
            encoder.write_all(line.as_ref())?;
            encoder.write_all(b"\n")?;
        }
        let _ = encoder.finish()?;
        Ok(())
    }
    pub fn drop(&mut self) {
        // When the `Sender`s are dropped, the corresponding writing threads will receive a `Disconnected` error and exit.
        info!("Writing fastq.gz. May cost some time..");
        self.writers.clear();
        // Wait for all writing threads to finish.
        for handle in self.handles.drain(..) {
            handle.join().expect("Writing thread panicked");
        }
    }
}