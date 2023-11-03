# Install

## Build from source

`git clone https://gitlab.inria.fr/coccinelle/coccinelleforrust && cd coccinelleforrust`

`cargo build --release`

`cp target/release/cfr ~/.local/bin/`

This will build the cfr binary and make it available for the user.

# Usage

cfr [OPTIONS] --coccifile <COCCIFILE> <TARGETPATH>

COCCIFILE - Is the Semantic Patch file
TARGETPATH - Can be a file or directory

Example - `cfr -c patch.cocci myproject/src/`

## Options
  -c, --coccifile <COCCIFILE>
          Path of Semantic Patch File path
  -o, --output <OUTPUT>
          Path of transformed file path
  -r, --rustfmt-config <RUSTFMT_CONFIG>
          rustfmt config file path
  -i, --ignore <IGNORE>
          [default: ]
  -d, --debug
          
      --apply
          
      --suppress-diff
          
      --suppress-formatting
          
      --no-parallel
          
      --worth-trying <WORTH_TRYING>
          strategy for identifying files that may be matched by the semantic patch 
          [default: cocci-grep] 
          [possible values: no-scanner, grep, git-grep, cocci-grep]
  -h, --help
          Print help
  -V, --version
          Print version