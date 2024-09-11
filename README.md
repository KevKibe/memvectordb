# MemVectorDB
An fast in-memory VectorDB in rust.

## ⚡️ Features
- **Fast:** MemVectorDB stores vectors in-memory, ensuring fast insertion and retrieval operations.
- **Vertical Scalability:** With vectors stored in-memory, MemVectorDB can scale vertically based on available system resources.
- **Metadata Support:** Supports metadata storage, beneficial for RAG (Retrieval Augmented Generation) applications and pipelines.
- **Option for Persistence:** Supports full restoration of data from logs.
- **Open Source:** MIT Licensed, free forever.


## 🚀 Usage
### 1. Clone the repository:
```bash
git clone https://github.com/KevKibe/memvectordb.git
```

### 2. Build dependencies:
```bash
make build
```
### 3. Start the DB.
```bash
make run
```
- DB runs on http://localhost:8000

### 4. Restore the DB from logs.
```bash
make run-restore
```
- DB runs on http://localhost:8000

## 🐳 Using Docker

### 1. Pull the Docker image:

- On x86_64 (Intel/AMD) systems:

```bash
docker pull kevkibe/memvectordb:v2.1.0

```
- On ARM-based systems (e.g., M1, M2, M3):
```bash
docker pull --platform linux/amd64 kevkibe/memvectordb:v2.1.0
```

### 2. Run the Docker container:
- On x86_64 (Intel/AMD) systems:
```bash
docker run -it --rm \
    -p 8000:8000 \
    -v /var/memvectordb:/memvectordb \
    kevkibe/memvectordb:v2.1.0
```
- On ARM-based systems (e.g., M1, M2, M3):
```bash
docker run -it --rm \
    --platform linux/amd64 \
    -p 8000:8000 \
    -v /var/memvectordb:/memvectordb \
    kevkibe/memvectordb:v2.1.0
```

### 3. Run the Docker container with DB restoration:
- On x86_64 (Intel/AMD) systems:
```bash
docker run -it --rm \
    -p 8000:8000 \
    -v /var/memvectordb:/memvectordb \
    -e RESTORE_DB=true \
    kevkibe/memvectordb:v2.1.0
```
- On ARM-based systems (e.g., M1, M2, M3):
```bash
docker run -it --rm \
    --platform linux/amd64 \
    -p 8000:8000 \
    -v /var/memvectordb:/memvectordb \
    -e RESTORE_DB=true \
    kevkibe/memvectordb:v2.1.0
```
### 4. DB runs on http://localhost:8000

MemVectorDB Python client: [Docs](https://github.com/KevKibe/memvectordb-python-client/blob/main/README.md)

## Load Tests
- All tests done with 100000 requests on a Macbook Air M1.
### POST
```console
Summary:
  Success rate: 100.00%
  Total:        1.9317 secs
  Slowest:      0.0363 secs
  Fastest:      0.0000 secs
  Average:      0.0010 secs
  Requests/sec: 51766.9796

  Total data:   5.15 MiB
  Size/request: 54 B
  Size/sec:     2.67 MiB
```
### GET
```console
Summary:
  Success rate: 100.00%
  Total:        1.0847 secs
  Slowest:      0.0081 secs
  Fastest:      0.0000 secs
  Average:      0.0005 secs
  Requests/sec: 92191.6443

  Total data:   4.58 MiB
  Size/request: 48 B
  Size/sec:     4.22 MiB
```
### DELETE
```console
Summary:
  Success rate: 100.00%
  Total:        1.0714 secs
  Slowest:      0.0168 secs
  Fastest:      0.0000 secs
  Average:      0.0005 secs
  Requests/sec: 93339.6446

  Total data:   5.05 MiB
  Size/request: 52 B
  Size/sec:     4.72 MiB
```
### PUT
```console
Summary:
  Success rate: 100.00%
  Total:        2.7216 secs
  Slowest:      0.0395 secs
  Fastest:      0.0001 secs
  Average:      0.0014 secs
  Requests/sec: 36743.3094

  Total data:   7.06 MiB
  Size/request: 74 B
  Size/sec:     2.59 MiB
```
