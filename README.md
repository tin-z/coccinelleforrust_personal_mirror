Not my project, ref to: [https://gitlab.inria.fr/coccinelle/coccinelleforrust](https://gitlab.inria.fr/coccinelle/coccinelleforrust)


-----

# README

## Introduction

Coccinelle is a tool for automatic program matching and transformation that
was originally developed for making large scale changes to the Linux kernel
source code (ie, C code).  Matches and transformations are driven by
user-specific transformation rules having the form of abstracted patches,
referred to as semantic patches. As the Linux kernel, and systems software
more generally, is starting to adopt Rust, we are developing Coccinelle for
Rust, to make the power of Coccinelle available to Rust codebases.

## Install

### Build from source

`git clone https://gitlab.inria.fr/coccinelle/coccinelleforrust && cd coccinelleforrust`

`cargo build --release`

`cp target/release/cfr ~/.local/bin/`

This will build the cfr binary and make it available for the user.

## Usage

    cfr [OPTIONS] -c <COCCIFILE> <TARGETPATH>


COCCIFILE - Is the Semantic Patch file

TARGETPATH - Can be a file or directory

Example - `cfr -c patch.cocci myproject/src/`

### Options

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

## Joining the mailing list

Email `subscribe cfr FirstName LastName` to `sympa_inria@inria.fr` to join the cfr mailing list.

## Contributing:

Contributions are welcome.  Please sign your contributions, according to
the following text extracted from Documentation/SubmittingPatches.txt of
the Linux kernel:

The sign-off is a simple line at the end of the explanation for the
patch, which certifies that you wrote it or otherwise have the right to
pass it on as an open-source patch.  The rules are pretty simple: if you
can certify the below:

        Developer's Certificate of Origin 1.1

        By making a contribution to this project, I certify that:

        (a) The contribution was created in whole or in part by me and I
            have the right to submit it under the open source license
            indicated in the file; or

        (b) The contribution is based upon previous work that, to the best
            of my knowledge, is covered under an appropriate open source
            license and I have the right under that license to submit that
            work with modifications, whether created in whole or in part
            by me, under the same open source license (unless I am
            permitted to submit under a different license), as indicated
            in the file; or

        (c) The contribution was provided directly to me by some other
            person who certified (a), (b) or (c) and I have not modified
            it.

	(d) I understand and agree that this project and the contribution
	    are public and that a record of the contribution (including all
	    personal information I submit with it, including my sign-off) is
	    maintained indefinitely and may be redistributed consistent with
	    this project or the open source license(s) involved.

then you just add a line saying

	Signed-off-by: Random J Developer <random@developer.example.org>

using your real name (sorry, no pseudonyms or anonymous contributions.)
