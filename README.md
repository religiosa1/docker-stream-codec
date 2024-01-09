# Docker Stream CLI encoder/decoder

Small CLI-utility for parsing and creation of
[multiplexed Docker Streams](https://docs.docker.com/engine/api/v1.43/#tag/Container/operation/ContainerAttach),
i.e. downloaded container logs. 

Intended usage -- inspection and testing.

It's comprised of two separate binaries:
- docker-stream-decoder
- docker-stream-encoder

## Building

Docker Stream Parser is written in Rust, you'll need a 
[Rust installation](https://www.rust-lang.org/tools/install) to compile it. 

```sh
git clone https://github.com/religiosa1/docker-stream-codec-cli.git
cd docker-stream-codec-cli
cargo build --release
```

## Running tests

```sh
cargo test --all
```

## Examples

### Decoder 

Parses multiplexed docker stream into each separate IO stream and writes them to
separate files.

```sh
# assuming you have downloaded logs into ./log.vdm

# getting the stdout from log.vdm and writing it to log.txt
docker-stream-decoder log.vdm > log.txt

# getting all multiplexed streams and redirecting them to their files
docker-stream-decoder log.vdm -i log.stdin.txt -o log.stdout.txt -r log.stderr.txt

# reading and concatenating multiple files
docker-stream-decoder log1.vdm log2.vdm log3.vdm

# don't try to recover from an error but immediately fail the process instead
# Can be usefull for validation of docker stream dumps 
docker-stream-decoder -f log1.vdm -o /dev/null
```

### Ecnoder

Generates a multiplexed docker stream from multiple input files, with either
fixed-size frames or randomly sized within the specified range. 

Input file for each next chunk is selected at random.

```sh
# Reading contents of log files, multiplexing it into a single stream and redirecting to log.vdm
docker-stream-decoder -i log.stdin.txt -o log.stdout.txt -r log.stderr.txt > log.vdm 

# Reading a single log file, splitting into chunks of fixed size of 250 bytes and writing to log.vdm
docker-stream-decoder -o log.stdout.txt -M250  -O log.vdm 

# Reading a single log files, splitting into chunks of random size from 200 to 250 (inclusive) bytes
docker-stream-decoder -i log.stdin.txt -m 200 -M250

```

## License

The Docker Stream CLI encoder/decoder is MIT licensed.