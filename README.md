net-lookup
==========

A simple ip and domain lookup utility written in rust.


Getting Started
---------------

Build the project (ensure [rust & cargo](https://rustup.rs/) is installed)
    
    $ cargo build --release

Fetch remote data files (requires [pyasn](https://github.com/hadiasghari/pyasn))

    $ ./target/release/net-lookup-updater


Usage
-----

Run in daemon mode:

    $ ./target/release/net-lookup -d -p 9000

Perform single lookup:

    $ ./target/release/net-lookup <ip-address-or-domain>

Help:

    $ ./target/release/net-lookup -h

Run http query (running in daemon mode)

    $ curl 'http://localhost:8080/<ip-address-or-domain>'


Sample IP Response Payload
--------------------------

```json
{
  "ip": "89.242.204.127",
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


Sample Domain Response Payload
------------------------------

```json
{
  "domain": "google.com",
  "ipv4": [
    {
      "ip": "216.58.217.238",
      "asn": {
        "id": 15169,
        "handle": "GOOGLE",
        "name": "Google LLC,",
        "country": "US"
      },
      "geo": {
        "city": {
          "geoname_id": 5375480,
          "names": {
            "de": "Mountain View",
            "en": "Mountain View",
            "fr": "Mountain View",
            "ja": "マウンテンビュー",
            "ru": "Маунтин-Вью",
            "zh-CN": "芒廷维尤"
          }
        },
        "continent": {
          "code": "NA",
          "geoname_id": 6255149,
          "names": {
            "de": "Nordamerika",
            "en": "North America",
            "es": "Norteamérica",
            "fr": "Amérique du Nord",
            "ja": "北アメリカ",
            "pt-BR": "América do Norte",
            "ru": "Северная Америка",
            "zh-CN": "北美洲"
          }
        },
        "country": {
          "geoname_id": 6252001,
          "iso_code": "US",
          "names": {
            "de": "USA",
            "en": "United States",
            "es": "Estados Unidos",
            "fr": "États-Unis",
            "ja": "アメリカ合衆国",
            "pt-BR": "Estados Unidos",
            "ru": "США",
            "zh-CN": "美国"
          }
        },
        "location": {
          "latitude": 37.419200000000004,
          "longitude": -122.0574,
          "metro_code": 807,
          "time_zone": "America/Los_Angeles"
        },
        "postal": {
          "code": "94043"
        },
        "registered_country": {
          "geoname_id": 6252001,
          "iso_code": "US",
          "names": {
            "de": "USA",
            "en": "United States",
            "es": "Estados Unidos",
            "fr": "États-Unis",
            "ja": "アメリカ合衆国",
            "pt-BR": "Estados Unidos",
            "ru": "США",
            "zh-CN": "美国"
          }
        },
        "represented_country": null,
        "subdivisions": [
          {
            "geoname_id": 5332921,
            "iso_code": "CA",
            "names": {
              "de": "Kalifornien",
              "en": "California",
              "es": "California",
              "fr": "Californie",
              "ja": "カリフォルニア州",
              "pt-BR": "Califórnia",
              "ru": "Калифорния",
              "zh-CN": "加利福尼亚州"
            }
          }
        ],
        "traits": null
      },
      "reverse_dns": [
        "atl14s38-in-f14.1e100.net.",
        "atl14s38-in-f238.1e100.net.",
        "atl14s38-in-f238.1e100.net.",
        "atl14s38-in-f14.1e100.net."
      ]
    }
  ],
  "ipv6": [
    {
      "ip": "2607:f8b0:4002:805::200e",
      "asn": {
        "id": 15169,
        "handle": "GOOGLE",
        "name": "Google LLC,",
        "country": "US"
      },
      "geo": {
        "city": null,
        "continent": {
          "code": "NA",
          "geoname_id": 6255149,
          "names": {
            "de": "Nordamerika",
            "en": "North America",
            "es": "Norteamérica",
            "fr": "Amérique du Nord",
            "ja": "北アメリカ",
            "pt-BR": "América do Norte",
            "ru": "Северная Америка",
            "zh-CN": "北美洲"
          }
        },
        "country": {
          "geoname_id": 6252001,
          "iso_code": "US",
          "names": {
            "de": "USA",
            "en": "United States",
            "es": "Estados Unidos",
            "fr": "États-Unis",
            "ja": "アメリカ合衆国",
            "pt-BR": "Estados Unidos",
            "ru": "США",
            "zh-CN": "美国"
          }
        },
        "location": {
          "latitude": 37.751,
          "longitude": -97.822,
          "metro_code": null,
          "time_zone": null
        },
        "postal": null,
        "registered_country": {
          "geoname_id": 6252001,
          "iso_code": "US",
          "names": {
            "de": "USA",
            "en": "United States",
            "es": "Estados Unidos",
            "fr": "États-Unis",
            "ja": "アメリカ合衆国",
            "pt-BR": "Estados Unidos",
            "ru": "США",
            "zh-CN": "美国"
          }
        },
        "represented_country": null,
        "subdivisions": null,
        "traits": null
      },
      "reverse_dns": [
        "atl14s38-in-x0e.1e100.net."
      ]
    }
  ],
  "cname": [],
  "ns": [
    "ns4.google.com.",
    "ns2.google.com.",
    "ns3.google.com.",
    "ns1.google.com."
  ],
  "mx": [
    {
      "preference": 30,
      "exchange": "alt2.aspmx.l.google.com."
    },
    {
      "preference": 50,
      "exchange": "alt4.aspmx.l.google.com."
    },
    {
      "preference": 20,
      "exchange": "alt1.aspmx.l.google.com."
    },
    {
      "preference": 10,
      "exchange": "aspmx.l.google.com."
    },
    {
      "preference": 40,
      "exchange": "alt3.aspmx.l.google.com."
    }
  ],
  "txt": [
    "v=spf1 include:_spf.google.com ~all",
    "docusign=05958488-4752-4ef2-95eb-aa7ba8a3bd0e",
    "facebook-domain-verification=22rm551cu4k0ab0bxsw536tlds4h95"
  ],
  "soa": {
    "mname": "ns1.google.com.",
    "rname": "dns-admin.google.com.",
    "serial": 211774709,
    "refresh": 900,
    "retry": 900,
    "expire": 1800,
    "minimum": 60
  }
}
```
