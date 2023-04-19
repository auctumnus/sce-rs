# sce-rs

Rust implementation of [SCE]. Intended to be compatible with
[the current python implementation] and its [documentation][py-docs].

## Usage

From Rust code:

```sh
$ cargo install sce
```

The CLI isn't made yet, but eventually this should be possible.

WASM is also planned.

## Compatability with other SCE's

Compatability with existing SCE implementations is paramount such that sound changes
can be used across different implementations. However, given only one currently
exists, the compatability goals of this implementation are to be a _superset_
of allowed / working rules. This means, for instance, a rule that might error
in SCE would still be allowed to parse, but might not run, or might have an
effect. However, no rules from existing rulesets will fail to run.

We are also, thus, not error-compatible.

## License

[NVPLv7+]; however, the codebase that this is nominally a port of is [MIT]
(along with the [newer codebase] and [its license]).

[sce]: https://conworkshop.com/emma/sce/
[the current python implementation]: https://github.com/KathTheDragon/Conlanger
[py-docs]: http://www.dragonlinguistics.com/sce/doc.html
[nvplv7+]: https://thufie.lain.haus/NPL.html
[mit]: https://github.com/KathTheDragon/Conlanger/blob/master/license.txt
[newer codebase]: https://github.com/KathTheDragon/SCE
[its license]: https://github.com/KathTheDragon/SCE/blob/main/LICENSE
