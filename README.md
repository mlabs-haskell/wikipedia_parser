# Wikipedia Parser
This is a tool for processing Wikipedia articles and extracting important information from them as plaintext.

## Instructions for use
1. Run `./download.sh` to download all of Wikipedia as a single xml file. This will likely take a long time.
2. Install Rust on your system via [these instructions](https://rustup.rs/).
3. Install `just` with `cargo install just`.
4. Run `just extract-links` to create the graph of Wikipedia.
5. Run `just extract-contents` to parse the contents of the Wikipedia articles. This will also take a long time
6. Run `just extract-subgraph <root article> <degrees of separation>` to produce the list of all articles within `degrees of separation` of the root. For instance, if I wanted to find all articles within 5 degrees of separation from the article for RNA, I would run `just extract-subgraph RNA 5`.
