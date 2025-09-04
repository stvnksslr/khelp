use anyhow::Result;
use console::style;
use log::{debug, info};

/// Check for updates to khelp
///
/// Connects to GitHub to check if a new version is available.
/// If apply is true, automatically downloads and applies the update.
#[allow(dead_code)]
pub fn handle_update(apply: bool) -> Result<()> {
    debug!("Running update command with apply: {}", apply);

    #[cfg(feature = "self_update")]
    {
        if apply {
            info!("Checking for and applying updates...");

            match crate::utils::update() {
                Ok(()) => {
                    info!("{}", style("Update completed successfully!").green().bold());
                    Ok(())
                }
                Err(e) => {
                    anyhow::bail!("Failed to update: {}", e);
                }
            }
        } else {
            info!("Checking for updates without applying...");

            match crate::utils::check_for_updates() {
                Ok(update_available) => {
                    if update_available {
                        info!(
                            "{}",
                            style("A new version of khelp is available!").green().bold()
                        );
                        info!("Run {} to update", style("khelp update --apply").cyan());
                        Ok(())
                    } else {
                        info!("{}", style("Already at the latest version.").green());
                        Ok(())
                    }
                }
                Err(e) => {
                    anyhow::bail!("Failed to check for updates: {}", e);
                }
            }
        }
    }

    #[cfg(not(feature = "self_update"))]
    {
        anyhow::bail!(
            "The update feature is not enabled in this build. Please install khelp with the 'self_update' feature to enable updates."
        );
    }
}
