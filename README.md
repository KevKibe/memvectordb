# MemVectorDB
An fast in-memory VectorDB in rust.

## ⚡️ Features
- Fast: MemVectorDB stores vectors in-memory, ensuring fast insertion and retrieval operations.
- Vertical Scalability: With vectors stored in-memory, MemVectorDB can scale vertically based on available system resources.
- Metadata Support: Supports metadata storage, beneficial for RAG (Retrieval Augmented Generation) applications and pipelines.
- Open Source: MIT Licensed, free forever.


## 🚀 Usage
1. Clone the repository:
```bash
git clone https://github.com/KevKibe/memvectordb.git
```

2. Build dependencies:
```bash
make build
```
3. Start the server
```bash
make run
```
- Server runs on http://localhost:8000


## 🐳 Using Docker

1. Pull the Docker image:

- On x86_64 (Intel/AMD) systems:

```bash
docker pull kevkibe/memvectordb

```
- On ARM-based systems (e.g., M1, M2, M3):
```bash
docker pull --platform linux/amd64 kevkibe/memvectordb
```

2. Run the Docker container:
- On x86_64 (Intel/AMD) systems:
```bash
docker run -p 8000:8000 kevkibe/memvectordb
```
- On ARM-based systems (e.g., M1, M2, M3):
```bash
docker run -p 8000:8000 --platform linux/amd64 kevkibe/memvectordb
```
3. Server runs on http://localhost:8000

MemVectorDB Python client: [Docs](https://github.com/KevKibe/memvectordb-python-client/blob/main/README.md)

### In Development
- semantic search