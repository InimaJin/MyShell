# MyShell is a simple shell.

*And until I come up with a more distinguishable name, I'll stick with the resourcefulness of MyShell (=Your shell).*

Just like any other shell, MyShell allows you to run applications and interact with your system via text commands.

## Features

While still fairly unstable in some regards, it does provide the basic functionality you'd expect from a shell, such as:

### Output redirection

MyShell supports the usage of pipes to chain commands efficiently:

    > ls -la | wc -l

The standard output of a process may also be written to files:

*Overwrite textfile*

    > ls ~ > textfile

*Append to the end of textfile*

    > echo "The above are the contents of my home directory" >> textfile

### Nested commands

Allowing for the insertion of a command's standard output into another command's arguments:
    
    > echo "You can call me" ${whoami}

### Command history

Issued commands are stored in ~/.config/myshell/history

Traversing the history is possible by means of the up/down arrow keys.

## How to build

Build the application with Cargo, Rust's build system, by issuing the following command:

    cargo build

Alternatively, build and subsequently execute it:

    cargo run

You may also wish to start the shell from the resulting binary itself (located in target/debug or target/release), without Cargo as an intermediary, after building.
