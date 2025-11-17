# sedidata
League of Legends client connector, displays special statistics

This repository contains a Rust workspace with **two applications**:
1. **`sedidata-tui/`**
   The main desktop application.
   It interacts directly with the League Client (LCU) and handles the majority of logic locally.

2. **`sedidata-server/`**
   A small HTTP server deployed on **Render.com**.
   It performs specific external Riot API lookups (e.g., ranked player info) that cannot be retrieved via LCU.

Both applications share this repository but are built and deployed independently.


## Deployment Overview

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
