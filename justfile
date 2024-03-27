input_file := "data/enwiki-20231220-pages-articles-multistream.xml"	

_default:
	just -l

cargo_run := "cargo run --release"
parser := cargo_run + " --bin parser -- "
subgraph-extractor := cargo_run + " --bin subgraph-extractor -- "

extract-links:
	{{parser}} \
		--extractor links \
		--input-file {{input_file}} \
		--output-data-file "output/links/data.jsonl" \
		--output-index-file "output/links/index.txt" \
		--input-file-threads 16

extract-contents:
	{{parser}} \
		--extractor contents \
		--input-file {{input_file}} \
		--output-data-file "output/contents/data.jsonl" \
		--output-index-file "output/contents/index.txt" \
		--input-file-threads 16

extract-subgraph root depth:
	{{subgraph-extractor}} \
		--method depth-limited \
		--input-data-file "output/links/data.jsonl"	\
		--input-index-file "output/links/index.txt"	\
		--output-file "output/subgraph/{{root}}.txt" \
		--input-file-threads 16 \
		--root-page {{root}} \
		--depth {{depth}} 

extract-subgraph-fanout root depth fanout-factor:
	{{subgraph-extractor}} \
		--method depth-limited \
		--input-data-file "output/links/data.jsonl"	\
		--input-index-file "output/links/index.txt"	\
		--output-file "output/subgraph/root.txt" \
		--input-file-threads 16 \
		--root-page {{root}} \
		--depth {{depth}} \
		--fanout {{fanout-factor}} 
