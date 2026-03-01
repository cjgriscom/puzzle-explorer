# puzzle-explorer-rs

## Nauty and Traces Integration
Nauty and Traces are used via a WASM port of the Dreadnaut CLI.  These algorithms are used to canonize orbit generators, which is useful for caching purposes and for checking the uniqueness of piece types.
Homepage: https://pallini.di.uniroma1.it/
WASM port: https://github.com/cjgriscom/dreadnaut-wasm
Included version: 2_9_3, commit 0dc23ca

## GAP Integration
Puzzle Explorer runs a copy of GAP to assign group labels to piece orbits.
Homepage: https://www.gap-system.org/
WASM port: https://github.com/wangyenshu/gap-wasm
Included version: 4.16 (dev), commit 86d58ae
