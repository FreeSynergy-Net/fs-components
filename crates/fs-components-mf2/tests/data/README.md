# Test data of `tests/packed.rs`

`src/*.mf2` are the sources (de, en, fr). `packed/` is the OUTPUT of the packer, generated, not written by hand:

```sh
# 1. build the packer (fs-release, branch unreviewed/pack-texts, head add2d220a714b9a0010299d06cb3fcf4dad73689)
git clone https://github.com/FreeSynergy-Net/fs-release && cd fs-release
git checkout add2d220a714b9a0010299d06cb3fcf4dad73689
bash /srv/FreeSynergy/.scratch/build-slot.sh cargo build --release -p fs-release --locked
# 2. pack (from this directory); the same call runs in the CI
rm -rf packed
<fs-release>/target/release/fs-release pack-texts --sources src --out packed --locales de,en,fr
```

Output: `manifest.mf2m`, `<language>.<hash>.mf2b`, `dates/<language>/icu.table` (the date table, Gregorian, no zones, about 1.4 KB).
The names carry the hash of the content, so a change of the sources or of the `mf2` pin gives other names: remove `packed/`
first. `fr` has a table too; the test "no table" hides it with a `DateData` wrapper (`Only`), nothing is deleted by hand.
The tables must be made again at every change of the `mf2` pin (the format follows ICU4X).
