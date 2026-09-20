# Ministerium

Ministerium is a self-hosted deployment runner. A signed GitHub `push` webhook
for a repository's `main` branch causes it to clone or pull that repository,
build its Docker image, run it with a dynamic host port, and expose it at
`/<repository-name>/` through NGINX.

For example, `demo-repository` is served at:

```text
https://apps.example.com/demo-repository/
```

## What belongs in Git

The templates in [`deploy/`](deploy/) are safe to commit and are the source
of truth for a new server:

- systemd service templates for Ministerium and Cloudflare Tunnel;
- the NGINX base server configuration;
- the least-privilege sudoers rule;
- an environment-file template.

Do **not** commit real environment files, webhook secrets, Cloudflare Tunnel
tokens, SSH private keys, generated `locations/*.conf` files, cloned apps, or
Docker state.

## Requirements

- A Debian/Ubuntu host with a public ingress. Cloudflare Tunnel is supported.
- A dedicated Linux account that can read a GitHub deploy key and use Docker.
- Rust stable, Docker, NGINX, Git, and `sudo`.
- A GitHub repository SSH key or machine account with read access to every app
  Ministerium should deploy.

Each deployed application must have a `Dockerfile` and must listen on container
port **8080**. Ministerium currently deploys only signed `push` events for
`main`.

## New machine setup

The commands below use `rst` as the service account. Replace it consistently if
you use another account.

### 1. Install host dependencies

```bash
sudo apt update
sudo apt install -y build-essential ca-certificates curl docker.io git nginx

sudo usermod -aG docker rst
```

Log out and back in as `rst` so Docker group membership applies. Then install
Rust as that user:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"
rustc --version
docker ps
```

### 2. Give the service account GitHub access

Create a dedicated deploy key (or use a restricted machine account), add its
public key to the GitHub organization/repositories as read-only, then trust
GitHub's host key:

```bash
sudo -u rst mkdir -p /home/rst/.ssh
sudo -u rst chmod 700 /home/rst/.ssh
sudo -u rst ssh-keygen -t ed25519 -f /home/rst/.ssh/ministerium_deploy -C ministerium
sudo -u rst sh -c 'ssh-keyscan github.com >> /home/rst/.ssh/known_hosts'
```

Configure SSH to use that key in `/home/rst/.ssh/config`:

```sshconfig
Host github.com
    IdentityFile ~/.ssh/ministerium_deploy
    IdentitiesOnly yes
```

Verify it with a repository Ministerium may deploy:

```bash
sudo -u rst git ls-remote git@github.com:YOUR_ORG/YOUR_APP.git HEAD
```

### 3. Clone and build Ministerium

```bash
sudo install -d -o rst -g rst /opt/ministerium
sudo install -d -o rst -g rst /var/lib/ministerium/repos /var/lib/ministerium/docker
sudo -u rst git clone https://github.com/YOUR_ORG/ministerium.git /opt/ministerium
sudo -u rst /bin/bash -lc 'source "$HOME/.cargo/env" && cd /opt/ministerium && cargo build --release'
sudo install -o rst -g rst -m 0755 /opt/ministerium/target/release/ministerium /opt/ministerium/ministerium
```

If the repository is private, clone using an authenticated SSH URL instead.

### 4. Create the protected environment file

```bash
sudo install -d -m 0755 /etc/ministerium
sudo install -m 0600 /dev/null /etc/ministerium/ministerium.env
sudo nano /etc/ministerium/ministerium.env
```

Copy the keys from
[`deploy/environment/ministerium.env.example`](deploy/environment/ministerium.env.example),
set a long random webhook secret, and keep the file owned by `root`. Its
repository root should be:

```dotenv
GITHUB_ROOT_FOLDER=/var/lib/ministerium/repos
DEPLOYMENTS_DB_PATH=/var/lib/ministerium/repos/ministerium.sqlite3
```

### 5. Install NGINX configuration and permissions

```bash
sudo install -d -o rst -g rst /etc/nginx/conf.d/ministerium/locations
sudo install -m 0644 deploy/nginx/ministerium-server.conf /etc/nginx/conf.d/ministerium/server.conf
sudo mv /etc/nginx/sites-enabled/default /etc/nginx/sites-enabled/default.disabled
sudo nginx -t
sudo systemctl reload nginx
```

Install the sudoers rule safely. Replace `{{MINISTERIUM_USER}}` in a temporary
copy with `rst`, then validate and install it with `visudo`:

```bash
sed 's/{{MINISTERIUM_USER}}/rst/g' deploy/sudoers/ministerium | sudo visudo -cf -
sed 's/{{MINISTERIUM_USER}}/rst/g' deploy/sudoers/ministerium | sudo tee /etc/sudoers.d/ministerium >/dev/null
sudo chmod 0440 /etc/sudoers.d/ministerium
sudo visudo -cf /etc/sudoers.d/ministerium
```

### 6. Install and start the systemd service

Replace the three placeholders in a local copy of
[`deploy/systemd/ministerium.service.template`](deploy/systemd/ministerium.service.template):

- `{{MINISTERIUM_USER}}` and `{{MINISTERIUM_GROUP}}` → `rst`;
- `{{MINISTERIUM_HOME}}` → `/home/rst`.

Then install and start it:

```bash
sudo cp deploy/systemd/ministerium.service.template /etc/systemd/system/ministerium.service
sudo nano /etc/systemd/system/ministerium.service
sudo systemctl daemon-reload
sudo systemctl enable --now ministerium
curl -i http://127.0.0.1:8013/heartbeat
```

Expected response: `HTTP/1.1 200 OK` with body `OK`.

### 7. Expose the server through Cloudflare Tunnel (optional)

Create a remotely-managed tunnel and add public hostname routes in Cloudflare:

- `apps.example.com` → `http://localhost:80`
- `ministerium.example.com` → `http://localhost:8013`

Store the tunnel token outside the unit file:

```bash
sudo install -d -m 0700 /etc/cloudflared
sudo nano /etc/cloudflared/tunnel.token
sudo chown root:root /etc/cloudflared/tunnel.token
sudo chmod 0600 /etc/cloudflared/tunnel.token
```

Install
[`deploy/systemd/cloudflared.service.template`](deploy/systemd/cloudflared.service.template)
as `/etc/systemd/system/cloudflared.service`, then:

```bash
sudo systemctl daemon-reload
sudo systemctl enable --now cloudflared
sudo systemctl status cloudflared --no-pager
```

Do not paste a tunnel token into a committed file or a systemd `ExecStart` line.

### 8. Configure an application webhook

In each application repository, go to **Settings → Webhooks → Add webhook**:

- Payload URL: `https://ministerium.example.com/github/webhook`
- Content type: `application/json`
- Secret: the exact `GITHUB_WEBHOOK_SECRET` from the server
- Events: **Just the push event**

Push to `main`. Ministerium responds immediately to GitHub, then deploys in the
background.

## Verify a deployment

For a repository named `my-app`:

```bash
ssh rst@YOUR_SERVER
journalctl -u ministerium -n 100 --no-pager
docker ps --filter 'name=^my-app$'
curl -i http://127.0.0.1/my-app/
```

When using Cloudflare with `apps.example.com → http://localhost:80`, the public
URL is:

```text
https://apps.example.com/my-app/
```

## Updating Ministerium

Run [`update-server.sh`](update-server.sh) from a checked-out copy of this
repository on the server. It only accepts fast-forward Git updates, rebuilds the
release binary, copies the checkout's `.env` to `/etc/ministerium/ministerium.env`,
installs the binary, and restarts the service. The local `.env` is required.

Defaults match the setup above. Override them for a different installation:

```bash
MINISTERIUM_INSTALL_USER=deploy \
MINISTERIUM_INSTALL_GROUP=deploy \
MINISTERIUM_INSTALL_DIR=/opt/ministerium \
./update-server.sh
```

### Create the API token

The API token is created by the server administrator; Ministerium does not
issue one. Generate a value, place it in `/etc/ministerium/ministerium.env`,
and restart the service so systemd reloads the environment:

```bash
openssl rand -hex 32
sudo nano /etc/ministerium/ministerium.env
sudo systemctl restart ministerium
```

Set the generated value as `MINISTERIUM_API_TOKEN`. Keep it private. The
restart endpoint requires it as `Authorization: Bearer <token>`.

### Use the deployment dashboard

Open `/ui` on the Ministerium server, for example:

```text
https://ministerium.example.com/ui
```

The dashboard lists deployments and lets you restart an application such as
`rstharun`. Enter the `MINISTERIUM_API_TOKEN` in the token field before using
the restart button. The token is kept only in the browser session and is sent
only with restart requests.

If port `8013` is not publicly routed, use an SSH tunnel and open
`http://127.0.0.1:8013/ui` locally:

```bash
ssh -L 8013:127.0.0.1:8013 rst@YOUR_SERVER
```

## Operational notes

## Monitoring API

Ministerium maintains its deployment inventory in SQLite. By default, the database
is `ministerium.sqlite3` inside `GITHUB_ROOT_FOLDER`; set `DEPLOYMENTS_DB_PATH`
to store it elsewhere.

- `GET /api/deployments` lists deployed applications.
- `GET /api/deployments/{repository_name}` returns one application, or `404`.
- `GET /api/deployments/{repository_name}/versions` lists its immutable image history.
- `GET /api/metrics` reports live host uptime, load average, CPU core count,
  memory, repository-filesystem usage, and Raspberry Pi CPU temperature when
  available.
- `POST /api/deployments/{repository_name}/restart` restarts an application.
- `POST /api/deployments/{repository_name}/rollback` redeploys the previous
  successfully deployed image; add `?image_tag=<tag>` to select a listed version.
- `GET /openapi.json` serves the OpenAPI definition; browse it at `/swagger`.

The deployment responses exclude repository SSH URLs. Put these monitoring
endpoints behind your normal network or frontend authentication layer before
exposing them publicly.

Set `MINISTERIUM_API_TOKEN` to a long random value before using either control
endpoint, then send `Authorization: Bearer <token>`. The OpenAPI page includes
the GitHub webhook endpoint but does not expose its webhook secret.

- Ministerium has no deployment health check or rollback yet. Watch the
  `ministerium` journal while adding new apps.
- Routes in `/etc/nginx/conf.d/ministerium/locations/` are generated state;
  do not edit them manually.
- The service currently needs narrowly-scoped passwordless `sudo` to validate
  and reload NGINX. The supplied sudoers rule grants only those two commands.
