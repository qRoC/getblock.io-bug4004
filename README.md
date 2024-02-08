Sometimes, in **batch** requests response contains id: `gbiid:REAL_REQUEST`, where `REAL_REQUEST` is `id` from request.

I think problem in your proxy.

1. Create shared node WS endpoint (Ethereum Sepolia)

2. Open `src/main.rs` and set `KEY`.

When the error is caught, the application will display `BUG FOUND! Response: ...`

Examples (responses with error):

```json
[
  {"id":4107,"jsonrpc":"2.0","result":"0x8746f8de6f0e8bc1"},
  {"id":4108,"jsonrpc":"2.0","result":"0x2a"},
  {"id":4109,"jsonrpc":"2.0","result":"0x"},
  {"id":4110,"jsonrpc":"2.0","result":"0x0"},
  {"id":4111,"jsonrpc":"2.0","result":"0x1"},
  {"id":"gbiid:4112","jsonrpc":"2.0","result":"..."}
]
```

```json
[
  {"jsonrpc":"2.0","id":"gbiid:1784","result":"0x00000000000000006597b52c0000000000000000000000000000000000000001"},
  {"jsonrpc":"2.0","id":"gbiid:1785","result":"0x000000000000000000000000000000000000000000000044a1478471cc880000"}
]
```

```json
 [
{"id":569,"jsonrpc":"2.0","result":"0x000000000000000000000000978d8944a2a6e0e988761b31bfdf8ef873f526f4"},
{"id":570,"jsonrpc":"2.0","result":"0x0000000000000000000000000000000000000000000000000000000000000000"},
{"id":571,"jsonrpc":"2.0","result":"0x0000000000000000000000000000000000000000000000000000000000000000"},
{"id":572,"jsonrpc":"2.0","result":"0x0"},
{"id":573,"jsonrpc":"2.0","result":"0x1"},
{"id":"gbiid:574","jsonrpc":"2.0","result":"..."},
{"id":575,"jsonrpc":"2.0","result":"0x00000000000000000000000000000000000000000000f182d76dfe4a8f9e813f"},
{"id":576,"jsonrpc":"2.0","result":"0x00000000000000000000000000000000000000000003f0736acec40078ee0eac"},
{"id":577,"jsonrpc":"2.0","result":"0x0"},
{"id":578,"jsonrpc":"2.0","result":"0x0"},
{"id":579,"jsonrpc":"2.0","result":"0x"},
{"id":580,"jsonrpc":"2.0","result":"0x0000000000000000000000000000000000000000000000000000000000000001"}
]
```

* `...` - content cut due to size
