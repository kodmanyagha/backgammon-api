# Tavla Api

Backend for the Tavla (backgammon) game. Built with Rust, Axum, MySQL, Redis.

Handles user authentication, guest sessions, role/permission management, matchmaking,
real-time gameplay over WebSocket, and persistence of game history and scores.

---

# How to run?

```bash
# Run backend service
watchexec -w src -r cargo run --bin tavla_api

# Run DB seeds
cargo run --bin seed
```

Swagger UI is served at `/swagger-ui` once the server is running.
