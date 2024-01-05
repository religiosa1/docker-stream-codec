# Docker Stream Parser

Small CLI-utility for parsing 
[multiplexed Docker Streams](https://docs.docker.com/engine/api/v1.43/#tag/Container/operation/ContainerAttach),
i.e. downloaded container logs.

## Building

Docker Stream Parser is written in Rust, you'll need a 
[Rust installation](https://www.rust-lang.org/tools/install) to compile it. 

```sh
git clone https://github.com/religiosa1/docker_stream_parser.git
cd docker_stream_parser
cargo build --release
```

## Running tests

```sh
cargo test --all
```

## Examples

```sh
# assuming you have downloaded logs into ./log.vdm

# getting the stdout from log.vdm and writing it to log.txt
docker_stream_parser log.vdm > log.txt

# getting all multiplexed streams and redirecting them to their files
docker_stream_parser log.vdm -i log.stdin.txt -o log.stdout.txt -r log.stderr.txt

# reading and concatenating multiple files
docker_stream_parser log1.vdm log2.vdm log3.vdm

# don't try to recover from an error but immediately fail the process instead
# Can be usefull for validation of docker stream dumps 
docker_stream_parser -f log1.vdm -o /dev/null

```

## License

The parsing utility is MIT licensed.