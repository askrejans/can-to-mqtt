# Third-party notices

First-party can-to-mqtt 0.4.1 is GPL-3.0-only. Third-party components retain the terms below; this table is generated from the exact Cargo.lock resolution, including build and platform-specific dependencies that may not be linked into every binary. Multiple-license choices remain available as specified upstream.

Release archives include collected original license/copyright/notice texts in `licenses/third-party/`. Run `node scripts/collect-licenses.cjs <output-directory>` from the source checkout to regenerate them after `cargo fetch --locked`. The four overrides in `licenses/overrides/` record pinned upstream texts omitted from their crate packages. Bundled Paho C and OpenSSL notices are included separately inside their sys/source crate directories.

Linux system libraries such as glibc and libudev are supplied by the host distribution, not included in these archives. Their runtime packages retain their own licensing and source distribution terms. The vendored OpenSSL source crate currently contains OpenSSL 3.6.3; this is the latest compatible published crate in this lockfile and is not a claim that upstream OpenSSL has no newer release.

| Crate | Version | Declared SPDX license | Upstream |
| --- | --- | --- | --- |
| aho-corasick | 1.1.5 | Unlicense OR MIT | [source](https://github.com/BurntSushi/aho-corasick) |
| android_system_properties | 0.1.6 | MIT OR Apache-2.0 | [source](https://github.com/nical/android_system_properties) |
| anstream | 1.0.0 | MIT OR Apache-2.0 | [source](https://github.com/rust-cli/anstyle.git) |
| anstyle | 1.0.14 | MIT OR Apache-2.0 | [source](https://github.com/rust-cli/anstyle.git) |
| anstyle-parse | 1.0.0 | MIT OR Apache-2.0 | [source](https://github.com/rust-cli/anstyle.git) |
| anstyle-query | 1.1.5 | MIT OR Apache-2.0 | [source](https://github.com/rust-cli/anstyle.git) |
| anstyle-wincon | 3.0.11 | MIT OR Apache-2.0 | [source](https://github.com/rust-cli/anstyle.git) |
| arraydeque | 0.5.1 | MIT/Apache-2.0 | [source](https://github.com/andylokandy/arraydeque) |
| async-channel | 2.5.0 | Apache-2.0 OR MIT | [source](https://github.com/smol-rs/async-channel) |
| async-trait | 0.1.92 | MIT OR Apache-2.0 | [source](https://github.com/dtolnay/async-trait) |
| atomic-waker | 1.1.2 | Apache-2.0 OR MIT | [source](https://github.com/smol-rs/atomic-waker) |
| autocfg | 1.5.1 | Apache-2.0 OR MIT | [source](https://github.com/cuviper/autocfg) |
| aws-lc-rs | 1.18.2 | ISC AND (Apache-2.0 OR ISC) | [source](https://github.com/aws/aws-lc-rs) |
| aws-lc-sys | 0.46.0 | ISC AND (Apache-2.0 OR ISC) AND Apache-2.0 AND MIT AND BSD-3-Clause AND (Apache-2.0 OR ISC OR MIT) AND (Apache-2.0 OR ISC OR MIT-0) | [source](https://github.com/aws/aws-lc-rs) |
| base64 | 0.22.1 | MIT OR Apache-2.0 | [source](https://github.com/marshallpierce/rust-base64) |
| bitflags | 2.13.2 | MIT OR Apache-2.0 | [source](https://github.com/bitflags/bitflags) |
| bumpalo | 3.20.3 | MIT OR Apache-2.0 | [source](https://github.com/fitzgen/bumpalo) |
| byteorder | 1.5.0 | Unlicense OR MIT | [source](https://github.com/BurntSushi/byteorder) |
| bytes | 1.12.1 | MIT | [source](https://github.com/tokio-rs/bytes) |
| cc | 1.9.0 | MIT OR Apache-2.0 | [source](https://github.com/rust-lang/cc-rs) |
| cfg_aliases | 0.2.2 | MIT | [source](https://github.com/katharostech/cfg_aliases) |
| cfg-if | 1.0.5 | MIT OR Apache-2.0 | [source](https://github.com/rust-lang/cfg-if) |
| chrono | 0.4.45 | MIT OR Apache-2.0 | [source](https://github.com/chronotope/chrono) |
| clap_builder | 4.6.7 | MIT OR Apache-2.0 | [source](https://github.com/clap-rs/clap) |
| clap_derive | 4.6.7 | MIT OR Apache-2.0 | [source](https://github.com/clap-rs/clap) |
| clap_lex | 1.1.1 | MIT OR Apache-2.0 | [source](https://github.com/clap-rs/clap) |
| clap | 4.6.7 | MIT OR Apache-2.0 | [source](https://github.com/clap-rs/clap) |
| cmake | 0.1.58 | MIT OR Apache-2.0 | [source](https://github.com/rust-lang/cmake-rs) |
| colorchoice | 1.0.5 | MIT OR Apache-2.0 | [source](https://github.com/rust-cli/anstyle.git) |
| concurrent-queue | 2.5.0 | Apache-2.0 OR MIT | [source](https://github.com/smol-rs/concurrent-queue) |
| config | 0.15.27 | MIT OR Apache-2.0 | [source](https://github.com/rust-cli/config-rs) |
| const-random | 0.1.18 | MIT OR Apache-2.0 | [source](https://github.com/tkaitchuck/constrandom) |
| const-random-macro | 0.1.16 | MIT OR Apache-2.0 | [source](https://github.com/tkaitchuck/constrandom) |
| convert_case | 0.6.0 | MIT | [source](https://github.com/rutrum/convert-case) |
| core_detect | 1.0.0 | MIT/Apache-2.0 | [source](https://github.com/thomcc/core_detect) |
| core-foundation | 0.10.1 | MIT OR Apache-2.0 | [source](https://github.com/servo/core-foundation-rs) |
| core-foundation-sys | 0.8.7 | MIT OR Apache-2.0 | [source](https://github.com/servo/core-foundation-rs) |
| crossbeam-channel | 0.5.17 | MIT OR Apache-2.0 | [source](https://github.com/crossbeam-rs/crossbeam) |
| crossbeam-epoch | 0.9.21 | MIT OR Apache-2.0 | [source](https://github.com/crossbeam-rs/crossbeam) |
| crossbeam-utils | 0.8.23 | MIT OR Apache-2.0 | [source](https://github.com/crossbeam-rs/crossbeam) |
| crunchy | 0.2.4 | MIT | [source](https://github.com/eira-fransham/crunchy) |
| csv | 1.4.0 | Unlicense/MIT | [source](https://github.com/BurntSushi/rust-csv) |
| csv-core | 0.1.13 | Unlicense/MIT | [source](https://github.com/BurntSushi/rust-csv) |
| darling_core | 0.20.11 | MIT | [source](https://github.com/TedDriggs/darling) |
| darling_macro | 0.20.11 | MIT | [source](https://github.com/TedDriggs/darling) |
| darling | 0.20.11 | MIT | [source](https://github.com/TedDriggs/darling) |
| derive_builder_core | 0.20.2 | MIT OR Apache-2.0 | [source](https://github.com/colin-kiegel/rust-derive-builder) |
| derive_builder_macro | 0.20.2 | MIT OR Apache-2.0 | [source](https://github.com/colin-kiegel/rust-derive-builder) |
| derive_builder | 0.20.2 | MIT OR Apache-2.0 | [source](https://github.com/colin-kiegel/rust-derive-builder) |
| dirs-next | 2.0.0 | MIT OR Apache-2.0 | [source](https://github.com/xdg-rs/dirs) |
| dirs-sys-next | 0.1.2 | MIT OR Apache-2.0 | [source](https://github.com/xdg-rs/dirs/tree/master/dirs-sys) |
| dlv-list | 0.5.2 | MIT OR Apache-2.0 | [source](https://github.com/sgodwincs/dlv-list-rs) |
| dunce | 1.0.5 | CC0-1.0 OR MIT-0 OR Apache-2.0 | [source](https://gitlab.com/kornelski/dunce) |
| either | 1.19.0 | MIT OR Apache-2.0 | [source](https://github.com/rayon-rs/either) |
| embedded-can | 0.4.1 | MIT OR Apache-2.0 | [source](https://github.com/rust-embedded/embedded-hal) |
| encode_unicode | 1.0.0 | Apache-2.0 OR MIT | [source](https://github.com/tormol/encode_unicode) |
| encoding_rs | 0.8.42 | (Apache-2.0 OR MIT) AND BSD-3-Clause | [source](https://github.com/hsivonen/encoding_rs) |
| equivalent | 1.0.3 | Apache-2.0 OR MIT | [source](https://github.com/indexmap-rs/equivalent) |
| erased-serde | 0.4.10 | MIT OR Apache-2.0 | [source](https://github.com/dtolnay/erased-serde) |
| errno | 0.3.14 | MIT OR Apache-2.0 | [source](https://github.com/lambda-fairy/rust-errno) |
| event-listener | 5.4.2 | Apache-2.0 OR MIT | [source](https://github.com/smol-rs/event-listener) |
| event-listener-strategy | 0.5.4 | Apache-2.0 OR MIT | [source](https://github.com/smol-rs/event-listener-strategy) |
| evmap | 11.0.0 | MIT OR Apache-2.0 | [source](https://github.com/jonhoo/evmap.git) |
| find-msvc-tools | 0.1.15 | MIT OR Apache-2.0 | [source](https://github.com/rust-lang/cc-rs) |
| fnv | 1.0.7 | Apache-2.0 / MIT | [source](https://github.com/servo/rust-fnv) |
| foldhash | 0.2.0 | Zlib | [source](https://github.com/orlp/foldhash) |
| fs_extra | 1.3.0 | MIT | [source](https://github.com/webdesus/fs_extra) |
| futures | 0.3.34 | MIT OR Apache-2.0 | [source](https://github.com/rust-lang/futures-rs) |
| futures-channel | 0.3.34 | MIT OR Apache-2.0 | [source](https://github.com/rust-lang/futures-rs) |
| futures-core | 0.3.34 | MIT OR Apache-2.0 | [source](https://github.com/rust-lang/futures-rs) |
| futures-executor | 0.3.34 | MIT OR Apache-2.0 | [source](https://github.com/rust-lang/futures-rs) |
| futures-io | 0.3.34 | MIT OR Apache-2.0 | [source](https://github.com/rust-lang/futures-rs) |
| futures-macro | 0.3.34 | MIT OR Apache-2.0 | [source](https://github.com/rust-lang/futures-rs) |
| futures-sink | 0.3.34 | MIT OR Apache-2.0 | [source](https://github.com/rust-lang/futures-rs) |
| futures-task | 0.3.34 | MIT OR Apache-2.0 | [source](https://github.com/rust-lang/futures-rs) |
| futures-timer | 3.0.4 | MIT/Apache-2.0 | [source](https://github.com/async-rs/futures-timer) |
| futures-util | 0.3.34 | MIT OR Apache-2.0 | [source](https://github.com/rust-lang/futures-rs) |
| generator | 0.8.11 | MIT/Apache-2.0 | [source](https://github.com/Xudong-Huang/generator-rs.git) |
| getrandom | 0.2.17 | MIT OR Apache-2.0 | [source](https://github.com/rust-random/getrandom) |
| getrandom | 0.3.4 | MIT OR Apache-2.0 | [source](https://github.com/rust-random/getrandom) |
| getrandom | 0.4.3 | MIT OR Apache-2.0 | [source](https://github.com/rust-random/getrandom) |
| getset | 0.1.7 | MIT | [source](https://github.com/jbaublitz/getset) |
| h2 | 0.4.20 | MIT | [source](https://github.com/hyperium/h2) |
| hashbag | 0.1.13 | MIT OR Apache-2.0 | [source](https://github.com/jonhoo/hashbag.git) |
| hashbrown | 0.14.5 | MIT OR Apache-2.0 | [source](https://github.com/rust-lang/hashbrown) |
| hashbrown | 0.16.1 | MIT OR Apache-2.0 | [source](https://github.com/rust-lang/hashbrown) |
| hashbrown | 0.17.1 | MIT OR Apache-2.0 | [source](https://github.com/rust-lang/hashbrown) |
| hashlink | 0.11.1 | MIT OR Apache-2.0 | [source](https://github.com/djc/hashlink) |
| heck | 0.5.0 | MIT OR Apache-2.0 | [source](https://github.com/withoutboats/heck) |
| hermit-abi | 0.5.3 | MIT OR Apache-2.0 | [source](https://github.com/hermit-os/hermit-rs) |
| hex | 0.4.3 | MIT OR Apache-2.0 | [source](https://github.com/KokaKiwi/rust-hex) |
| http | 1.5.0 | MIT OR Apache-2.0 | [source](https://github.com/hyperium/http) |
| http-body | 1.1.0 | MIT | [source](https://github.com/hyperium/http-body) |
| http-body-util | 0.1.5 | MIT | [source](https://github.com/hyperium/http-body) |
| httparse | 1.10.1 | MIT OR Apache-2.0 | [source](https://github.com/seanmonstar/httparse) |
| httpdate | 1.0.3 | MIT OR Apache-2.0 | [source](https://github.com/pyfisch/httpdate) |
| hyper | 1.12.0 | MIT | [source](https://github.com/hyperium/hyper) |
| hyper-rustls | 0.27.10 | Apache-2.0 OR ISC OR MIT | [source](https://github.com/rustls/hyper-rustls) |
| hyper-util | 0.1.21 | MIT | [source](https://github.com/hyperium/hyper-util) |
| iana-time-zone | 0.1.65 | MIT OR Apache-2.0 | [source](https://github.com/strawlab/iana-time-zone) |
| iana-time-zone-haiku | 0.1.2 | MIT OR Apache-2.0 | [source](https://github.com/strawlab/iana-time-zone) |
| ident_case | 1.0.1 | MIT/Apache-2.0 | [source](https://github.com/TedDriggs/ident_case) |
| indexmap | 2.14.2 | Apache-2.0 OR MIT | [source](https://github.com/indexmap-rs/indexmap) |
| ipnet | 2.12.2 | MIT OR Apache-2.0 | [source](https://github.com/krisprice/ipnet) |
| is_terminal_polyfill | 1.70.2 | MIT OR Apache-2.0 | [source](https://github.com/polyfill-rs/is_terminal_polyfill) |
| is-terminal | 0.4.17 | MIT | [source](https://github.com/sunfishcode/is-terminal) |
| itoa | 1.0.18 | MIT OR Apache-2.0 | [source](https://github.com/dtolnay/itoa) |
| jobserver | 0.1.35 | MIT OR Apache-2.0 | [source](https://github.com/rust-lang/jobserver-rs) |
| js-sys | 0.3.106 | MIT OR Apache-2.0 | [source](https://github.com/wasm-bindgen/wasm-bindgen/tree/master/crates/js-sys) |
| json5 | 1.3.3 | MIT | [source](https://github.com/callum-oakley/json5-rs) |
| lazy_static | 1.5.1 | MIT OR Apache-2.0 | [source](https://github.com/rust-lang-nursery/lazy-static.rs) |
| left-right | 0.11.8 | MIT OR Apache-2.0 | [source](https://github.com/jonhoo/left-right.git) |
| libc | 0.2.191 | MIT OR Apache-2.0 | [source](https://github.com/rust-lang/libc) |
| libredox | 0.1.25 | MIT | [source](https://gitlab.redox-os.org/redox-os/libredox.git) |
| linux-raw-sys | 0.12.1 | Apache-2.0 WITH LLVM-exception OR Apache-2.0 OR MIT | [source](https://github.com/sunfishcode/linux-raw-sys) |
| lock_api | 0.4.14 | MIT OR Apache-2.0 | [source](https://github.com/Amanieu/parking_lot) |
| log | 0.4.34 | MIT OR Apache-2.0 | [source](https://github.com/rust-lang/log) |
| loom | 0.7.2 | MIT | [source](https://github.com/tokio-rs/loom) |
| matchers | 0.2.0 | MIT | [source](https://github.com/hawkw/matchers) |
| memchr | 2.8.3 | Unlicense OR MIT | [source](https://github.com/BurntSushi/memchr) |
| memoffset | 0.9.1 | MIT | [source](https://github.com/Gilnaa/memoffset) |
| metrics | 0.24.6 | MIT | [source](https://github.com/metrics-rs/metrics) |
| metrics-exporter-prometheus | 0.18.3 | MIT AND Apache-2.0 | [source](https://github.com/metrics-rs/metrics) |
| metrics-util | 0.20.4 | MIT | [source](https://github.com/metrics-rs/metrics) |
| mio | 1.2.4 | MIT | [source](https://github.com/tokio-rs/mio) |
| multiversion_no_op | 1.0.0 | Apache-2.0 OR MIT | [source](https://github.com/hsivonen/multiversion_no_op) |
| nb | 1.1.0 | MIT OR Apache-2.0 | [source](https://github.com/rust-embedded/nb) |
| neli | 0.7.5 | BSD-3-Clause | [source](https://github.com/jbaublitz/neli) |
| neli-proc-macros | 0.2.2 | BSD-3-Clause | [source](https://github.com/jbaublitz/neli) |
| nix | 0.31.3 | MIT | [source](https://github.com/nix-rust/nix) |
| nu-ansi-term | 0.50.3 | MIT | [source](https://github.com/nushell/nu-ansi-term) |
| num-traits | 0.2.19 | MIT OR Apache-2.0 | [source](https://github.com/rust-num/num-traits) |
| once_cell_polyfill | 1.70.2 | MIT OR Apache-2.0 | [source](https://github.com/polyfill-rs/once_cell_polyfill) |
| once_cell | 1.21.4 | MIT OR Apache-2.0 | [source](https://github.com/matklad/once_cell) |
| openssl-probe | 0.2.1 | MIT OR Apache-2.0 | [source](https://github.com/rustls/openssl-probe) |
| openssl-src | 300.6.1+3.6.3 | MIT/Apache-2.0 | [source](https://github.com/alexcrichton/openssl-src-rs) |
| openssl-sys | 0.9.117 | MIT | [source](https://github.com/rust-openssl/rust-openssl) |
| ordered-multimap | 0.7.3 | MIT | [source](https://github.com/sgodwincs/ordered-multimap-rs) |
| paho-mqtt | 0.14.0 | EPL-2.0 | [source](https://github.com/eclipse/paho.mqtt.rust) |
| paho-mqtt-sys | 0.11.0 | EPL-2.0 | [source](https://github.com/eclipse/paho.mqtt.rust) |
| parking_lot_core | 0.9.12 | MIT OR Apache-2.0 | [source](https://github.com/Amanieu/parking_lot) |
| parking_lot | 0.12.5 | MIT OR Apache-2.0 | [source](https://github.com/Amanieu/parking_lot) |
| parking | 2.2.1 | Apache-2.0 OR MIT | [source](https://github.com/smol-rs/parking) |
| pathdiff | 0.2.3 | MIT/Apache-2.0 | [source](https://github.com/Manishearth/pathdiff) |
| pin-project-lite | 0.2.17 | Apache-2.0 OR MIT | [source](https://github.com/taiki-e/pin-project-lite) |
| pkg-config | 0.3.34 | MIT OR Apache-2.0 | [source](https://github.com/rust-lang/pkg-config-rs) |
| portable-atomic | 1.15.0 | Apache-2.0 OR MIT | [source](https://github.com/taiki-e/portable-atomic) |
| ppv-lite86 | 0.2.21 | MIT OR Apache-2.0 | [source](https://github.com/cryptocorrosion/cryptocorrosion) |
| prettytable-rs | 0.10.0 | BSD-3-Clause | [source](https://github.com/phsym/prettytable-rs) |
| proc-macro2 | 1.0.107 | MIT OR Apache-2.0 | [source](https://github.com/dtolnay/proc-macro2) |
| quanta | 0.12.6 | MIT | [source](https://github.com/metrics-rs/quanta) |
| quote | 1.0.47 | MIT OR Apache-2.0 | [source](https://github.com/dtolnay/quote) |
| r-efi | 5.3.0 | MIT OR Apache-2.0 OR LGPL-2.1-or-later | [source](https://github.com/r-efi/r-efi) |
| r-efi | 6.0.0 | MIT OR Apache-2.0 OR LGPL-2.1-or-later | [source](https://github.com/r-efi/r-efi) |
| rand_chacha | 0.9.0 | MIT OR Apache-2.0 | [source](https://github.com/rust-random/rand) |
| rand_core | 0.9.5 | MIT OR Apache-2.0 | [source](https://github.com/rust-random/rand) |
| rand_xoshiro | 0.7.0 | MIT OR Apache-2.0 | [source](https://github.com/rust-random/rngs) |
| rand | 0.9.5 | MIT OR Apache-2.0 | [source](https://github.com/rust-random/rand) |
| rapidhash | 4.5.1 | MIT OR Apache-2.0 | [source](https://github.com/hoxxep/rapidhash) |
| raw-cpuid | 11.6.0 | MIT | [source](https://github.com/gz/rust-cpuid) |
| redox_syscall | 0.5.18 | MIT | [source](https://gitlab.redox-os.org/redox-os/syscall) |
| redox_users | 0.4.6 | MIT | [source](https://gitlab.redox-os.org/redox-os/users) |
| regex-automata | 0.4.18 | MIT OR Apache-2.0 | [source](https://github.com/rust-lang/regex) |
| regex-syntax | 0.8.11 | MIT OR Apache-2.0 | [source](https://github.com/rust-lang/regex) |
| ring | 0.17.14 | Apache-2.0 AND ISC | [source](https://github.com/briansmith/ring) |
| ron | 0.12.2 | MIT OR Apache-2.0 | [source](https://github.com/ron-rs/ron) |
| rust-ini | 0.21.3 | MIT | [source](https://github.com/zonyitoo/rust-ini) |
| rustls | 0.23.46 | Apache-2.0 OR ISC OR MIT | [source](https://github.com/rustls/rustls) |
| rustls-native-certs | 0.8.5 | Apache-2.0 OR ISC OR MIT | [source](https://github.com/rustls/rustls-native-certs) |
| rustls-pki-types | 1.15.1 | MIT OR Apache-2.0 | [source](https://github.com/rustls/pki-types) |
| rustls-webpki | 0.103.16 | ISC | [source](https://github.com/rustls/webpki) |
| rustversion | 1.0.23 | MIT OR Apache-2.0 | [source](https://github.com/dtolnay/rustversion) |
| ryu | 1.0.23 | Apache-2.0 OR BSL-1.0 | [source](https://github.com/dtolnay/ryu) |
| schannel | 0.1.30 | MIT | [source](https://github.com/steffengy/schannel-rs) |
| scoped-tls | 1.0.1 | MIT/Apache-2.0 | [source](https://github.com/alexcrichton/scoped-tls) |
| scopeguard | 1.2.0 | MIT OR Apache-2.0 | [source](https://github.com/bluss/scopeguard) |
| sd-notify | 0.4.5 | MIT OR Apache-2.0 | [source](https://github.com/lnicola/sd-notify) |
| security-framework | 3.7.0 | MIT OR Apache-2.0 | [source](https://github.com/kornelski/rust-security-framework) |
| security-framework-sys | 2.17.0 | MIT OR Apache-2.0 | [source](https://github.com/kornelski/rust-security-framework) |
| serde_core | 1.0.229 | MIT OR Apache-2.0 | [source](https://github.com/serde-rs/serde) |
| serde_derive | 1.0.229 | MIT OR Apache-2.0 | [source](https://github.com/serde-rs/serde) |
| serde_json | 1.0.154 | MIT OR Apache-2.0 | [source](https://github.com/serde-rs/json) |
| serde_spanned | 1.1.2 | MIT OR Apache-2.0 | [source](https://github.com/toml-rs/toml) |
| serde | 1.0.229 | MIT OR Apache-2.0 | [source](https://github.com/serde-rs/serde) |
| serde-untagged | 0.1.9 | MIT OR Apache-2.0 | [source](https://github.com/dtolnay/serde-untagged) |
| sharded-slab | 0.1.7 | MIT | [source](https://github.com/hawkw/sharded-slab) |
| shlex | 2.0.1 | MIT OR Apache-2.0 | [source](https://github.com/comex/rust-shlex) |
| signal-hook-registry | 1.4.8 | MIT OR Apache-2.0 | [source](https://github.com/vorner/signal-hook) |
| simdutf8 | 0.1.5 | MIT OR Apache-2.0 | [source](https://github.com/rusticstuff/simdutf8) |
| sketches-ddsketch | 0.3.1 | Apache-2.0 | [source](https://github.com/mheffner/rust-sketches-ddsketch) |
| slab | 0.4.12 | MIT | [source](https://github.com/tokio-rs/slab) |
| smallvec | 1.16.3 | MIT OR Apache-2.0 | [source](https://github.com/servo/rust-smallvec) |
| socket2 | 0.6.5 | MIT OR Apache-2.0 | [source](https://github.com/rust-lang/socket2) |
| socketcan | 4.0.0 | MIT | [source](https://github.com/mbr/socketcan-rs) |
| strsim | 0.11.1 | MIT | [source](https://github.com/rapidfuzz/strsim-rs) |
| subtle | 2.6.1 | BSD-3-Clause | [source](https://github.com/dalek-cryptography/subtle) |
| syn | 2.0.119 | MIT OR Apache-2.0 | [source](https://github.com/dtolnay/syn) |
| syn | 3.0.9 | MIT OR Apache-2.0 | [source](https://github.com/dtolnay/syn) |
| term | 0.7.0 | MIT/Apache-2.0 | [source](https://github.com/Stebalien/term) |
| thiserror | 1.0.69 | MIT OR Apache-2.0 | [source](https://github.com/dtolnay/thiserror) |
| thiserror | 2.0.21 | MIT OR Apache-2.0 | [source](https://github.com/dtolnay/thiserror) |
| thiserror-impl | 1.0.69 | MIT OR Apache-2.0 | [source](https://github.com/dtolnay/thiserror) |
| thiserror-impl | 2.0.21 | MIT OR Apache-2.0 | [source](https://github.com/dtolnay/thiserror) |
| thread_local | 1.1.10 | MIT OR Apache-2.0 | [source](https://github.com/Amanieu/thread_local-rs) |
| tiny-keccak | 2.0.2 | CC0-1.0 | [crate](https://crates.io/crates/tiny-keccak/2.0.2) |
| tokio | 1.54.1 | MIT | [source](https://github.com/tokio-rs/tokio) |
| tokio-macros | 2.7.2 | MIT | [source](https://github.com/tokio-rs/tokio) |
| tokio-rustls | 0.26.6 | MIT OR Apache-2.0 | [source](https://github.com/rustls/tokio-rustls) |
| tokio-util | 0.7.20 | MIT | [source](https://github.com/tokio-rs/tokio) |
| toml_datetime | 1.1.2+spec-1.1.0 | MIT OR Apache-2.0 | [source](https://github.com/toml-rs/toml) |
| toml_parser | 1.1.5+spec-1.1.0 | MIT OR Apache-2.0 | [source](https://github.com/toml-rs/toml) |
| toml_writer | 1.1.3+spec-1.1.0 | MIT OR Apache-2.0 | [source](https://github.com/toml-rs/toml) |
| toml | 1.1.8+spec-1.1.0 | MIT OR Apache-2.0 | [source](https://github.com/toml-rs/toml) |
| tower-service | 0.3.3 | MIT | [source](https://github.com/tower-rs/tower) |
| tracing | 0.1.44 | MIT | [source](https://github.com/tokio-rs/tracing) |
| tracing-attributes | 0.1.31 | MIT | [source](https://github.com/tokio-rs/tracing) |
| tracing-core | 0.1.36 | MIT | [source](https://github.com/tokio-rs/tracing) |
| tracing-log | 0.2.0 | MIT | [source](https://github.com/tokio-rs/tracing) |
| tracing-subscriber | 0.3.23 | MIT | [source](https://github.com/tokio-rs/tracing) |
| try-lock | 0.2.5 | MIT | [source](https://github.com/seanmonstar/try-lock) |
| typeid | 1.0.3 | MIT OR Apache-2.0 | [source](https://github.com/dtolnay/typeid) |
| ucd-trie | 0.1.7 | MIT OR Apache-2.0 | [source](https://github.com/BurntSushi/ucd-generate) |
| unicode-ident | 1.0.27 | (MIT OR Apache-2.0) AND Unicode-3.0 | [source](https://github.com/dtolnay/unicode-ident) |
| unicode-segmentation | 1.13.3 | MIT OR Apache-2.0 | [source](https://github.com/unicode-rs/unicode-segmentation) |
| unicode-width | 0.1.14 | MIT OR Apache-2.0 | [source](https://github.com/unicode-rs/unicode-width) |
| untrusted | 0.9.0 | ISC | [source](https://github.com/briansmith/untrusted) |
| utf8parse | 0.2.2 | Apache-2.0 OR MIT | [source](https://github.com/alacritty/vte) |
| valuable | 0.1.1 | MIT | [source](https://github.com/tokio-rs/valuable) |
| vcpkg | 0.2.15 | MIT/Apache-2.0 | [source](https://github.com/mcgoo/vcpkg-rs) |
| want | 0.3.2 | MIT | [source](https://github.com/seanmonstar/want) |
| wasi | 0.11.1+wasi-snapshot-preview1 | Apache-2.0 WITH LLVM-exception OR Apache-2.0 OR MIT | [source](https://github.com/bytecodealliance/wasi) |
| wasip2 | 1.0.4+wasi-0.2.12 | Apache-2.0 WITH LLVM-exception OR Apache-2.0 OR MIT | [source](https://github.com/bytecodealliance/wasi-rs) |
| wasm-bindgen | 0.2.129 | MIT OR Apache-2.0 | [source](https://github.com/wasm-bindgen/wasm-bindgen) |
| wasm-bindgen-macro | 0.2.129 | MIT OR Apache-2.0 | [source](https://github.com/wasm-bindgen/wasm-bindgen/tree/master/crates/macro) |
| wasm-bindgen-macro-support | 0.2.129 | MIT OR Apache-2.0 | [source](https://github.com/wasm-bindgen/wasm-bindgen/tree/main/crates/macro-support) |
| wasm-bindgen-shared | 0.2.129 | MIT OR Apache-2.0 | [source](https://github.com/wasm-bindgen/wasm-bindgen/tree/master/crates/shared) |
| web-sys | 0.3.106 | MIT OR Apache-2.0 | [source](https://github.com/wasm-bindgen/wasm-bindgen/tree/master/crates/web-sys) |
| winapi | 0.3.9 | MIT/Apache-2.0 | [source](https://github.com/retep998/winapi-rs) |
| winapi-i686-pc-windows-gnu | 0.4.0 | MIT/Apache-2.0 | [source](https://github.com/retep998/winapi-rs) |
| winapi-x86_64-pc-windows-gnu | 0.4.0 | MIT/Apache-2.0 | [source](https://github.com/retep998/winapi-rs) |
| windows_aarch64_gnullvm | 0.52.6 | MIT OR Apache-2.0 | [source](https://github.com/microsoft/windows-rs) |
| windows_aarch64_msvc | 0.52.6 | MIT OR Apache-2.0 | [source](https://github.com/microsoft/windows-rs) |
| windows_i686_gnu | 0.52.6 | MIT OR Apache-2.0 | [source](https://github.com/microsoft/windows-rs) |
| windows_i686_gnullvm | 0.52.6 | MIT OR Apache-2.0 | [source](https://github.com/microsoft/windows-rs) |
| windows_i686_msvc | 0.52.6 | MIT OR Apache-2.0 | [source](https://github.com/microsoft/windows-rs) |
| windows_x86_64_gnu | 0.52.6 | MIT OR Apache-2.0 | [source](https://github.com/microsoft/windows-rs) |
| windows_x86_64_gnullvm | 0.52.6 | MIT OR Apache-2.0 | [source](https://github.com/microsoft/windows-rs) |
| windows_x86_64_msvc | 0.52.6 | MIT OR Apache-2.0 | [source](https://github.com/microsoft/windows-rs) |
| windows-core | 0.62.2 | MIT OR Apache-2.0 | [source](https://github.com/microsoft/windows-rs) |
| windows-implement | 0.60.2 | MIT OR Apache-2.0 | [source](https://github.com/microsoft/windows-rs) |
| windows-interface | 0.59.3 | MIT OR Apache-2.0 | [source](https://github.com/microsoft/windows-rs) |
| windows-link | 0.2.1 | MIT OR Apache-2.0 | [source](https://github.com/microsoft/windows-rs) |
| windows-result | 0.4.1 | MIT OR Apache-2.0 | [source](https://github.com/microsoft/windows-rs) |
| windows-strings | 0.5.1 | MIT OR Apache-2.0 | [source](https://github.com/microsoft/windows-rs) |
| windows-sys | 0.52.0 | MIT OR Apache-2.0 | [source](https://github.com/microsoft/windows-rs) |
| windows-sys | 0.61.2 | MIT OR Apache-2.0 | [source](https://github.com/microsoft/windows-rs) |
| windows-targets | 0.52.6 | MIT OR Apache-2.0 | [source](https://github.com/microsoft/windows-rs) |
| winnow | 1.0.4 | MIT | [source](https://github.com/winnow-rs/winnow) |
| wit-bindgen | 0.57.1 | Apache-2.0 WITH LLVM-exception OR Apache-2.0 OR MIT | [source](https://github.com/bytecodealliance/wit-bindgen) |
| yaml-rust2 | 0.11.1 | MIT OR Apache-2.0 | [source](https://github.com/Ethiraric/yaml-rust2) |
| zerocopy | 0.8.63 | BSD-2-Clause OR Apache-2.0 OR MIT | [source](https://github.com/google/zerocopy) |
| zerocopy-derive | 0.8.63 | BSD-2-Clause OR Apache-2.0 OR MIT | [source](https://github.com/google/zerocopy) |
| zeroize | 1.9.1 | Apache-2.0 OR MIT | [source](https://github.com/RustCrypto/utils) |
| zmij | 1.0.23 | MIT | [source](https://github.com/dtolnay/zmij) |
