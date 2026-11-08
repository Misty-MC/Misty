//! Options (fenêtre par-dessus le launcher) : apparence, langue, comptes, mémoire, versions,
//! comportement, ressources, dépannage.

use super::{dialogs, lancer, modele, App};
use crate::services::{account, app_info, discord, loc, paths, pseudos, settings, shortcut, storage, system_info};
use crate::{LangItem, PseudoRow, SettingsState, StorageRow};
use slint::ComponentHandle;
use std::path::Path;
use std::rc::Rc;

#[derive(Default)]
pub struct Etat {
    types_modifies: bool,
    langues: Vec<String>,
    stock: Vec<storage::Ligne>,
    stock_ouverts: std::collections::HashSet<String>,
}

fn s(app: &App) -> SettingsState<'_> {
    app.ui.global::<SettingsState>()
}

pub fn installer(app: &Rc<App>) {
    let st = s(app);
    st.set_data_dir(paths::data_dir().to_string_lossy().to_string().into());

    let a = app.clone();
    st.on_add_pseudo(move || {
        let pseudo = s(&a).get_new_pseudo().trim().replace(',', "");
        if pseudo.is_empty() {
            return;
        }
        pseudos::ajouter(&pseudo);
        s(&a).set_new_pseudo("".into());
        charger_pseudos(&a);
    });
    let a = app.clone();
    st.on_remove_pseudo(move |pseudo| {
        let a = a.clone();
        lancer(async move {
            let ok = dialogs::confirmer(
                &a,
                &loc::t("Options_SupprimerCompte"),
                &loc::tf("Options_SupprimerCompte_Texte", &[&pseudo]),
                Some(&loc::t("Commun_Supprimer")),
                None,
                true,
            )
            .await;
            if ok {
                pseudos::supprimer(&pseudo);
                charger_pseudos(&a);
            }
        });
    });

    // Microsoft
    // « Ajouter un compte » : ouvre toujours la fenêtre de connexion
    let a = app.clone();
    st.on_connect(move || {
        let a = a.clone();
        lancer(async move {
            occupe_microsoft(&a, true);
            if let Err(e) = a.bg(account::ajouter()).await {
                dialogs::erreur(&a, &loc::t("Compte_ConnexionImpossible"), &e.to_string()).await;
            }
            occupe_microsoft(&a, false);
            super::home::session_changee(&a);
        });
    });
    let a = app.clone();
    st.on_ms_switch(move |uuid| {
        let a = a.clone();
        lancer(async move { super::home::changer_compte_microsoft(&a, uuid.to_string()).await });
    });
    let a = app.clone();
    st.on_ms_forget(move |uuid| {
        let a = a.clone();
        lancer(async move {
            let pseudo = account::comptes().into_iter().find(|c| c.uuid == uuid.as_str()).map(|c| c.pseudo).unwrap_or_default();
            let ok = dialogs::confirmer(
                &a,
                &loc::t("Compte_Oublier"),
                &loc::tf("Compte_Oublier_Texte", &[&pseudo]),
                Some(&loc::t("Commun_Supprimer")),
                None,
                true,
            )
            .await;
            if ok {
                account::oublier(&uuid);
                super::home::session_changee(&a);
            }
        });
    });
    let a = app.clone();
    st.on_disconnect(move || {
        account::deconnecter();
        super::home::session_changee(&a);
    });
    let a = app.clone();
    st.on_disconnect_all(move || {
        let a = a.clone();
        lancer(async move {
            match account::deconnexion_totale() {
                Ok(()) => dialogs::info(&a, &loc::t("Compte_Oublie"), &loc::t("Compte_Oublie_Texte")).await,
                Err(e) => dialogs::erreur(&a, &loc::t("Compte_ErreurDeconnexion"), &e.to_string()).await,
            }
            super::home::session_changee(&a);
        });
    });

    // Raccourci
    let a = app.clone();
    st.on_shortcut_toggled(move |actif| {
        let a = a.clone();
        lancer(async move { raccourci_change(&a, actif).await });
    });
    let a = app.clone();
    st.on_choose_shortcut(move || {
        let a = a.clone();
        lancer(async move { choisir_dossier_raccourci(&a).await });
    });
    let a = app.clone();
    st.on_recreate_shortcut(move || {
        let a = a.clone();
        lancer(async move {
            let dossier = settings::get(settings::RACCOURCI);
            if dossier.is_empty() {
                choisir_dossier_raccourci(&a).await;
            } else {
                creer_raccourci(&a, &dossier).await;
            }
        });
    });

    // Langue
    let a = app.clone();
    st.on_lang_selected(move |i| {
        let code = a.settings.borrow().langues.get(i as usize).cloned();
        if let Some(code) = code {
            if code != loc::code() {
                settings::set(settings::LANGUE, &code);
                loc::charger(&code);
                super::langue_changee(&a);
            }
        }
    });

    // RAM
    let a = app.clone();
    st.on_ram_changed(move |v| {
        settings::set(settings::RAM, &(v as u32).to_string());
        afficher_ram(&a);
    });

    // Types de versions
    let a = app.clone();
    st.on_types_changed(move || {
        let st = s(&a);
        settings::set_bool(settings::SNAPSHOT, st.get_snapshot());
        settings::set_bool(settings::BETA, st.get_beta());
        settings::set_bool(settings::ALPHA, st.get_alpha());
        a.settings.borrow_mut().types_modifies = true;
    });

    // Stockage
    st.on_open_folder(|| system_info::ouvrir_dossier(&paths::data_dir()));
    let a = app.clone();
    st.on_delete_all(move || {
        let a = a.clone();
        lancer(async move { tout_supprimer(&a).await });
    });

    let a = app.clone();
    st.on_storage_show(move || {
        s(&a).set_storage_open(true);
        stock_analyser(&a);
    });
    let a = app.clone();
    st.on_storage_close(move || {
        s(&a).set_storage_open(false);
        calculer_taille(&a);
    });
    let a = app.clone();
    st.on_storage_toggle(move |cle| {
        let cle = cle.to_string();
        {
            let mut e = a.settings.borrow_mut();
            if !e.stock_ouverts.remove(&cle) {
                e.stock_ouverts.insert(cle);
            }
        }
        stock_afficher(&a);
    });
    let a = app.clone();
    st.on_storage_delete(move |cle| {
        let a = a.clone();
        let cle = cle.to_string();
        lancer(async move { stock_supprimer(&a, cle).await });
    });

    // Discord
    st.on_rich_toggled(|actif| {
        settings::set_bool(settings::RICH_PRESENCE, actif);
        discord::reappliquer();
    });

    let a = app.clone();
    st.on_check_update(move || {
        if s(&a).get_checking_update() {
            return; // recherche déjà en cours
        }
        let a = a.clone();
        lancer(async move { super::verifier_mise_a_jour(&a, true).await });
    });

    st.on_github(|| system_info::ouvrir(&format!("https://github.com/{}", app_info::GITHUB_REPO)));

    // Fenêtre
    st.set_system_text(system_info::version_windows().into());
    let a = app.clone();
    st.on_close(move || fermer(&a));

    // Apparence
    let a = app.clone();
    st.on_theme_selected(move |mode| {
        settings::set(settings::THEME, &mode.to_string());
        a.ui.global::<crate::Theme>().set_mode(mode);
        // La capture floutée montre encore l'ancien thème : on garde seulement le voile
        s(&a).set_has_backdrop(false);
    });
    let a = app.clone();
    st.on_particles_toggled(move |actif| {
        settings::set_bool(settings::PARTICULES, actif);
        a.ui.global::<crate::Theme>().set_particles(actif);
    });
    let a = app.clone();
    st.on_blur_toggled(move |actif| {
        settings::set_bool(settings::FLOU, actif);
        a.ui.global::<crate::Theme>().set_blur(actif);
    });

    let a = app.clone();
    st.on_seasonal_toggled(move |actif| {
        settings::set_bool(settings::SAISON, actif);
        super::appliquer_saison(&a);
        s(&a).set_has_backdrop(false);
    });

    // Ressources
    st.on_downloads_changed(|v| settings::set(settings::TELECHARGEMENTS, &(v.round() as u32).to_string()));

    // Dépannage
    let a = app.clone();
    st.on_clear_cache(move || {
        let a = a.clone();
        lancer(async move {
            let resultat = a.bloquant(|| {
                let cache = paths::data_dir().join("cache");
                if cache.exists() {
                    std::fs::remove_dir_all(&cache)
                } else {
                    Ok(())
                }
            });
            if let Err(e) = resultat.await {
                dialogs::erreur(&a, &loc::t("Param_Cache"), &e.to_string()).await;
            }
            calculer_depannage(&a);
            calculer_taille(&a);
        });
    });
    st.on_open_logs(|| {
        let logs = paths::data_dir().join("logs");
        let _ = std::fs::create_dir_all(&logs);
        system_info::ouvrir_dossier(&logs);
    });
    let a = app.clone();
    st.on_export_debug(move || {
        let fichier = paths::data_dir().join("misty-debug.txt");
        match std::fs::write(&fichier, infos_debug()) {
            Ok(()) => system_info::montrer_fichier(&fichier),
            Err(e) => {
                let a = a.clone();
                lancer(async move { dialogs::erreur(&a, &loc::t("Param_Debug"), &e.to_string()).await });
            }
        }
    });
}

/// Ouvre la fenêtre des options : capture floutée de la fenêtre en fond, puis réglages à jour.
pub fn ouvrir(app: &Rc<App>) {
    let st = s(app);
    if st.get_open() {
        return;
    }
    app.ui.global::<crate::Bulle>().set_visible(false);
    let fond = if app.ui.global::<crate::Theme>().get_blur() { capture_floutee(app) } else { None };
    st.set_has_backdrop(fond.is_some());
    if let Some(image) = fond {
        st.set_backdrop(image);
    }
    rafraichir(app);
    st.set_open(true);
}

/// Ferme la fenêtre des options (et recharge l'accueil si les types de versions ont changé).
pub fn fermer(app: &Rc<App>) {
    s(app).set_open(false);
    if consommer_changement_types(app) {
        super::home::rafraichir(app, true);
    }
}

/// Capture de la fenêtre, réduite puis floutée (le fond « verre dépoli » de la fenêtre des options).
fn capture_floutee(app: &App) -> Option<slint::Image> {
    let capture = app.ui.window().take_snapshot().ok()?;
    let image = image::RgbaImage::from_raw(capture.width(), capture.height(), capture.as_bytes().to_vec())?;
    // Réduire d'abord : le flou est bien plus rapide, et l'agrandissement à l'affichage l'adoucit encore
    let (l, h) = ((image.width() / 6).max(1), (image.height() / 6).max(1));
    let petite = image::imageops::resize(&image, l, h, image::imageops::FilterType::Triangle);
    let floue = image::imageops::blur(&petite, 3.5);
    let tampon = slint::SharedPixelBuffer::<slint::Rgba8Pixel>::clone_from_slice(floue.as_raw(), floue.width(), floue.height());
    Some(slint::Image::from_rgba8(tampon))
}

/// Tailles du cache et des journaux (page Dépannage).
fn calculer_depannage(app: &Rc<App>) {
    let a = app.clone();
    lancer(async move {
        let (cache, logs) = a
            .bloquant(|| {
                let d = paths::data_dir();
                let logs = system_info::taille_dossier(&d.join("logs")) + system_info::taille_dossier(&d.join("crash-reports"));
                (system_info::taille_dossier(&d.join("cache")), logs)
            })
            .await;
        s(&a).set_cache_size(taille_texte(cache).into());
        s(&a).set_logs_size(taille_texte(logs).into());
    });
}

/// Fichier texte à joindre à un signalement de bug.
fn infos_debug() -> String {
    let theme = ["brume", "nuit", "systeme"].get(settings::theme() as usize).copied().unwrap_or("?");
    [
        format!("Misty {}", app_info::VERSION),
        system_info::version_windows(),
        format!("RAM : {} Mo (allouee au jeu : {} Mo)", system_info::ram_totale_mo(), settings::ram_mo()),
        format!("Langue : {}", loc::code()),
        format!("Theme : {theme}"),
        format!("Telechargements simultanes : {}", settings::telechargements()),
        format!("Dossier : {}", paths::data_dir().display()),
        format!("Profils : {}", crate::services::instances::lister().len()),
        format!("Compte Microsoft : {}", if account::session().is_some() { "oui" } else { "non" }),
    ]
    .join("
")
}

/// Recharge l'affichage depuis les fichiers de config (appelé à chaque ouverture de la page).
pub fn rafraichir(app: &Rc<App>) {
    let st = s(app);

    // Langues
    let langues = loc::disponibles();
    let items: Vec<LangItem> = langues
        .iter()
        .map(|l| LangItem { name: l.nom.clone().into(), english: nom_anglais(&l.code).into(), flag: drapeau(&l.pays) })
        .collect();
    st.set_langs(modele(items));
    st.set_lang_index(langues.iter().position(|l| l.code == loc::code()).map(|i| i as i32).unwrap_or(-1));
    app.settings.borrow_mut().langues = langues.into_iter().map(|l| l.code).collect();

    charger_pseudos(app);

    let ram_max = system_info::ram_max_proposee_mo();
    st.set_ram_max(ram_max as f32);
    st.set_ram_value(settings::ram_mo().clamp(1024, ram_max) as f32);

    st.set_snapshot(settings::get_bool(settings::SNAPSHOT, false));
    st.set_beta(settings::get_bool(settings::BETA, false));
    st.set_alpha(settings::get_bool(settings::ALPHA, false));
    st.set_shortcut_enabled(!settings::get(settings::RACCOURCI).is_empty());
    st.set_rich(discord::active());
    st.set_downloads(settings::telechargements() as f32);
    st.set_seasonal(settings::get_bool(settings::SAISON, true));

    maj_textes(app);
    calculer_depannage(app);
}

/// Nom de la langue en anglais, affiché en gris à côté de son nom.
fn nom_anglais(code: &str) -> &'static str {
    match code {
        "fr" => "French",
        "en" => "English",
        "es" => "Spanish",
        "it" => "Italian",
        "ru" => "Russian",
        _ => "",
    }
}

/// Drapeau intégré (assets/flags/xx.png).
fn drapeau(pays: &str) -> slint::Image {
    let octets: &[u8] = match pays {
        "fr" => include_bytes!("../../assets/flags/fr.png"),
        "us" => include_bytes!("../../assets/flags/us.png"),
        "es" => include_bytes!("../../assets/flags/es.png"),
        "ru" => include_bytes!("../../assets/flags/ru.png"),
        "it" => include_bytes!("../../assets/flags/it.png"),
        _ => &[],
    };
    super::images::decoder(octets, 0).map(slint::Image::from_rgba8).unwrap_or_default()
}

/// Textes calculés en code (rappelé au changement de langue).
pub fn maj_textes(app: &Rc<App>) {
    let st = s(app);
    st.set_about_text(loc::tf("Options_APropos_Texte", &[&app_info::VERSION]).into());
    st.set_ram_total_text(loc::tf("Options_Ram_Texte", &[&format!("{:.1}", system_info::ram_totale_mo() as f64 / 1024.0).replace(".0", "")]).into());
    let raccourci = settings::get(settings::RACCOURCI);
    st.set_shortcut_path(if raccourci.is_empty() { loc::t("Options_AucunDossier") } else { raccourci }.into());
    maj_microsoft(app);
    afficher_ram(app);
    calculer_taille(app);
}

/// Indique (une seule fois) si les types de versions ont changé depuis le dernier appel.
pub fn consommer_changement_types(app: &App) -> bool {
    std::mem::take(&mut app.settings.borrow_mut().types_modifies)
}

fn charger_pseudos(app: &Rc<App>) {
    // Trois pseudos par ligne
    let lignes: Vec<PseudoRow> = pseudos::lire()
        .chunks(3)
        .map(|c| PseudoRow { items: super::modele_textes(c) })
        .collect();
    s(app).set_pseudo_rows(modele(lignes));
}

// ───────────────────────── Microsoft ─────────────────────────

pub fn maj_microsoft(app: &App) {
    let session = account::session();
    let st = s(app);
    st.set_ms_state(
        match &session {
            Some(s) => loc::tf("Compte_Connecte", &[&s.pseudo]),
            None => loc::t("Compte_NonConnecte"),
        }
        .into(),
    );
    st.set_ms_connected(session.is_some());
    let comptes: Vec<crate::MsCompte> = account::comptes()
        .into_iter()
        .map(|c| crate::MsCompte { pseudo: c.pseudo.into(), uuid: c.uuid.into(), active: c.actif })
        .collect();
    st.set_ms_accounts(modele(comptes));
}

pub fn occupe_microsoft(app: &App, occupe: bool) {
    s(app).set_ms_busy(occupe);
}

// ───────────────────────── RAM ─────────────────────────

fn afficher_ram(app: &App) {
    let st = s(app);
    st.set_ram_text(format!("{:.0} {}", st.get_ram_value() / 1024.0, loc::t("Unite_Go")).into());
}

// ───────────────────────── Raccourci ─────────────────────────

async fn raccourci_change(app: &Rc<App>, actif: bool) {
    if actif {
        // Pas encore de dossier : on le demande directement
        if settings::get(settings::RACCOURCI).is_empty() {
            choisir_dossier_raccourci(app).await;
        }
        // Choix annulé : on remet l'interrupteur sur off
        if settings::get(settings::RACCOURCI).is_empty() {
            s(app).set_shortcut_enabled(false);
        }
    } else {
        shortcut::supprimer(Path::new(&settings::get(settings::RACCOURCI)));
        settings::set(settings::RACCOURCI, "");
        s(app).set_shortcut_path(loc::t("Options_AucunDossier").into());
    }
}

async fn choisir_dossier_raccourci(app: &Rc<App>) {
    let Some(dossier) = dialogs::choisir_dossier(&loc::t("Options_DossierRaccourci")).await else { return };
    shortcut::supprimer(Path::new(&settings::get(settings::RACCOURCI))); // ancien emplacement
    let texte = dossier.to_string_lossy().to_string();
    settings::set(settings::RACCOURCI, &texte);
    s(app).set_shortcut_path(texte.clone().into());
    creer_raccourci(app, &texte).await;
}

async fn creer_raccourci(app: &App, dossier: &str) {
    if let Err(e) = shortcut::creer(Path::new(dossier)) {
        dialogs::erreur(app, &loc::t("Options_Raccourci"), &loc::tf("Options_RaccourciErreur", &[&e])).await;
    }
}

// ───────────────────────── Stockage ─────────────────────────

fn taille_texte(octets: u64) -> String {
    let mo = octets as f64 / (1024.0 * 1024.0);
    if mo >= 1024.0 {
        format!("{:.2} {}", mo / 1024.0, loc::t("Unite_Go"))
    } else if mo >= 1.0 {
        format!("{mo:.0} {}", loc::t("Unite_Mo"))
    } else {
        format!("{:.0} Ko", octets as f64 / 1024.0)
    }
}

fn calculer_taille(app: &Rc<App>) {
    s(app).set_size_text(loc::t("Options_Calcul").into());
    let a = app.clone();
    lancer(async move {
        let octets = a.bloquant(|| system_info::taille_dossier(&paths::data_dir())).await;
        s(&a).set_size_text(taille_texte(octets).into());
    });
}

/// Nom lisible d'un groupe (« versions » → « Versions de Minecraft »).
fn nom_groupe(cle: &str) -> String {
    loc::t(&format!("Stock_G_{}", cle.trim_start_matches('@')))
}

fn stock_analyser(app: &Rc<App>) {
    s(app).set_storage_busy(true);
    let a = app.clone();
    lancer(async move {
        let lignes = a.bloquant(storage::analyser).await;
        let total: u64 = lignes.iter().filter(|l| l.niveau == 0).map(|l| l.taille).sum();
        s(&a).set_storage_total(taille_texte(total).into());
        a.settings.borrow_mut().stock = lignes;
        stock_afficher(&a);
        s(&a).set_storage_busy(false);
    });
}

/// Reconstruit la liste affichée : les groupes, et le détail des groupes dépliés.
fn stock_afficher(app: &Rc<App>) {
    let e = app.settings.borrow();
    let max = e.stock.iter().filter(|l| l.niveau == 0 && !l.protege).map(|l| l.taille).max().unwrap_or(1).max(1);
    let mut lignes = Vec::new();
    let mut groupe_ouvert = false;
    for l in &e.stock {
        if l.niveau == 0 {
            groupe_ouvert = e.stock_ouverts.contains(&l.cle);
        } else if !groupe_ouvert {
            continue;
        }
        let (nom, info) = if l.niveau == 0 {
            let base = nom_groupe(&l.cle);
            let mut info = loc::t(&format!("Stock_I_{}", l.cle.trim_start_matches('@')));
            if l.reste {
                info = format!("{info} — {}", loc::t("Stock_ResteGroupe"));
            }
            (base, info)
        } else {
            (l.nom.clone(), if l.reste { loc::t("Stock_Reste") } else { String::new() })
        };
        lignes.push(StorageRow {
            cle: l.cle.clone().into(),
            nom: nom.into(),
            info: info.into(),
            size_text: taille_texte(l.taille).into(),
            fraction: l.taille as f32 / max as f32,
            niveau: l.niveau as i32,
            expandable: l.depliable,
            expanded: l.depliable && e.stock_ouverts.contains(&l.cle),
            locked: l.protege,
            reste: l.reste,
        });
    }
    drop(e);
    s(app).set_storage_rows(modele(lignes));
}

async fn stock_supprimer(app: &Rc<App>, cle: String) {
    let (nom, taille) = {
        let e = app.settings.borrow();
        let Some(l) = e.stock.iter().find(|l| l.cle == cle) else { return };
        (if l.niveau == 0 { nom_groupe(&l.cle) } else { l.nom.clone() }, l.taille)
    };
    let avertissement = loc::t(&format!("Stock_W_{}", storage::groupe_de(&cle)));
    let texte = format!("{}\n\n{avertissement}", loc::tf("Stock_Confirm_Texte", &[&nom, &taille_texte(taille)]));
    let ok = dialogs::confirmer(app, &loc::tf("Stock_Confirm_Titre", &[&nom]), &texte, Some(&loc::t("Stock_Supprimer")), None, true).await;
    if !ok {
        return;
    }
    s(app).set_storage_busy(true);
    let c = cle.clone();
    let resultat = app.bloquant(move || storage::supprimer(&c)).await;
    if let Err(e) = resultat {
        dialogs::erreur(app, &loc::t("Options_SuppressionIncomplete"), &e.to_string()).await;
    }
    stock_analyser(app);
}

async fn tout_supprimer(app: &Rc<App>) {
    let ok = dialogs::confirmer(
        app,
        &loc::t("Options_ToutSupprimer"),
        &loc::t("Options_ToutSupprimer_Texte"),
        Some(&loc::t("Options_ToutSupprimer")),
        None,
        true,
    )
    .await;
    if !ok {
        return;
    }
    let resultat = app.bloquant(|| {
        let d = paths::data_dir();
        if d.exists() {
            std::fs::remove_dir_all(d)
        } else {
            Ok(())
        }
    });
    if let Err(e) = resultat.await {
        dialogs::erreur(app, &loc::t("Options_SuppressionIncomplete"), &e.to_string()).await;
    }
    calculer_taille(app);

    if dialogs::confirmer(
        app,
        &loc::t("Options_FermerMisty"),
        &loc::t("Options_FermerMisty_Texte"),
        Some(&loc::t("Commun_Fermer")),
        Some(&loc::t("Options_Rester")),
        false,
    )
    .await
    {
        let _ = slint::quit_event_loop();
    }
}
