use zed_extension_api::{self as zed};

#[derive(Default)]
struct SqruffExtension {
    cached_binary_path: Option<String>,
}

impl zed::Extension for SqruffExtension {
    fn new() -> Self {
        Self::default()
    }

    fn language_server_command(
        &mut self,
        language_server_id: &zed::LanguageServerId,
        worktree: &zed::Worktree,
    ) -> zed::Result<zed::Command> {
        let command = language_server_binary(self, language_server_id, worktree)?;

        Ok(zed::Command {
            command,
            args: vec!["lsp".to_string()],
            env: Vec::new(),
        })
    }
}

fn language_server_binary(
    extension: &mut SqruffExtension,
    language_server_id: &zed::LanguageServerId,
    worktree: &zed::Worktree,
) -> zed::Result<String> {
    if let Some(path) = worktree.which("sqruff") {
        return Ok(path);
    }

    zed::set_language_server_installation_status(
        language_server_id,
        &zed::LanguageServerInstallationStatus::CheckingForUpdate,
    );

    let release = zed::latest_github_release(
        "quarylabs/sqruff",
        zed::GithubReleaseOptions {
            require_assets: true,
            pre_release: false,
        },
    )?;

    let (platform, arch) = zed::current_platform();
    let asset_stem = format!(
        "sqruff-{os}-{arch}",
        os = match platform {
            zed::Os::Mac => "darwin",
            zed::Os::Linux => "linux",
            zed::Os::Windows => "windows",
        },
        arch = match arch {
            zed::Architecture::Aarch64 if platform == zed::Os::Linux => "aarch64-musl",
            zed::Architecture::Aarch64 => "aarch64",
            zed::Architecture::X86 => "x86",
            zed::Architecture::X8664 if platform == zed::Os::Linux => "x86_64-musl",
            zed::Architecture::X8664 => "x86_64",
        },
    );

    let asset_name = format!(
        "{asset_stem}.{suffix}",
        suffix = match platform {
            zed::Os::Windows => "zip",
            _ => "tar.gz",
        }
    );

    let asset = release
        .assets
        .iter()
        .find(|asset| asset.name == asset_name)
        .ok_or_else(|| format!("no asset found matching {:?}", asset_name))?;

    let version_dir = format!("sqruff-{}", release.version);
    let binary_path = format!("{version_dir}/sqruff");

    if !std::fs::metadata(&binary_path).is_ok_and(|stat| stat.is_file()) {
        zed::set_language_server_installation_status(
            language_server_id,
            &zed::LanguageServerInstallationStatus::Downloading,
        );
        let file_kind = match platform {
            zed::Os::Windows => zed::DownloadedFileType::Zip,
            _ => zed::DownloadedFileType::GzipTar,
        };
        zed::download_file(&asset.download_url, &version_dir, file_kind)
            .map_err(|e| format!("failed to download file: {e}"))?;
        let entries =
            std::fs::read_dir(".").map_err(|e| format!("failed to list working directory {e}"))?;
        for entry in entries {
            let entry = entry.map_err(|e| format!("failed to load directory entry {e}"))?;
            if entry.file_name().to_str() != Some(&version_dir) {
                std::fs::remove_dir_all(entry.path()).ok();
            }
        }
    }

    extension.cached_binary_path = Some(binary_path.clone());
    Ok(binary_path)
}

zed::register_extension!(SqruffExtension);
