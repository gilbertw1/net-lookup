net-lookup
==========

A simple ip and domain lookup utility written in rust.


Getting Started
---------------

Build the project (ensure rust & cargo is installed)
    
    $ cargo build --release

Fetch remote data files (requires pyasn)

    $ ./target/release/net-lookup-updater


Usage
-----

Run in daemon mode:

    $ ./target/release/net-lookup -d -p 9000

Perform single lookup:

    $ ./target/release/net-lookup <ip-address>

Help:

    $ ./target/release/net-lookup -h

Run http query (running in daemon mode)

    $ curl 'http://localhost:8080/<ip-address-or-domain>'


Sample Response Payload
-----------------------

```json
{
  "asn": {
    "id": 13285,
    "handle": "OPALTELECOM-AS TalkTalk Communications Limited,",
    "name": null,
    "country": "GB"
  },
  "geo": {
    "city": {
      "geoname_id": 2655984,
      "names": {
        "de": "Belfast",
        "en": "Belfast",
        "es": "Belfast",
        "fr": "Belfast",
        "ja": "ベルファスト",
        "pt-BR": "Belfast",
        "ru": "Белфаст",
        "zh-CN": "贝尔法斯特"
      }
    },
    "continent": {
      "code": "EU",
      "geoname_id": 6255148,
      "names": {
        "de": "Europa",
        "en": "Europe",
        "es": "Europa",
        "fr": "Europe",
        "ja": "ヨーロッパ",
        "pt-BR": "Europa",
        "ru": "Европа",
        "zh-CN": "欧洲"
      }
    },
    "country": {
      "geoname_id": 2635167,
      "iso_code": "GB",
      "names": {
        "de": "Vereinigtes Königreich",
        "en": "United Kingdom",
        "es": "Reino Unido",
        "fr": "Royaume-Uni",
        "ja": "イギリス",
        "pt-BR": "Reino Unido",
        "ru": "Великобритания",
        "zh-CN": "英国"
      }
    },
    "location": {
      "latitude": 54.5833,
      "longitude": -5.9333,
      "metro_code": null,
      "time_zone": "Europe/London"
    },
    "postal": {
      "code": "BT15"
    },
    "registered_country": {
      "geoname_id": 2635167,
      "iso_code": "GB",
      "names": {
        "de": "Vereinigtes Königreich",
        "en": "United Kingdom",
        "es": "Reino Unido",
        "fr": "Royaume-Uni",
        "ja": "イギリス",
        "pt-BR": "Reino Unido",
        "ru": "Великобритания",
        "zh-CN": "英国"
      }
    },
    "represented_country": null,
    "subdivisions": [
      {
        "geoname_id": 2641364,
        "iso_code": "NIR",
        "names": {
          "de": "Nordirland",
          "en": "Northern Ireland",
          "es": "Irlanda del Norte",
          "fr": "Irlande du Nord",
          "ru": "Северная Ирландия"
        }
      },
      {
        "geoname_id": 3333223,
        "iso_code": "BFS",
        "names": {
          "en": "Belfast"
        }
      }
    ],
    "traits": null
  },
  "reverse_dns": [
    "host-89-242-204-127.as13285.net"
  ]
}
```
