use crate::tool::{Output, Tool};
use anyhow::Context;
use axum::{
    extract::{OriginalUri, Request, State},
    handler::Handler,
    http::{StatusCode, header},
    middleware::{self, Next},
    response::{IntoResponse, Response},
};
use axum_extra::headers::{Authorization, Header, authorization::Basic};
use clap::{Command, CommandFactory, Parser};
use relative_path::{PathExt, RelativePathBuf};
use std::{
    fs,
    path::{Component, PathBuf},
    str::FromStr,
};
use tracing_subscriber;

#[derive(Parser, Debug)]
#[command(name = "serve")]
pub struct ServeTool {
    /// Path to the directory to serve
    #[arg(short, long, default_value = ".")]
    directory: PathBuf,

    /// Port number the server should listen to
    #[arg(short, long, default_value = "3000")]
    port: u16,

    /// Host address the server should bind to
    #[arg(long, default_value = "127.0.0.1")]
    host: String,

    /// Authentication credentials (username:password)
    #[arg(long)]
    auth: Option<Auth>,
}

#[derive(Debug, Clone)]
struct Auth(String, String);

impl FromStr for Auth {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().split_once(':') {
            Some((u, _)) if u.is_empty() => Err("Username cannot be empty".to_string()),
            Some((_, p)) if p.is_empty() => Err("Password cannot be empty".to_string()),
            Some((u, p)) => Ok(Auth(u.to_string(), p.to_string())),
            _ => Err("Expected a colon separated username password".to_string()),
        }
    }
}

impl Tool for ServeTool {
    fn cli() -> Command {
        ServeTool::command()
    }

    fn execute(&self) -> anyhow::Result<Option<Output>> {
        tracing_subscriber::fmt()
            .with_max_level(tracing::Level::DEBUG)
            .init();

        tokio::runtime::Runtime::new()
            .context("Could not create tokio runtime")?
            .block_on(self.run())
            .context("Could not run server")?;

        Ok(None)
    }
}

impl ServeTool {
    async fn run(&self) -> anyhow::Result<()> {
        let root = self
            .directory
            .canonicalize()
            .context("Could not resolve directory")?;

        let listener = tokio::net::TcpListener::bind(format!("{}:{}", self.host, self.port))
            .await
            .context("Could not setup listener")?;

        let serve_dir = tower_http::services::ServeDir::new(root.clone())
            .append_index_html_on_directories(true)
            .fallback(list_dir.with_state(root.clone()));

        let mut app = axum::Router::new()
            // your app state for other routes if you want
            .with_state(root.clone())
            .fallback_service(serve_dir)
            .layer(tower_http::trace::TraceLayer::new_for_http());

        if let Some(auth) = &self.auth {
            tracing::debug!("auth is enabled");
            app = app.layer(middleware::from_fn_with_state(
                auth.clone(),
                basic_auth_middleware,
            ));
        }

        tracing::info!("listening on {}:{}", self.host, self.port);
        axum::serve(listener, app)
            .await
            .context("Could not serve")?;

        Ok(())
    }
}

async fn list_dir(
    State(ref root): State<PathBuf>,
    OriginalUri(uri): OriginalUri,
) -> Result<impl IntoResponse, impl IntoResponse> {
    // Construct the relative path of the file.
    let mut relative_path = PathBuf::new();
    for part in uri.path().trim_matches('/').split("/") {
        relative_path.push(
            urlencoding::decode(part)
                .map_err(|_| (StatusCode::BAD_REQUEST,))?
                .to_string(),
        );
    }

    // The path cannot container anything but regular names.
    if relative_path.components().any(|c| match c {
        Component::CurDir => true,
        Component::ParentDir => true,
        Component::Prefix(_) => true,
        _ => false,
    }) {
        return Err((StatusCode::BAD_REQUEST,));
    }

    // This the path on the disk of the file/directory.
    let absolute_path = root.join(&relative_path);
    if !absolute_path.exists() {
        return Err((StatusCode::NOT_FOUND,));
    }
    if !absolute_path.is_dir() {
        return Err((StatusCode::BAD_REQUEST,));
    }

    let mut html = String::new();

    // Add .. entry.
    // Check if we are at the root.
    if relative_path.components().count() > 0
        // Resolve the location for the parent path.
        && let Some((location, _)) = absolute_path
            .parent()
            .and_then(|parent| parent.relative_to(root).ok())
            .and_then(|parent_relative| for_listing_item(&parent_relative))
    {
        html.push_str(&format!(
            r#"
                <tr>
                    <td>dir</td>
                    <td><a href="{}">..</a></td>
                </tr>
            "#,
            location,
        ));
    }

    // Get a list of all the entries in the directory.
    let mut items: Vec<_> = fs::read_dir(absolute_path)
        .map_err(|_| (StatusCode::UNPROCESSABLE_ENTITY,))?
        .filter_map(|item| {
            item.map(|entry| entry.path())
                .map(|entry| (entry.is_dir(), entry))
                .ok()
        })
        .collect::<Vec<(bool, PathBuf)>>();

    // Sort them by directories first.
    items.sort();
    items.reverse();

    for (is_dir, path) in items {
        // The location of the entry.
        if let Some(relative_path) = path.relative_to(root).ok()
            && let Some((location, name)) = for_listing_item(&relative_path)
        {
            html.push_str(&format!(
                r#"
                    <tr>
                        <td>{}</td>
                        <td><a href="{}">{}</a></td>
                    </tr>
                "#,
                if is_dir { "dir" } else { "" },
                location,
                name,
            ));
        }
    }

    Ok(axum::response::Html(format!(
        r#"
            <html>
                <head>
                    <style>
                        html, body {{
                            margin: 0;
                            padding: 1em;
                            font-family: monospace;
                        }}

                        tr td:first-child {{
                            color: gray;
                        }}
                    </style>
                </head>
                <body>
                    <table>
                        {}
                    </table>
                </body>
            </html>
        "#,
        html,
    )))
}

// Takes a filesystem path and returns an absolute url encoded url for it
// relative to the root path.
fn for_listing_item(path: &RelativePathBuf) -> Option<(String, String)> {
    let encoded: Vec<_> = path
        .components()
        .map(|comp| comp.as_str())
        .map(|part| urlencoding::encode(part))
        .collect();

    Some((
        format!("/{}", encoded.join("/")),
        path.file_name().unwrap_or("/").to_string(),
    ))
}

// Enforces authentication.
async fn basic_auth_middleware(State(auth): State<Auth>, request: Request, next: Next) -> Response {
    let headers = request.headers().get_all(header::AUTHORIZATION);
    if let Ok(basic) = Authorization::<Basic>::decode(&mut headers.iter()) {
        if basic.username() == auth.0 && basic.password() == auth.1 {
            return next.run(request).await;
        }
    }

    (
        StatusCode::UNAUTHORIZED,
        [(header::WWW_AUTHENTICATE, "Basic realm=\"ut serve\"")],
        "Unauthorized",
    )
        .into_response()
}
