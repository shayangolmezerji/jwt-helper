# JWT-Helper (jwth): The Ultimate JWT Swiss Army Knife 🔑

JWT-Helper (`jwth`) is a fast, production-ready CLI tool for working with JSON Web Tokens (JWTs). It provides everything you need to build, decode, verify, scan, and manipulate JWTs from the command line—no more online tools or manual decoding!

Built in Rust for speed and security, `jwth` is your all-in-one JWT companion for developers, pentesters, and security engineers.

---

### ✨ Features

- **All-in-One JWT Toolkit**: Build, decode, verify, scan, diff, and visualize JWTs.
- **Key Management**: Generate RSA, EC, and HMAC keys/secrets.
- **Vulnerability Scanning**: Detect weak algorithms, empty kids, and more.
- **Batch Processing**: Automate JWT operations on files of tokens.
- **Professional CLI**: Fast, robust, and user-friendly interface.
- **JWE/JWK/JWKS/OIDC**: Advanced support for modern JWT workflows.
- **No Dependencies**: Single binary, no Python or Node required.

---

### 🚀 Getting Started

#### 1. Clone the repository

```bash
git clone https://github.com/shayangolmezerji/jwt-helper.git
cd jwt-helper/jwt-cli
```

#### 2. Build and Install

Make sure you have [Rust](https://rustup.rs/) installed.

```bash
cargo install --path .
```

This will install the `jwth` binary to `~/.cargo/bin/`.

#### 3. Add to PATH (if needed)

Add this to your `~/.bashrc` or `~/.zshrc`:

```bash
export PATH="$HOME/.cargo/bin:$PATH"
```

Then run:

```bash
source ~/.bashrc  # or source ~/.zshrc
```

#### 4. Usage Example

```bash
jwth --help
jwth build --alg hs256 --secret mysecret --payload '{"sub":"123","name":"Alice"}' --exp 9999999999
jwth decode <token>
jwth verify --alg hs256 --secret mysecret <token>
jwth vuln-scan <token>
jwth keys generate-hmac --size 256
```

---

### 🧰 Command Overview

- `build`         — Create a new JWT
- `decode`        — Decode a JWT without verifying
- `verify`        — Verify a JWT signature and claims
- `keys`          — Generate RSA, EC, or HMAC keys
- `batch`         — Process a file of JWTs
- `fuzz`          — Brute-force HS* JWTs with a wordlist
- `jwks`          — Verify JWT using JWKS
- `jwk`           — Convert JWK <-> PEM
- `claim-edit`    — Edit claims/headers and resign
- `vuln-scan`     — Scan JWT for vulnerabilities
- `diff`          — Compare two JWTs
- `exp-check`     — Check JWT expiration/time-travel
- `visualize`     — Visualize JWT (json, qr)
- `oidc`          — OIDC/OAuth2 token introspection
- `jwe-encrypt`   — JWE encrypt
- `jwe-decrypt`   — JWE decrypt

---

### 📜 License

This project is licensed under the [DON'T BE A DICK PUBLIC LICENSE](LICENSE.md).

### 👨‍💻 Author

Made with ❤️ by [Shayan Golmezerji](https://github.com/shayangolmezerji)
