use axum::{response::Html, Json};
use serde_json::{json, Value};

pub async fn openapi() -> Json<Value> {
    Json(json!({
        "openapi": "3.0.3", "info": {"title": "Ministerium API", "version": "1.0.0"},
        "paths": {
            "/heartbeat": {"get": {"summary": "Health check"}},
            "/github/webhook": {"post": {"summary": "GitHub webhook receiver", "description": "Requires GitHub HMAC signature."}},
            "/api/deployments": {"get": {"summary": "List deployment inventory"}},
            "/api/deployments/{repository_name}": {"get": {"summary": "Get a deployment", "parameters": [{"name": "repository_name", "in": "path", "required": true, "schema": {"type": "string"}}]}},
            "/api/deployments/{repository_name}/versions": {"get": {"summary": "List immutable image versions"}},
            "/api/deployments/{repository_name}/restart": {"post": {"summary": "Restart a deployed container", "security": [{"bearerAuth": []}]}},
            "/api/deployments/{repository_name}/rollback": {"post": {"summary": "Roll back to the previous image, or ?image_tag= for a selected version", "security": [{"bearerAuth": []}]}},
            "/api/metrics": {"get": {"summary": "Get Raspberry Pi host metrics"}}
        },
        "components": {"securitySchemes": {"bearerAuth": {"type": "http", "scheme": "bearer"}}}
    }))
}

pub async fn swagger() -> Html<&'static str> {
    Html(r#"<!doctype html><html><head><title>Ministerium API</title><link rel="stylesheet" href="https://unpkg.com/swagger-ui-dist@5/swagger-ui.css"></head><body><div id="swagger-ui"></div><script src="https://unpkg.com/swagger-ui-dist@5/swagger-ui-bundle.js"></script><script>SwaggerUIBundle({url:'/openapi.json',dom_id:'#swagger-ui',persistAuthorization:true});</script></body></html>"#)
}

pub async fn ui() -> Html<&'static str> {
        Html(r###"<!doctype html>
<html lang="en">
<head>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1">
    <title>Ministerium Control</title>
    <style>
        :root { color-scheme: light; --ink: #17211b; --muted: #68756d; --line: #d8e0da; --paper: #f5f7f2; --panel: #ffffff; --accent: #c6532d; --accent-dark: #96391f; --ok: #28734b; }
        * { box-sizing: border-box; }
        body { margin: 0; min-height: 100vh; background: radial-gradient(circle at 90% 0%, #e9d7c5 0, transparent 32%), var(--paper); color: var(--ink); font: 16px/1.5 Georgia, serif; }
        main { width: min(980px, calc(100% - 32px)); margin: 0 auto; padding: 48px 0 64px; }
        header { display: flex; justify-content: space-between; gap: 24px; align-items: end; margin-bottom: 32px; }
        h1 { margin: 0; font-size: clamp(2.2rem, 6vw, 4.4rem); line-height: .95; letter-spacing: 0; max-width: 560px; }
        .kicker { margin: 0 0 10px; color: var(--accent-dark); font: 700 12px/1.2 system-ui, sans-serif; letter-spacing: 0; text-transform: uppercase; }
        .lede { max-width: 320px; margin: 0; color: var(--muted); }
        .toolbar { display: flex; flex-wrap: wrap; gap: 10px; align-items: center; padding: 16px; border-top: 1px solid var(--line); border-bottom: 1px solid var(--line); }
        input { flex: 1 1 300px; min-width: 0; padding: 11px 12px; border: 1px solid var(--line); background: var(--panel); color: var(--ink); font: 14px system-ui, sans-serif; }
        button { border: 0; padding: 11px 16px; background: var(--ink); color: white; cursor: pointer; font: 700 13px system-ui, sans-serif; }
        button:hover { background: var(--accent-dark); } button:disabled { cursor: wait; opacity: .55; }
        #message { min-height: 24px; margin: 18px 0 10px; color: var(--muted); font: 14px system-ui, sans-serif; }
        .grid { display: grid; grid-template-columns: repeat(auto-fit, minmax(280px, 1fr)); gap: 14px; }
        .card { padding: 20px; border: 1px solid var(--line); background: rgba(255,255,255,.82); }
        .card-head { display: flex; justify-content: space-between; align-items: start; gap: 12px; }
        h2 { margin: 0; font-size: 1.55rem; overflow-wrap: anywhere; }
        .status { color: var(--ok); font: 700 11px system-ui, sans-serif; text-transform: uppercase; }
        dl { display: grid; grid-template-columns: auto 1fr; gap: 6px 16px; margin: 22px 0; font: 13px system-ui, sans-serif; }
        dt { color: var(--muted); } dd { margin: 0; text-align: right; overflow-wrap: anywhere; }
        .empty { color: var(--muted); }
        @media (max-width: 620px) { main { padding-top: 30px; } header { display: block; } .lede { margin-top: 16px; } }
    </style>
</head>
<body>
    <main>
        <header>
            <div><p class="kicker">Deployment control</p><h1>Ministerium</h1></div>
            <p class="lede">See deployed applications and restart a running container without leaving this page.</p>
        </header>
        <section class="toolbar" aria-label="Controls">
            <input id="token" type="password" autocomplete="off" placeholder="API token for restart actions">
            <button id="refresh" type="button">Refresh</button>
        </section>
        <p id="message" role="status"></p>
        <section id="deployments" class="grid" aria-live="polite"></section>
    </main>
    <script>
        const tokenInput = document.querySelector('#token');
        const deployments = document.querySelector('#deployments');
        const message = document.querySelector('#message');
        tokenInput.value = sessionStorage.getItem('ministerium-api-token') || '';
        tokenInput.addEventListener('input', () => sessionStorage.setItem('ministerium-api-token', tokenInput.value));

        function showMessage(text, isError = false) { message.textContent = text; message.style.color = isError ? '#96391f' : ''; }
        function card(deployment) {
            const element = document.createElement('article');
            element.className = 'card';
            element.innerHTML = `<div class="card-head"><h2></h2><span class="status">running</span></div><dl><dt>Image</dt><dd class="image"></dd><dt>Port</dt><dd class="port"></dd><dt>Updated</dt><dd class="updated"></dd></dl><button class="restart" type="button">Restart app</button>`;
            element.querySelector('h2').textContent = deployment.repository_name;
            element.querySelector('.image').textContent = deployment.image_tag;
            element.querySelector('.port').textContent = `127.0.0.1:${deployment.host_port}`;
            element.querySelector('.updated').textContent = new Date(deployment.updated_at * 1000).toLocaleString();
            element.querySelector('.restart').addEventListener('click', async () => {
                const button = element.querySelector('.restart');
                const token = tokenInput.value.trim();
                if (!token) { showMessage('Enter MINISTERIUM_API_TOKEN before restarting.', true); tokenInput.focus(); return; }
                button.disabled = true;
                showMessage(`Restarting ${deployment.repository_name}...`);
                const response = await fetch(`/api/deployments/${encodeURIComponent(deployment.repository_name)}/restart`, { method: 'POST', headers: { Authorization: `Bearer ${token}` } });
                button.disabled = false;
                if (!response.ok) { showMessage(`Restart failed (${response.status}). Check the token and service logs.`, true); return; }
                showMessage(`${deployment.repository_name} restarted.`);
            });
            return element;
        }
        async function loadDeployments() {
            showMessage('Loading deployments...');
            try {
                const response = await fetch('/api/deployments');
                if (!response.ok) throw new Error(`HTTP ${response.status}`);
                const items = await response.json();
                deployments.replaceChildren(...items.map(card));
                if (!items.length) { deployments.innerHTML = '<p class="empty">No deployments have been recorded yet.</p>'; }
                showMessage(`${items.length} deployment${items.length === 1 ? '' : 's'} loaded.`);
            } catch (error) { deployments.replaceChildren(); showMessage(`Could not load deployments: ${error.message}`, true); }
        }
        document.querySelector('#refresh').addEventListener('click', loadDeployments);
        loadDeployments();
    </script>
</body>
</html>"###)
}
