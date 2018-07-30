#!/bin/sh

PORT=${1:-8080}

./target/release/ip-asn-lookup-service --port $PORT \
                                       --asn-database data/asn.txt \
                                       --ip2asn-database data/ip2asn.dat \
                                       --maxmind-city-database data/GeoLite2-City.mmdb
