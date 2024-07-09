# jasper




## Quick start

```shell
# stdin
zcat *.fastq.gz | jasper -p pattern1.list -d pattern.db


# file in
./jasper -i /home/jiangchen/project/jasper/temple.fq.gz -t 4 -o test_out -p example/cyclone_barcode.list --db example/pattern.db --id_sep "&"


# yt cluster command test
name=TB2000771F_jasper_err0.15_0.2_t10
file='/home/long_read/LRS/data/LRS_MTB/2024-3-8-MTB_p2/output_data/TB2000771F-202403071104161_read.fq.gz'
outdir=/home/long_read/LRS/data/LRS_MTB/2024-3-8-MTB_p2/$name

/home/long_read/LRS/software/jasper/jasper -i $file -t 10 -m 100 -e 0.15,0.2 -p /home/long_read/LRS/software/jasper/example/cyclone_barcode.list -o $outdir

```

支持多线程：-t/--threads

支持在不同pattern裁剪：--trim-n ,当trim-n>pattern次数时，保留全长,默认0,减去序列

支持短片段过滤：--min-length,过滤小片段

支持连续拆分：-p 输入多个pattern文件为连续拆分

支持每个pattern定义单双端： --match，如dual single为先双端再单端

支持多个pattern识别位置拆分：--pos，识别第一对pattern，在pattern前搜索后续pattern

支持写出引物对组合or引物对类型：--write-type，写出type还是引物对命名文件

支持设置不同错误率（可定义多个）：-e/--pattern-errate --pattern-shift

支持前后引物校正：--pattern-maxdist





## build

```sh
# build with musl
cargo build --release --target x86_64-unknown-linux-musl
# build with system
cargo build --release
```
