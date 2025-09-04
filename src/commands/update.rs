#[cfg(feature = "self_update")]
use {anyhow::Result, console::style, log::debug, log::info};

/// Check for updates to khelp
///
/// Connects to GitHub to check if a new version is available.
/// If apply is true, automatically downloads and applies the update.
#[cfg(feature = "self_update")]
pub fn handle_update(apply: bool) -> Result<()> {
    debug!("Running update command with apply: {}", apply);

    if apply {
        info!("Checking for and applying updates...");
        println!("Checking for and applying updates...");

        match crate::utils::update() {
            Ok(()) => {
                info!("Update completed successfully!");
                println!("{}", style("Update completed successfully!").green().bold());
                Ok(())
            }
            Err(e) => {
                anyhow::bail!("Failed to update: {}", e);
            }
        }
    } else {
        info!("Checking for updates without applying...");
        println!("Checking for updates...");

        match crate::utils::check_for_updates() {
            Ok(update_available) => {
                if update_available {
                    info!("A new version of khelp is available!");
                    println!(
                        "{}",
                        style("A new version of khelp is available!").green().bold()
                    );
                    println!("Run {} to update", style("khelp update --apply").cyan());
                    Ok(())
                } else {
                    info!("Already at the latest version.");
                    println!("{}", style("Already at the latest version.").green());
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
    println!(
        "The update feature is not enabled in this build. Please install khelp with the 'self_update' feature to enable updates."
    );
    anyhow::bail!(
        "The update feature is not enabled in this build. Please install khelp with the 'self_update' feature to enable updates."
    );
}
