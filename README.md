# jasper




## Quick start

```shell
# stdin
zcat *.fastq.gz | jasper -p pattern1.list --db pattern.db


# file in
jasper -i test.fq.gz -o test_out -p cyclone_barcode.list --db example/pattern.db


# yt cluster command test
name=TB2000771F_jasper_err0.15_0.2_t10
file='/home/long_read/LRS/data/LRS_MTB/2024-3-8-MTB_p2/output_data/TB2000771F-202403071104161_read.fq.gz'
outdir=/home/long_read/LRS/data/LRS_MTB/2024-3-8-MTB_p2/$name

/home/long_read/LRS/software/jasper/jasper -i $file -t 10 --log_num 200000 -m 100 --pattern-errate 0.15,0.2 -p /home/long_read/LRS/software/jasper/example/cyclone_barcode.list -o $outdir

```

支持多线程：--threads

支持在不同pattern裁剪：--trim-n ,当trim-n>pattern次数时，保留全长

支持短片段过滤：--min-length

支持连续拆分：-p 输入多个pattern文件

支持每个pattern定义单双端： --pattern-match

支持多个pattern识别位置拆分：--pos

支持写出引物对组合or引物对类型：--write-type

支持设置不同错误率（可定义多个）：--pattern-errate --pattern-shift

支持前后引物校正：--pattern-maxdist





## build

```sh
# build with musl
cargo build --release --target x86_64-unknown-linux-musl
# build with system
cargo build --release
```
