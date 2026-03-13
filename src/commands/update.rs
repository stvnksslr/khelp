#[cfg(feature = "self_update")]
use {anyhow::Result, console::style, log::debug};

/// Check for updates to khelp
///
/// Connects to GitHub to check if a new version is available.
/// If apply is true, automatically downloads and applies the update.
#[cfg(feature = "self_update")]
pub fn handle_update(apply: bool) -> Result<()> {
    debug!("Running update command with apply: {}", apply);

    if apply {
        eprintln!("Checking for and applying updates...");

        match crate::utils::update() {
            Ok(()) => {
                eprintln!("{}", style("Update completed successfully!").green().bold());
                Ok(())
            }
            Err(e) => {
                anyhow::bail!("Failed to update: {}", e);
            }
        }
    } else {
        eprintln!("Checking for updates...");

        match crate::utils::check_for_updates() {
            Ok(update_available) => {
                if update_available {
                    eprintln!(
                        "{}",
                        style("A new version of khelp is available!").green().bold()
                    );
                    eprintln!("Run {} to update", style("khelp update --apply").cyan());
                    Ok(())
                } else {
                    eprintln!("{}", style("Already at the latest version.").green());
                    Ok(())
                }
            }
            Err(e) => {
                anyhow::bail!("Failed to check for updates: {}", e);
            }
        }
    }
}

/// Stub function for when self_update feature is not enabled
#[cfg(not(feature = "self_update"))]
#[allow(dead_code)]
pub fn handle_update(_apply: bool) -> anyhow::Result<()> {
    eprintln!(
        "The update feature is not enabled in this build. Please install khelp with the 'self_update' feature to enable updates."
    );
    anyhow::bail!(
        "The update feature is not enabled in this build. Please install khelp with the 'self_update' feature to enable updates."
    );
}
