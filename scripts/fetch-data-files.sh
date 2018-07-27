#!/bin/sh

mkdir -p data

# Download asn data (and clean non-utf8 data)
curl 'https://ftp.ripe.net/ripe/asnames/asn.txt' -o data/asn-unclean.txt
iconv -f utf-8 -t utf-8 -c data/asn-unclean.txt > data/asn.txt
rm data/asn-unclean.txt

# Download ip2asn data
curl 'https://iptoasn.com/data/ip2asn-combined.tsv.gz' -o data/ip2asn-combined.tsv.gz
gunzip data/ip2asn-combined.tsv.gz
