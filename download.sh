# Download Wikipedia
WIKIPEDIA_LINK="https://dumps.wikimedia.org/enwiki/20250123/enwiki-20250123-pages-articles-multistream.xml.bz2"
mkdir -pv data
curl $WIKIPEDIA_LINK | bzip2 -d > data/wikipedia.xml