use super::{resolve_profile, AppState, CommandError};
use preflight_core::{
    engine::Platform,
    host::SystemHost,
    install::{git_action, install_git, FixAction, InstallError},
    process::SystemRunner,
    Profile, Report,
};
use serde::Serialize;
use std::time::{Duration, Instant};
pub struct PendingFix {
    nonce: String,
    canonical: String,
    check_id: String,
    created: Instant,
}
#[derive(Serialize)]
pub struct PreparedFix {
    nonce: String,
    action: FixAction,
}
#[derive(Serialize)]
pub struct InstallOutcome {
    pub message_sl: &'static str,
    pub report: Report,
    pub uncertain: bool,
}
fn err(message_sl: &'static str) -> CommandError {
    CommandError { message_sl }
}
fn allowed(profile: &Profile, report: &Report, id: &str) -> Result<FixAction, CommandError> {
    let check = profile
        .checks()
        .iter()
        .find(|c| c.id == id)
        .ok_or_else(|| err("Neznana zahteva."))?;
    let result = report
        .results
        .iter()
        .find(|r| r.check_id == id)
        .ok_or_else(|| err("Najprej preveri okolje."))?;
    git_action(check, result, std::env::consts::OS, std::env::consts::ARCH)
        .ok_or_else(|| err("Za to zahtevo ni podprtega samodejnega popravka. Uporabi ročni vodič."))
}
#[tauri::command]
pub async fn prepare_fix(
    profile_id: String,
    profile_yaml: Option<String>,
    check_id: String,
    state: tauri::State<'_, AppState>,
) -> Result<PreparedFix, CommandError> {
    let _gate = state
        .gate
        .try_lock()
        .map_err(|_| err("Drugo opravilo že poteka."))?;
    if *state.install_uncertain.lock().await {
        return Err(err("Prejšnja namestitev ima neznan izid. Najprej ročno preveri, ali namestitveni program še deluje."));
    }
    let profile = resolve_profile(&profile_id, profile_yaml)?;
    let canonical = profile.to_yaml()?;
    if state.last_profile.lock().await.as_deref() != Some(canonical.as_str()) {
        return Err(err("Najprej preveri celoten nespremenjeni profil."));
    }
    let report = state
        .last
        .lock()
        .await
        .clone()
        .ok_or_else(|| err("Najprej preveri okolje."))?;
    let action = allowed(&profile, &report, &check_id)?;
    let nonce = uuid::Uuid::new_v4().to_string();
    *state.pending_fix.lock().await = Some(PendingFix {
        nonce: nonce.clone(),
        canonical,
        check_id,
        created: Instant::now(),
    });
    Ok(PreparedFix { nonce, action })
}
fn consume(
    pending: Option<PendingFix>,
    nonce: &str,
    confirmed: bool,
) -> Result<PendingFix, CommandError> {
    let p = pending.ok_or_else(|| err("Najprej odpri pregled namestitve."))?;
    if !confirmed || p.nonce != nonce || p.created.elapsed() > Duration::from_secs(300) {
        return Err(err(
            "Potrditev ni veljavna ali je potekla. Ponovno odpri pregled namestitve.",
        ));
    }
    Ok(p)
}
#[tauri::command]
pub async fn confirm_fix(
    nonce: String,
    confirmed: bool,
    state: tauri::State<'_, AppState>,
) -> Result<InstallOutcome, CommandError> {
    let _gate = state
        .gate
        .try_lock()
        .map_err(|_| err("Drugo opravilo že poteka."))?;
    let pending = consume(state.pending_fix.lock().await.take(), &nonce, confirmed)?;
    if *state.install_uncertain.lock().await {
        return Err(err("Prejšnja namestitev še nima potrjenega izida."));
    }
    if state.last_profile.lock().await.as_deref() != Some(pending.canonical.as_str()) {
        return Err(err(
            "Profil ali pregled se je spremenil. Ponovno preveri okolje.",
        ));
    }
    let profile = Profile::parse(&pending.canonical)?;
    let previous = state
        .last
        .lock()
        .await
        .clone()
        .ok_or_else(|| err("Najprej preveri okolje."))?;
    allowed(&profile, &previous, &pending.check_id)?;
    let outcome = install_git(&SystemRunner).await;
    let uncertain = matches!(
        outcome,
        Err(InstallError::Timeout | InstallError::OutputLimit)
    );
    *state.install_uncertain.lock().await = uncertain;
    let updated = preflight_core::engine::check_selected(
        &profile,
        &pending.check_id,
        &SystemRunner,
        Platform::current(),
        &SystemHost,
        false,
    )
    .await?;
    let report = Report::new(
        previous
            .results
            .into_iter()
            .map(|r| {
                if r.check_id == pending.check_id {
                    updated.clone()
                } else {
                    r
                }
            })
            .collect(),
    );
    *state.last.lock().await = Some(report.clone());
    Ok(InstallOutcome{report,uncertain,message_sl:match outcome{Ok(())=>"Namestitveni program se je zaključil. Preverba Git je ponovljena; če PATH še ni osvežen, znova odpri Preflight.",Err(InstallError::ManagerUnavailable)=>"Winget ni dosegljiv. Uporabi ročni vodič; nič ni bilo nameščeno.",Err(InstallError::Timeout|InstallError::OutputLimit)=>"Izid namestitve ni znan. Namestitveni podproces lahko še deluje; ne ponavljaj namestitve. Ročno preveri stanje in po potrebi znova zaženi aplikacijo.",Err(InstallError::Failed)=>"Namestitev ni bila uspešno potrjena. Preveri dovoljenja, pogoje vira in omrežje; uporabi ročni vodič."}})
}
#[cfg(test)]
mod tests {
    use super::*;
    fn pending() -> PendingFix {
        PendingFix {
            nonce: "nonce".into(),
            canonical: "".into(),
            check_id: "git".into(),
            created: Instant::now(),
        }
    }
    #[test]
    fn requires_exact_fresh_explicit_confirmation() {
        assert!(consume(None, "nonce", true).is_err());
        assert!(consume(Some(pending()), "nonce", false).is_err());
        assert!(consume(Some(pending()), "wrong", true).is_err());
        let mut p = pending();
        p.created = Instant::now() - Duration::from_secs(301);
        assert!(consume(Some(p), "nonce", true).is_err());
        assert!(consume(Some(pending()), "nonce", true).is_ok())
    }
}
