//! Packs de textures : liste, création, export, suppression, et import depuis Modrinth ou
//! CurseForge (l'explorateur de browser.rs). L'édition se fait dans editor.rs.

use super::{dialogs, images, lancer, maj_ligne, modele, modele_textes, App};
use crate::minecraft::install;
use crate::services::packs::{self, PackInfo};
use crate::services::catalogue::ProjetDistant;
use crate::services::{loc, system_info};
use crate::{PackCard, PacksState};
use slint::ComponentHandle;
use std::rc::Rc;

#[derive(Default)]
pub struct Etat {
    liste: Vec<PackInfo>,
    versions: Vec<String>,
    /// projets déjà importés depuis l'explorateur (source:id), pendant ce lancement
    importes: std::collections::HashSet<String>,
}

fn s(app: &App) -> PacksState<'_> {
    app.ui.global::<PacksState>()
}

fn pack(app: &App, i: i32) -> Option<PackInfo> {
    app.packs.borrow().liste.get(i as usize).cloned()
}

pub fn installer(app: &Rc<App>) {
    let st = s(app);
    let a = app.clone();
    st.on_new_pack(move || {
        let a = a.clone();
        lancer(async move { nouveau(&a).await });
    });
    let a = app.clone();
    st.on_validate(move || {
        let a = a.clone();
        lancer(async move { valider(&a).await });
    });
    let a = app.clone();
    st.on_import(move || {
        let a = a.clone();
        lancer(async move { importer(&a).await });
    });
    let a = app.clone();
    st.on_browse(move || {
        super::browser::configurer_packs(&a);
        s(&a).set_view(1);
    });
    let a = app.clone();
    st.on_back(move || {
        s(&a).set_view(0);
        rafraichir(&a);
    });
    let a = app.clone();
    st.on_cancel_create(move || s(&a).set_creating(false));
    let a = app.clone();
    st.on_edit(move |i| {
        let Some(p) = pack(&a, i) else { return };
        let a = a.clone();
        lancer(async move {
            if p.version_mc.is_empty() {
                dialogs::erreur(&a, &loc::t("Pack_VersionManquante"), &loc::t("Pack_VersionManquante_Texte")).await;
                return;
            }
            super::editor::ouvrir(&a, p);
        });
    });
    let a = app.clone();
    st.on_folder(move |i| {
        if let Some(p) = pack(&a, i) {
            system_info::ouvrir_dossier(&p.dossier);
        }
    });
    let a = app.clone();
    st.on_export(move |i| {
        let Some(p) = pack(&a, i) else { return };
        let a = a.clone();
        lancer(async move { exporter(&a, &p).await });
    });
    let a = app.clone();
    st.on_delete(move |i| {
        let Some(p) = pack(&a, i) else { return };
        let a = a.clone();
        lancer(async move { supprimer(&a, &p).await });
    });
}

/// Recharge la liste des packs (nombre de textures modifiées, icône…).
pub fn rafraichir(app: &Rc<App>) {
    let liste = packs::lister();
    let cartes: Vec<PackCard> = liste
        .iter()
        .map(|p| PackCard {
            name: p.nom.clone().into(),
            description: sans_codes(&p.description).into(),
            version: p.version_mc.clone().into(),
            count: format!("{} {}", p.nb_textures, loc::t("Pack_Modifiees")).to_uppercase().into(),
            icon: images::vide(),
            has_icon: false,
            previews: modele(p.apercus.iter().filter_map(|f| apercu(f)).collect()),
        })
        .collect();
    let m = modele(cartes);
    s(app).set_list(m.clone());
    for (i, p) in liste.iter().enumerate() {
        if let Some(img) = p.icone().and_then(|f| images::depuis_fichier(&f, 128)) {
            maj_ligne(&m, i, |c| {
                c.icon = img;
                c.has_icon = true;
            });
        }
    }
    app.packs.borrow_mut().liste = liste;
}

/// Une texture pour l'aperçu : le premier carré (les textures animées sont des bandes verticales).
fn apercu(fichier: &std::path::Path) -> Option<slint::Image> {
    let img = image::open(fichier).ok()?.to_rgba8();
    let cote = img.width().min(img.height()).min(64);
    let carre = image::imageops::crop_imm(&img, 0, 0, cote, cote).to_image();
    Some(slint::Image::from_rgba8(images::tampon(&carre)))
}

/// Importe un .zip choisi par l'utilisateur.
async fn importer(app: &Rc<App>) {
    let Some(fichier) = dialogs::choisir_fichier(&loc::t("Pack_ImportDialogue"), &loc::t("Pack_ExportFiltre"), &["zip"]).await else {
        return;
    };
    match app.bloquant(move || packs::importer(&fichier)).await {
        Ok(_) => rafraichir(app),
        Err(e) => dialogs::erreur(app, &loc::t("Pack_ImportImpossible"), &e.to_string()).await,
    }
}

/// Description sans les codes de couleur de Minecraft (« §a », « §l »…), sur une ligne.
fn sans_codes(texte: &str) -> String {
    let mut sortie = String::new();
    let mut lettres = texte.chars();
    while let Some(c) = lettres.next() {
        match c {
            '§' => {
                lettres.next();
            }
            '\n' => sortie.push(' '),
            _ => sortie.push(c),
        }
    }
    sortie.trim().to_string()
}

// ───────────────────────── Import depuis Modrinth / CurseForge ─────────────────────────

fn cle(p: &ProjetDistant) -> String {
    format!("{}:{}", p.source, p.id)
}

pub fn deja_importe(app: &App, p: &ProjetDistant) -> bool {
    app.packs.borrow().importes.contains(&cle(p))
}

/// Télécharge le pack choisi dans l'explorateur et l'ajoute aux packs, prêt à être modifié.
pub async fn importer_distant(app: &Rc<App>, projet: ProjetDistant) {
    let (source, id, titre) = (projet.source.clone(), projet.id.clone(), projet.titre.clone());
    match app.bg(async move { packs::importer_distant(&source, &id, &titre).await }).await {
        Ok(_) => {
            app.packs.borrow_mut().importes.insert(cle(&projet));
            rafraichir(app);
        }
        Err(e) => dialogs::erreur(app, &loc::t("Pack_ImportImpossible"), &format!("{e:#}")).await,
    }
}

// ───────────────────────── Création ─────────────────────────

async fn nouveau(app: &Rc<App>) {
    let st = s(app);
    st.set_create_name("".into());
    st.set_create_desc("".into());
    st.set_create_info(loc::t("Pack_InfoTemplate").into());
    st.set_create_enabled(true);
    st.set_creating(true);

    if app.packs.borrow().versions.is_empty() {
        match app.bg(install::versions_proposees()).await {
            Ok(v) => {
                st.set_versions(modele_textes(&v));
                st.set_version_index(if v.is_empty() { -1 } else { 0 });
                app.packs.borrow_mut().versions = v;
            }
            Err(e) => dialogs::erreur(app, &loc::t("Home_VersionsIndisponibles"), &e.to_string()).await,
        }
    }
}

async fn valider(app: &Rc<App>) {
    let st = s(app);
    let nom = st.get_create_name().trim().to_string();
    let version = app.packs.borrow().versions.get(st.get_version_index().max(0) as usize).cloned();
    let Some(version) = version.filter(|_| !nom.is_empty()) else {
        st.set_create_info(loc::t("Pack_Manquant").into());
        return;
    };
    let description = st.get_create_desc().trim().to_string();

    st.set_create_enabled(false);
    let progression = app.progression(|ui, texte| ui.global::<PacksState>().set_create_info(texte.into()));
    let v = version.clone();
    let resultat = app
        .bg(async move {
            let (_, format) = packs::template(&v, progression).await?;
            packs::creer_pack(&nom, &description, format, &v)
        })
        .await;
    st.set_create_enabled(true);

    match resultat {
        Ok(dossier) => {
            st.set_creating(false);
            rafraichir(app);
            // On ouvre directement le nouveau pack dans l'éditeur
            let p = app.packs.borrow().liste.iter().find(|p| p.dossier == dossier).cloned();
            if let Some(p) = p {
                super::editor::ouvrir(app, p);
            }
        }
        Err(e) => {
            st.set_create_info("".into());
            dialogs::erreur(app, &loc::t("Pack_CreationImpossible"), &format!("{e:#}")).await;
        }
    }
}

// ───────────────────────── Actions sur un pack ─────────────────────────

async fn supprimer(app: &Rc<App>, p: &PackInfo) {
    let ok = dialogs::confirmer(
        app,
        &loc::t("Pack_Supprimer"),
        &loc::tf("Pack_Supprimer_Texte", &[&p.nom]),
        Some(&loc::t("Commun_Supprimer")),
        None,
        true,
    )
    .await;
    if !ok {
        return;
    }
    if let Err(e) = std::fs::remove_dir_all(&p.dossier) {
        dialogs::erreur(app, &loc::t("Commun_SuppressionImpossible"), &e.to_string()).await;
    }
    rafraichir(app);
}

/// Exporte le pack en .zip (aussi utilisé par l'éditeur).
pub async fn exporter(app: &Rc<App>, p: &PackInfo) {
    let Some(destination) =
        dialogs::enregistrer_fichier(&loc::t("Pack_ExportDialogue"), &format!("{}.zip", p.nom), &loc::t("Pack_ExportFiltre"), "zip").await
    else {
        return;
    };
    let (p2, d2) = (p.clone(), destination.clone());
    match app.bloquant(move || packs::exporter(&p2, &d2)).await {
        Ok(()) => {
            let ouvrir = dialogs::confirmer(
                app,
                &loc::t("Pack_ExportTitre"),
                &loc::tf("Pack_Exporte", &[&p.nom, &destination.display()]),
                Some(&loc::t("Commun_OuvrirDossier")),
                Some(&loc::t("Commun_Fermer")),
                false,
            )
            .await;
            if ouvrir {
                system_info::montrer_fichier(&destination);
            }
        }
        Err(e) => dialogs::erreur(app, &loc::t("Pack_ExportImpossible"), &e.to_string()).await,
    }
}
