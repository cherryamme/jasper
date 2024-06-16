mod args;
mod pattern;
mod utils;
mod fastq;
mod myers;
mod splitter;
mod writer;
use clap::Parser;
use std::path::PathBuf;
use lazy_static::lazy_static;
use writer::WriterManager;
use utils::ProcessInfo;
lazy_static! {
    static ref ARGS:args::Args = args::Args::parse();
}

fn main() {
    pretty_env_logger::init();
    // info!("{:?}", ARGS);
    let search_patterns = pattern::get_patterns(&ARGS);
    // debug!("{:?}", search_patterns);
    // let reader = fastq::FastqFilesReader::new(args.inputs.clone());
    let path = PathBuf::from("/mnt/c/Users/Administrator/Desktop/rust_learn/jasper/example/barcode21.fastq.gz");
    let rrx = fastq::spawn_reader(path);
    let (srx,handles) = splitter::splitter_receiver(rrx, &search_patterns, ARGS.threads);
    let mut logger: Vec<String> = Vec::new();
    let mut counter = fastq::ReadCounter::new();
    let mut writer_manager = WriterManager::new(ARGS.outdir.clone()).expect("build writer manager fail");
    // let mut readsinfo = HashMap::new();
    let mut processinfo = ProcessInfo::new(1000);
    for readinfo in srx {
        //将readinfo.tsv()写入文件ARG.output, 需要使用GzEncoder写出为gz文件
        // splitter::splitter_logger(&readinfo, &ARGS.output);
        logger.push(readinfo.to_tsv());
        // info!("read to_name: {:?}", readinfo.read_names);
        counter.counter_read(&readinfo.read_names);
        // readsinfo.insert(readinfo.record.id().to_string(), readinfo);
        writer_manager.write(&readinfo).expect("writing readinfo fail");
        processinfo.info();
    }
    counter.write_to_tsv(&ARGS.outdir).expect("writer split_info fail");
    // splitter::splitter_logger(&readinfo, &mut logger);
    writer::write_log_file(logger, &ARGS.outdir).expect("writer read_log fail");

    for handle in handles {
        handle.join().expect("Error joining thread");
    }
}