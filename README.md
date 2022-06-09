# Quartz
## Quartz is a simple messenger based on [libquartz](https://github.com/Bultek/libquartz)

## Usage
```shell
$ quartz-messenger -h
$ quartz-messenger send -i <server_index> -t <to> -f <from> -m <message>
$ quartz-messenger get -i <server_index> -a <as>
```

## Installation via cargo (git)
```shell
$ git clone https://github.com/mrquantumoff/quartz
$ cd quartz
$ cargo install --path . 
```

## Installation via cargo (crates.io)
```shell
$ cargo install quartz-messenger
```