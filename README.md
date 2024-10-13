nbt-json
========

![version 0.2.0](https://shields.io/badge/version-0.2.0-orange)
![GitHub Actions Workflow Status](https://img.shields.io/github/actions/workflow/status/ms0503/nbt-json/rust.yml)

nbt-json is a simple converter for Minecraft's NBT and JSON.

# Installing

```
$ cargo install --git https://github.com/ms0503/nbt-json.git
```

# Using

```
$ nbt-json input.dat                   # NBT -> JSON, out: input.json
$ nbt-json input.dat -o output.json    # NBT -> JSON, out: output.json
$ nbt-json input.json                  # JSON -> NBT, out: input.dat
$ nbt-json input.json -o output.dat    # JSON -> NBT, out: output.dat
```

By default, no processing is performed if the output file already exists.  
If you wish to overwrite the file, use the `-f`/`--force` option.

```
$ nbt-json input.dat -f                   # NBT -> JSON, out: input.json even if it already exists
$ nbt-json input.dat -o output.json -f    # NBT -> JSON, out: output.json even if it already exists
$ nbt-json input.json -f                  # JSON -> NBT, out: input.dat even if it already exists
$ nbt-json input.json -o output.dat -f    # JSON -> NBT, out: output.dat even if it already exists
```
