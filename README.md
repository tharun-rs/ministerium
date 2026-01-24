# Ministerium – GitHub Webhook → Docker → NGINX Runner

Ministerium is a lightweight self-hosted runner that:
1. Receives GitHub webhooks
2. Clones or updates a repository
3. Builds it using Docker
4. Runs the container on a dynamic port
5. Exposes it via NGINX using path-based routing

This document covers **all system-level setup** required **outside Rust code**.

---

## System Requirements

- Linux (tested on Ubuntu/Debian)
- Docker
- NGINX
- Git
- Rust (stable, edition 2024)
- `sudo` access (one-time setup)

---

## 1. Install Required Packages

```bash
sudo apt update
sudo apt install -y \
    docker.io \
    nginx \
    git \
    curl \
    build-essential
2. Docker Setup (IMPORTANT)
Add your user to the Docker group
bash
Copy code
sudo usermod -aG docker $USER
Then log out and log back in (or reboot).

Verify:

bash
Copy code
docker ps
This must work without sudo.

3. Rust Toolchain
Install Rust using rustup (recommended):

bash
Copy code
curl https://sh.rustup.rs -sSf | sh
Restart your shell, then verify:

bash
Copy code
rustc --version
cargo --version
4. Directory Layout Required by Ministerium
Ministerium assumes the following directories exist:

Git repositories root
bash
Copy code
mkdir -p ~/repos
Set environment variable:

bash
Copy code
export GITHUB_ROOT_FOLDER="$HOME/repos"
(Optional: add this to ~/.bashrc)

5. NGINX Configuration (CRITICAL)
5.1 Disable Debian default site
Ministerium uses its own default server.

bash
Copy code
sudo mv /etc/nginx/sites-enabled/default \
        /etc/nginx/sites-enabled/default.disabled
5.2 Create Ministerium NGINX directories
bash
Copy code
sudo mkdir -p /etc/nginx/conf.d/ministerium/locations
sudo chown -R $USER:$USER /etc/nginx/conf.d/ministerium
5.3 Create the main NGINX server config
Create:

bash
Copy code
nano /etc/nginx/conf.d/ministerium/server.conf
Paste exactly this:

nginx
Copy code
server {
    listen 80 default_server;
    server_name _;

    include /etc/nginx/conf.d/ministerium/locations/*.conf;
}
Save and exit.

5.4 Verify NGINX includes recursive configs
Ensure /etc/nginx/nginx.conf contains:

nginx
Copy code
include /etc/nginx/conf.d/**/*.conf;
(This is present by default on modern Debian/Ubuntu.)

6. Allow Ministerium to Reload NGINX (SUDOERS)
Ministerium runs as a normal user but needs to reload NGINX.

Edit sudoers:

bash
Copy code
sudo visudo
Add this exact line (replace rst with your username):

ruby
Copy code
rst ALL=(root) NOPASSWD: /usr/sbin/nginx -t, /usr/sbin/nginx -s reload
Verify nginx path:

bash
Copy code
which nginx
7. Restart NGINX
bash
Copy code
sudo nginx -t
sudo systemctl restart nginx
Verify:

bash
Copy code
curl http://localhost
You may see a 404 — that’s expected until apps are deployed.

8. Docker Image Expectations
Repositories handled by Ministerium must:

Contain a Dockerfile

Expose and listen on port 8080 inside the container

Example (NGINX-based app):

dockerfile
Copy code
FROM nginx:alpine
COPY nginx.conf /etc/nginx/nginx.conf
COPY index.html /usr/share/nginx/html/index.html
EXPOSE 8080
CMD ["nginx", "-g", "daemon off;"]
Ministerium runs containers using:

bash
Copy code
docker run -d -p 0:8080 <image>
9. GitHub Webhook Setup
In your GitHub repository:

Go to Settings → Webhooks

Payload URL:

perl
Copy code
http://<your-server>/github/webhook
Content type: application/json

Set a secret

Select events:

Push

Pull request (optional)

The secret is validated using X-Hub-Signature-256.

10. Running Ministerium
From the project root:

bash
Copy code
cargo run
Expected behavior:

Webhook received

Repo cloned or pulled

Docker image built

Container restarted

NGINX route created:

perl
Copy code
http://<host>/<repo-name>/
11. Verifying a Deployment
Check container:

bash
Copy code
docker ps
Check NGINX routing:

bash
Copy code
ls /etc/nginx/conf.d/ministerium/locations
Test:

bash
Copy code
curl http://localhost/<repo-name>/
12. Architecture Summary
One NGINX process

One default server

One location file per app

One Docker container per app

Path-based routing (Cloudflare Tunnel safe)

Notes
Do NOT run cargo with sudo

Do NOT manually edit locations/*.conf

NGINX state is filesystem-driven

Containers are restarted on every deploy

Troubleshooting
Check NGINX config:
bash
Copy code
sudo nginx -T
Check logs:
bash
Copy code
sudo tail -f /var/log/nginx/error.log
Check container port:
bash
Copy code
docker port <container-name>
License
MIT (or your choice)

markdown
Copy code

---

If you want, next I can:
- Add **diagram section** (ASCII or Mermaid)
- Write **developer README vs operator README**
- Add **uninstall / cleanup steps**
- Add **Cloudflare Tunnel specific notes**
- Add **security hardening section**

Just tell me.