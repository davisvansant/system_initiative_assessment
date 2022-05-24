#### System Initiative 

---

Backend

Prerequisites:
 - Rust - https://www.rust-lang.org/tools/install

 Tested with `Rust 1.61`

```
cargo run backend --release
```

Frontend

Prerequisites:
 - NodeJS/npm - https://nodejs.org/en/download/

Tested with `Node v18.2.0` and `npm 8.10.0`

```
cd frontend && npm install && npm run dev
```

Testing

Backend

```
cargo test --workspace -- --test-threads=1
```

Frontend

```
cd frontend && npm install && npm run test:e2e:ci
```
