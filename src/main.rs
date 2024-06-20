mod args;
mod pattern;
mod utils;
mod fastq;
mod myers;
mod splitter;
mod writer;
use clap::Parser;
use log::info;
use std::path::PathBuf;
use writer::WriterManager;
use utils::ProcessInfo;

fn main() {
    pretty_env_logger::init();
    let comands: Vec<String> = std::env::args().collect();
    let args = args::Args::parse();
    info!("{:?}", comands);
    // debug!("{:?}", args);
    let search_patterns = pattern::get_patterns(&args);
    // debug!("{:?}", search_patterns);
    // let reader = fastq::FastqFilesReader::new(args.inputs.clone());
    let path = PathBuf::from(&args.inputs[0]);
    // info!("Create fq.gz reader handler");
    let rrx = fastq::spawn_reader(path);
    // info!("Create fq.gz spliter handler");
    let srx = splitter::splitter_receiver(rrx, &search_patterns, args.threads);
    let mut logger: Vec<String> = Vec::new();
    let mut counter = fastq::ReadCounter::new();
    let mut writer_manager = WriterManager::new(args.outdir.clone()).expect("build writer manager fail");
    // let mut readsinfo = HashMap::new();
    let mut processinfo = ProcessInfo::new(args.log_num.clone());

    info!("Writing reads fq.gz");
    for readinfo in srx {
        //将readinfo.tsv()写入文件ARG.output, 需要使用GzEncoder写出为gz文件
        // splitter::splitter_logger(&readinfo, &ARGS.output);
        logger.push(readinfo.to_tsv());
        // info!("read to_name: {:?}", readinfo.read_names);
        counter.counter_read(&readinfo.read_names);
        writer_manager.write(readinfo).expect("writing readinfo fail");
        processinfo.info();
    }
    info!("Writing split info to tsv");
    counter.write_to_tsv(&args.outdir).expect("writer split_info fail");
    // splitter::splitter_logger(&readinfo, &mut logger);
    info!("Writing logger to reads_log.gz");
    writer::write_log_file(logger, &args.outdir).expect("writer read_log fail");
    
    info!("Ending threads");

}