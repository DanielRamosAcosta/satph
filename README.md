
# Satph


A Node.js-powered authentication proxy for SSH, FTP, and DAV, integrating with Authelia for secure multi-factor authentication. Node.js 24+ can run TypeScript directly.

## Features
- Fast Node.js server (TypeScript, Node.js 24+)
- First and second factor authentication via Authelia
- Healthcheck endpoint (`/health`) to verify connectivity
- Simple API for authentication requests
- Docker-ready for easy deployment

## Endpoints

### `/auth` (POST)
Authenticate a user with username, password, and protocol.

**Request JSON:**
```json
{
  "username": "youruser",
  "password": "yourpasswordTOTP",
  "ip": "client-ip",
  "protocol": "SSH|FTP|DAV"
}
```
- `password` should be the user's password followed by their 6-digit TOTP code.

**Response:**
- `{ status: 1 }` on success
- `{ status: 0, message: "..." }` on failure

### `/health` (GET)
Checks connectivity to Authelia.

**Response:**
- `{ status: "ok", authelia: "reachable" }` if healthy
- `{ status: "fail", authelia: "...error..." }` if not

## Environment Variables
- `AUTHELIA_BASE_URL`: Base URL for your Authelia instance (required)

## Running Locally
1. Install dependencies:
	```fish
	npm install
	```
2. Set environment variable:
	```fish
	set -x AUTHELIA_BASE_URL "https://your-authelia-url"
	```
3. Start the server (Node.js 24+ runs TypeScript directly):
	```fish
	node index.ts
	```

## Docker Usage
Build and run with Docker:
```fish
docker build -t satph .
docker run -e AUTHELIA_BASE_URL="https://your-authelia-url" -p 3000:3000 satph
```

## Project Structure
- `index.ts` - Main entry point
- `src/authelia.ts` - Authelia integration
- `src/app.ts` - API routes

## License
MIT

