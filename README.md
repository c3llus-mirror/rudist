# Rudist
A Redis Clone Implemented in Rust

## Overview
**Rudist** is a high-performance, in-memory key-value store that mimics the functionality of Redis, written in Rust. It provides core Redis features like a command engine, memory management, and support for multiple data types, all built with a focus on performance and scalability.

## System Architecture

![System Graph](./misc/system-architecture.svg)

*The system architecture diagram above illustrates the components and their interactions within Rudist.*

## Directory Structure

The project follows a modular directory structure to promote clarity and maintainability:

    .
    ├── Cargo.toml
    ├── src/
    │   ├── main.rs                # entry point, server initialization
    │   ├── config/
    │   │   ├── mod.rs             # config module
    │   │   └── settings.rs        # xxx
    │   │
    │   ├── network/
    │   │   ├── mod.rs             # network module
    │   │   ├── event_loop.rs      # xxx
    │   │   ├── io_multiplexer.rs  # xxx
    │   │   └── server.rs          # xxx
    │   │


## [GOAL] Supported Features

### Data Types
- **Strings**
  - GET, SET operations
  - INCR, DECR operations
  - String operations (APPEND, STRLEN)
  - TTL support

- **Lists**
  - LPUSH, RPUSH operations
  - LPOP, RPOP operations
  - LRANGE operation
  - List length operations

### Core Features
- Single-threaded architecture with event loop
- RESP (Redis Serialization Protocol) support
- Basic key eviction (LRU,LFU policy)
- Memory limits enforcement
- Command pipelining

### Connections
- TCP connections
- Multiple client support
- Basic client connection management

### Commands
#### Key Operations
- DEL
- EXISTS
- EXPIRE
- TTL

#### String Commands
- SET [key] [value]
- GET [key]
- INCR [key]
- DECR [key]
- APPEND [key] [value]
- STRLEN [key]

#### List Commands
- LPUSH [key] [value]
- RPUSH [key] [value]
- LPOP [key]
- RPOP [key]
- LLEN [key]
- LRANGE [key] [start] [stop]

#### Server Commands
- PING
- INFO
- COMMAND
- QUIT

### Memory Management
- Basic memory usage tracking
- Maximum memory limit
- LRU eviction when memory limit is reached

### Error Handling
- Standard Redis error responses
- Connection error handling
- Command syntax validation

---
⚠️ **Note**: This is a simplified implementation for learning purposes. For production use, please refer to the official Redis server.

## Getting Started

To get started with Rudist, follow these steps:

1. **Clone the repository:**
   ```bash
   git clone https://github.com/c3llus/rudist.git
