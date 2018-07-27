#!/bin/sh

mkdir -p data

# Download asn data (and clean non-utf8 data)
curl 'https://ftp.ripe.net/ripe/asnames/asn.txt' -o data/asn-unclean.txt
iconv -f utf-8 -t utf-8 -c data/asn-unclean.txt > data/asn.txt
rm data/asn-unclean.txt

# Create ip2asn data
pyasn_util_download.py -46
pyasn_util_convert.py --single *.bz2 data/ip2asn.dat
rm *.bz2
