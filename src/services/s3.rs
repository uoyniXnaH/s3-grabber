use aws_config::BehaviorVersion;
use aws_sdk_s3::Client;
use aws_types::region::Region;
use std::error::Error;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct S3ObjectSummary {
    pub key: String,
    pub size: u64,
    pub modified: String,
}

#[derive(Debug, Clone)]
pub struct S3ListResult {
    pub prefixes: Vec<String>,
    pub objects: Vec<S3ObjectSummary>,
}

#[derive(Debug, Clone)]
pub struct S3DownloadResult {
    pub bytes_written: u64,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum S3Target {
    Profile { profile: String },
    Endpoint { endpoint_url: String },
    DefaultChain,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum TargetWarning {
    ProfileOverridesEndpoint,
}

#[derive(Debug, Clone)]
pub struct S3ConnectParams {
    pub profile: String,
    pub region: String,
    pub bucket: String,
    pub prefix: String,
    pub endpoint_url: String,
    pub max_keys: i32,
}

pub fn resolve_target(profile: &str, endpoint_url: &str) -> (S3Target, Option<TargetWarning>) {
    let profile = profile.trim();
    let endpoint_url = endpoint_url.trim();

    if !profile.is_empty() {
        let warning = if !endpoint_url.is_empty() {
            Some(TargetWarning::ProfileOverridesEndpoint)
        } else {
            None
        };
        return (
            S3Target::Profile {
                profile: profile.to_string(),
            },
            warning,
        );
    }

    if !endpoint_url.is_empty() {
        return (
            S3Target::Endpoint {
                endpoint_url: endpoint_url.to_string(),
            },
            None,
        );
    }

    (S3Target::DefaultChain, None)
}

pub fn validate_endpoint_url(endpoint_url: &str) -> Result<(), String> {
    let endpoint = endpoint_url.trim();
    if endpoint.is_empty() {
        return Ok(());
    }

    let (scheme, rest) = endpoint
        .split_once("://")
        .ok_or_else(|| "endpoint-url must include scheme (http:// or https://)".to_string())?;

    if scheme != "http" && scheme != "https" {
        return Err("endpoint-url scheme must be http or https".to_string());
    }

    let authority = rest.split('/').next().unwrap_or_default();
    if authority.is_empty() {
        return Err("endpoint-url host is missing".to_string());
    }

    if authority.contains(' ') {
        return Err("endpoint-url must not contain spaces".to_string());
    }

    Ok(())
}

pub fn list_objects_sync(params: &S3ConnectParams) -> Result<S3ListResult, String> {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|err| format!("failed to initialize async runtime: {err}"))?;

    runtime.block_on(list_objects(params))
}

pub fn list_all_objects_sync(params: &S3ConnectParams) -> Result<Vec<S3ObjectSummary>, String> {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|err| format!("failed to initialize async runtime: {err}"))?;

    runtime.block_on(list_all_objects(params))
}

pub fn download_object_to_path_sync(
    params: &S3ConnectParams,
    key: &str,
    destination: &Path,
) -> Result<S3DownloadResult, String> {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|err| format!("failed to initialize async runtime: {err}"))?;

    runtime.block_on(download_object_to_path(params, key, destination))
}

async fn list_objects(params: &S3ConnectParams) -> Result<S3ListResult, String> {
    let (client, target) = build_client(params).await;

    let mut request = client
        .list_objects_v2()
        .bucket(params.bucket.trim())
        .delimiter("/")
        .max_keys(params.max_keys);

    if let Some(prefix) = s3_api_prefix(&params.prefix) {
        request = request.prefix(prefix);
    }

    let response = request
        .send()
        .await
        .map_err(|err| format_s3_error("list_objects_v2", params, &target, &err))?;

    let mut prefixes = response
        .common_prefixes()
        .iter()
        .filter_map(|p| p.prefix().map(ToString::to_string))
        .collect::<Vec<_>>();
    prefixes.sort();

    let mut objects = response
        .contents()
        .iter()
        .filter_map(|obj| {
            let key = obj.key()?.to_string();
            let size = obj.size().unwrap_or(0).max(0) as u64;
            let modified = obj
                .last_modified()
                .map(ToString::to_string)
                .unwrap_or_else(|| "-".to_string());
            Some(S3ObjectSummary {
                key,
                size,
                modified,
            })
        })
        .collect::<Vec<_>>();
    objects.sort_by(|a, b| a.key.cmp(&b.key));

    Ok(S3ListResult { prefixes, objects })
}

async fn list_all_objects(params: &S3ConnectParams) -> Result<Vec<S3ObjectSummary>, String> {
    let (client, target) = build_client(params).await;
    let mut continuation: Option<String> = None;
    let mut out = Vec::new();

    loop {
        let mut request = client
            .list_objects_v2()
            .bucket(params.bucket.trim())
            .max_keys(params.max_keys.max(1_000));

        if let Some(prefix) = s3_api_prefix(&params.prefix) {
            request = request.prefix(prefix);
        }
        if let Some(token) = continuation.as_deref() {
            request = request.continuation_token(token);
        }

        let response = request
            .send()
            .await
            .map_err(|err| format_s3_error("list_objects_v2_all", params, &target, &err))?;

        out.extend(response.contents().iter().filter_map(|obj| {
            let key = obj.key()?.to_string();
            let size = obj.size().unwrap_or(0).max(0) as u64;
            let modified = obj
                .last_modified()
                .map(ToString::to_string)
                .unwrap_or_else(|| "-".to_string());
            Some(S3ObjectSummary {
                key,
                size,
                modified,
            })
        }));

        if response.is_truncated().unwrap_or(false) {
            continuation = response.next_continuation_token().map(ToString::to_string);
            if continuation.is_none() {
                break;
            }
        } else {
            break;
        }
    }

    out.sort_by(|a, b| a.key.cmp(&b.key));
    Ok(out)
}

async fn download_object_to_path(
    params: &S3ConnectParams,
    key: &str,
    destination: &Path,
) -> Result<S3DownloadResult, String> {
    let (client, target) = build_client(params).await;
    let response = client
        .get_object()
        .bucket(params.bucket.trim())
        .key(key)
        .send()
        .await
        .map_err(|err| format_s3_error("get_object", params, &target, &err))?;

    let body = response
        .body
        .collect()
        .await
        .map_err(|err| format_s3_error("get_object.body.collect", params, &target, &err))?;
    let bytes = body.into_bytes();

    if let Some(parent) = destination.parent() {
        fs::create_dir_all(parent).map_err(|err| {
            format!(
                "failed to create destination parent directory {}: {err}",
                parent.display()
            )
        })?;
    }
    fs::write(destination, &bytes)
        .map_err(|err| format!("failed to write {}: {err}", destination.display()))?;

    Ok(S3DownloadResult {
        bytes_written: bytes.len() as u64,
    })
}

fn s3_api_prefix(path: &str) -> Option<String> {
    let trimmed = path.trim().trim_start_matches('/');
    if trimmed.is_empty() {
        return None;
    }

    if trimmed.ends_with('/') {
        Some(trimmed.to_string())
    } else {
        Some(format!("{trimmed}/"))
    }
}

async fn build_client(params: &S3ConnectParams) -> (Client, S3Target) {
    let (target, _) = resolve_target(&params.profile, &params.endpoint_url);
    let mut loader =
        aws_config::defaults(BehaviorVersion::latest()).region(Region::new(params.region.clone()));
    if let S3Target::Profile { profile } = &target {
        loader = loader.profile_name(profile);
    }

    let shared = loader.load().await;
    let mut s3_config = aws_sdk_s3::config::Builder::from(&shared);
    if let S3Target::Endpoint { endpoint_url } = &target {
        s3_config = s3_config.endpoint_url(endpoint_url).force_path_style(true);
    }

    (Client::from_conf(s3_config.build()), target)
}

fn format_s3_error(
    operation: &str,
    params: &S3ConnectParams,
    target: &S3Target,
    err: &impl Error,
) -> String {
    let target_label = match target {
        S3Target::Profile { profile } => format!("aws-profile:{profile}"),
        S3Target::Endpoint { endpoint_url } => format!("endpoint:{endpoint_url}"),
        S3Target::DefaultChain => "default-chain".to_string(),
    };

    let mut message = String::new();
    message.push_str(&format!("operation={operation}\n"));
    message.push_str(&format!("target={target_label}\n"));
    message.push_str(&format!("region={}\n", params.region));
    message.push_str(&format!("bucket={}\n", params.bucket));
    message.push_str(&format!("prefix={}\n", params.prefix));
    message.push_str(&format!("endpoint_url={}\n", params.endpoint_url));
    message.push_str(&format!("error={err}\n"));
    message.push_str(&format!("debug={err:?}"));

    let mut source = err.source();
    let mut index = 0usize;
    while let Some(cause) = source {
        index += 1;
        message.push_str(&format!("\ncause[{index}]={cause}"));
        source = cause.source();
    }

    message
}
