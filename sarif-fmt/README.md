[![Workflow Status](https://github.com/psastras/sarif-rs/workflows/main/badge.svg)](https://github.com/psastras/sarif-rs/actions?query=workflow%3A%22main%22)

# sarif-fmt

This crate provides a command line tool to pretty print SARIF files to easy
human readable output.

The latest [documentation can be found here](https://docs.rs/sarif_fmt).

SARIF or the Static Analysis Results Interchange Format is an industry standard
format for the output of static analysis tools. More information can be found on
the official website:
[https://sarifweb.azurewebsites.net/](https://sarifweb.azurewebsites.net/).

## Installation

`sarif-fmt` may be insalled via `cargo`

```shell
cargo install sarif-fmt
```

## Usage

For most cases, simply pipe a SARIF file into `sarif-fmt`
(`cat ./foo.sarif | sarif-fmt`)

## Example

```shell
$ cargo clippy --message-format=json | clippy-sarif | sarif-fmt
$ warning: using `Option.and_then(|x| Some(y))`, which is more succinctly expressed as `map(|x| y)`
    ┌─ sarif-fmt/src/bin.rs:423:13
    │
423 │ ╭             the_rule
424 │ │               .full_description
425 │ │               .as_ref()
426 │ │               .and_then(|mfms| Some(mfms.text.clone()))
    │ ╰───────────────────────────────────────────────────────^
    │
    = `#[warn(clippy::bind_instead_of_map)]` on by default
      for further information visit https://rust-lang.github.io/rust-clippy/master#bind_instead_of_map
```

Often it is useful to record the SARIF file for machine processing but also
print the nicely formatted results to stdout at the same time. This can be done
using the `tee` command:

```shell
$ clang-tidy -checks=cert-* cpp.cpp -- | clang-tidy-sarif | tee clang-tidy.sarif | sarif-fmt
$ 2 warnings generated.
warning: 'atoi' used to convert a string to an integer value, but function will not report conversion errors; consider using 'strtol' instead [cert-err34-c]
  ┌─ /home/psastras/repos/sarif-rs/sarif-fmt/tests/data/cpp.cpp:4:10
  │
4 │   return atoi(num);
  │          ^^^^^^^^^^

warning: calling 'system' uses a command processor [cert-env33-c]
  ┌─ /home/psastras/repos/sarif-rs/sarif-fmt/tests/data/cpp.cpp:8:3
  │
8 │   system("ls");
  │   ^^^^^^^^^^^^^

$ cat clang-tidy.sarif
{
  "runs": [
    {
      "results": [
        {
          "level": "warning",
          "locations": [
            {
              "physicalLocation": {
                "artifactLocation": {
                  "uri": "cpp.cpp"
                },
                "region": {
                  "startColumn": 10,
                  "startLine": 4
                }
              }
            }
          ],
          "message": {
            "text": "'atoi' used to convert a string to an integer value, but function will not report conversion errors; consider using 'strtol' instead [cert-err34-c]"
          }
        },
        {
          "level": "warning",
          "locations": [
            {
              "physicalLocation": {
                "artifactLocation": {
                  "uri": "cpp.cpp"
                },
                "region": {
                  "startColumn": 3,
                  "startLine": 8
                }
              }
            }
          ],
          "message": {
            "text": "calling 'system' uses a command processor [cert-env33-c]"
          }
        }
      ],
      "tool": {
        "driver": {
          "name": "clang-tidy"
        }
      }
    }
  ],
  "version": "2.1.0"
}
```

License: MIT
