ip-asn-lookup-service
=====================

A simple ip-asn lookup service written in rust


Getting Started
---------------

Fetch remote data files

    $ ./scripts/fetch-data-files.sh

Build the server (ensure rust & cargo is installed)
    
    $ ./scripts/build.sh

Run the server

    $ ./scripts/run.sh

Usage
-----

The server can be queried on port 8080

    $ curl 'http://localhost:8080/<ip-address>'
