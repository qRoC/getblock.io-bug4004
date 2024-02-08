Sometimes, in **batch** requests response contains id: `gbiid:REAL_REQUEST`, where `REAL_REQUEST` is `id` from request.

I think problem in your proxy.

1. Create shared node WS endpoint (Ethereum Sepolia)

2. Open `src/main.rs` and set `KEY`.

When the error is caught, the application will display `BUG FOUND! Response: ...`
