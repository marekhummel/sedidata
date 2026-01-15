# sedidata
[![Github Actions](https://img.shields.io/github/actions/workflow/status/marekhummel/sedidata/ci.yml?branch=main&style=for-the-badge)](https://github.com/marekhummel/sedidata/actions)&emsp;
[![Github Release](https://img.shields.io/github/v/release/marekhummel/sedidata?style=for-the-badge)](https://github.com/marekhummel/sedidata/releases)&emsp;

League of Legends client connector, displays special statistics

This repository contains a Rust workspace with **two applications**:
1. **`sedidata-tui/`**
   The main desktop application.
   It interacts directly with the League Client (LCU) and handles the majority of logic locally.

2. **`sedidata-server/`**
   A small HTTP server deployed on **Render.com**.
   It performs specific external Riot API lookups (e.g., ranked player info) that cannot be retrieved via LCU.
car
Both applications share this repository but are built and deployed independently.


## Etymology
- Sanskrit “Siddhi” = mastery
- Latin “Sedes” = seat/position (→ rank)
- Latin “Sedulus" = diligent, skillful, persistent
- => "Sedi"


## Deployment Overview

## Build
Cargo uses the systems default target, which is usually `stable-x86_64-unknown-linux-gnu`.
To allow automatic deployment of the server on Render, this cannot be changed.
Since the TUI application is meant for usage under Windows, a build target of `x86_64-pc-windows-gnu` is recommended.
Thus, building with `cargo build --release` builds for linux, but for releases one can use:
`cargo build --release --package sedidata-tui --target x86_64-pc-windows-gnu`

### CI/CD Pipeline

* **On every push**
  Both applications are built to ensure they compile correctly.

* **On tag pushes (`v*`)**
  The `rust-service` binary is built in release mode,
  and a downloadable release artifact is published automatically.

* **On pushes to `main`**
  A deployment of the `sedidata-server` service is triggered on Render, iff any code changes
  were detected.
  Render then pulls the latest commit and builds the server internally.

### Required GitHub Secrets

The CI/CD workflow requires two secrets:

| Secret Name         | Purpose                                                    |
| ------------------- | ---------------------------------------------------------- |
| `RENDER_API_KEY`    | Authenticates GitHub when triggering deployments on Render |
| `RENDER_SERVICE_ID` | Identifies the specific Render service to deploy           |


## Server Runtime Secret (Riot API Key)

The `sedidata-server` service uses a **Riot API Key** to perform ranked info lookups.

This must be stored securely in Render as an environment variable:

```
RIOT_API_KEY = <your-riot-api-key>
```
