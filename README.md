# EquiCloud

EquiCloud is a Rust-based version of vencloud using ScyllaDB

## Requirements

<details>
<summary>Docker Installation</summary>

- **Docker**: https://docs.docker.com/engine/install
- **Docker Compose**: https://docs.docker.com/compose/install

</details>

<details>
<summary>Native Installation</summary>

- **Rust**: https://www.rust-lang.org/tools/install
- **ScyllaDB**: 5.4+ or 2025.3.1+ (recommended)

</details>

#### Discord OAuth Setup
1. Go to [Discord Developer Portal](https://discord.com/developers/applications)
2. Create a new application
3. Navigate to OAuth2 settings
4. Add redirect URI: `{SERVER_FQDN}/v1/oauth/callback`
5. Copy Client ID and Secret to your `.env`:

```env
DISCORD_CLIENT_ID=your_client_id_here
DISCORD_CLIENT_SECRET=your_client_secret_here
```

## Docker Installation

1. **Download required files**:
   ```bash
   mkdir equicloud && cd equicloud
   wget https://raw.githubusercontent.com/Equicord/Equicloud/main/compose.yml
   wget https://raw.githubusercontent.com/Equicord/Equicloud/main/.env.example
   ```

2. **Configure environment**:
   ```bash
   cp .env.example .env
   ```

3. **Start the services**:
   ```bash
   docker compose up -d
   ```

The API will be available at `http://{SERVER_HOST}:{SERVER_PORT}` (default: `http://0.0.0.0:9000`).

## Native Installation

### 1. Clone and Setup

```bash
git clone https://github.com/Equicord/Equicloud.git
cd Equicloud
```

### 2. Install ScyllaDB

```bash
docker run --name scylla -p 9042:9042 -d scylladb/scylla:5.4
```

### 3. Configure Environment

```bash
cp .env.example .env
```

### 4. Run the Application

```bash
cargo run

# Or build for production
cargo build --release
./target/release/equicloud
```

### Reverse Proxy Example (nginx)

```nginx
map $http_upgrade $connection_upgrade {
    default upgrade;
    '' close;
}
server {
    listen 80;
    server_name cloud.example.com;
    return 301 https://$host$request_uri;
}
server {
    listen [::]:443 ssl http2;
    listen 443 ssl http2;
    server_name cloud.example.com;
    client_max_body_size 60M;
    ssl_protocols TLSv1.2 TLSv1.3;
    ssl_ciphers ECDHE-RSA-AES128-GCM-SHA256:ECDHE-RSA-AES256-GCM-SHA384:ECDHE-RSA-CHACHA20-POLY1305;
    ssl_prefer_server_ciphers off;
    add_header Strict-Transport-Security "max-age=31536000; includeSubDomains" always;
    add_header X-Frame-Options DENY always;
    add_header X-Content-Type-Options nosniff always;
    add_header Referrer-Policy strict-origin-when-cross-origin always;
    ssl_certificate cloud.example.com/fullchain.cer;
    ssl_certificate_key cloud.example.com/private.key;
    
    location / {
        proxy_pass http://127.0.0.1:9000;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto https;
        proxy_set_header X-Forwarded-Ssl on;
        proxy_set_header X-Forwarded-Port 443;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection $connection_upgrade;
        proxy_cache_bypass $http_upgrade;
        proxy_connect_timeout 30s;
        proxy_send_timeout 30s;
        proxy_read_timeout 30s;
    }
}
```

## License

This project is licensed under the BSD 3-Clause License - see the [LICENSE](LICENSE) file for details.