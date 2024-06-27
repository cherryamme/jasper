mod args;
mod pattern;
mod utils;
mod counter;
mod fastq;
mod myers;
mod splitter;
mod writer;
use clap::Parser;
use log::{info,debug};
use utils::ProcessInfo;

fn main() {
    std::env::set_var("RUST_LOG", "info");
    pretty_env_logger::init();
    let comands: Vec<String> = std::env::args().collect();
    let args = args::Args::parse();
    info!("Run Command: {:?}", comands);
    // debug!("{:?}", args);
    let search_patterns = pattern::get_patterns(&args);
    // debug!("{:?}", search_patterns);
    let start_time = std::time::Instant::now();
    // info!("Create fq.gz reader handler");
    let rrx: flume::Receiver<fastq::ReadInfo> = fastq::spawn_reader(args.inputs);
    // info!("Create fq.gz spliter handler");
    let srx = splitter::splitter_receiver(rrx, &search_patterns, args.threads);
    let mut counter_manager = counter::CounterManager::new(args.outdir.clone());
    let mut writer_manager = writer::WriterManager::new(args.outdir.clone());
    // let mut readsinfo = HashMap::new();
    let mut processinfo = ProcessInfo::new(args.log_num.clone());

    for readinfo in srx {
        //将readinfo.tsv()写入文件ARG.output, 需要使用GzEncoder写出为gz文件
        // splitter::splitter_logger(&readinfo, &ARGS.output);
        
        writer_manager.logger.push(readinfo.to_tsv());
        // info!("read to_name: {:?}", readinfo.read_names);
        counter_manager.counter_read(&readinfo);
        writer_manager.write(readinfo).expect("writing readinfo fail");
        processinfo.info();
    }
    // splitter::splitter_logger(&readinfo, &mut logger);
    writer_manager.write_log_file(&args.outdir).expect("writer read_log fail");
    counter_manager.write_valid_info();
    debug!("counter_manager: {:?}", counter_manager.counter);
    let mut elapsed_time = start_time.elapsed();
    counter_manager.info();
    info!("Succes split! Time elapsed: {:.4?}", elapsed_time);
    writer_manager.drop();
    elapsed_time = start_time.elapsed();
    info!("All done! Total time elapsed: {:.4?}", elapsed_time);
}
