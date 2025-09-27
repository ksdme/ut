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
    path::{Component, Path, PathBuf},
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

// Resolve the file name on disk based on the url.
// Lifted from https://docs.rs/tower-http/0.6.6/src/tower_http/services/fs/serve_dir/mod.rs.html#453
fn build_and_validate_path(base_path: &Path, requested_path: &str) -> Option<PathBuf> {
    let path = requested_path.trim_start_matches('/');

    let path_decoded = urlencoding::decode(path.as_ref()).ok()?;
    let path_decoded = Path::new(&*path_decoded);

    let mut abs_path = base_path.to_path_buf();
    for component in path_decoded.components() {
        match component {
            Component::Normal(comp) => {
                // Protect against paths like `/foo/c:/bar/baz`
                if Path::new(&comp)
                    .components()
                    .all(|c| matches!(c, Component::Normal(_)))
                {
                    abs_path.push(comp)
                } else {
                    return None;
                }
            }
            Component::CurDir => {}
            Component::Prefix(_) | Component::RootDir | Component::ParentDir => {
                return None;
            }
        }
    }
    Some(abs_path)
}

async fn list_dir(
    State(ref root): State<PathBuf>,
    OriginalUri(uri): OriginalUri,
) -> Result<impl IntoResponse, impl IntoResponse> {
    // Turn the url path into a filesystem path.
    let Some(absolute_path) = build_and_validate_path(root, uri.path()) else {
        return Err((StatusCode::BAD_REQUEST,));
    };
    if !absolute_path.exists() {
        return Err((StatusCode::NOT_FOUND,));
    }
    if !absolute_path.is_dir() {
        return Err((StatusCode::BAD_REQUEST,));
    }

    let mut html = String::new();

    // Add .. entry.
    // Resolve the relative path of the entry.
    if let Some(relative_path) = absolute_path
        .relative_to(root)
        .ok()
        // Check if we are at the root.
        && relative_path.components().count() > 0
        // Resolve the location for the parent path.
        && let Some((location, _)) = relative_path
            .parent()
            .and_then(|relative_parent| for_listing_item(&relative_parent.to_relative_path_buf()))
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
    )
        .into_response()
}
