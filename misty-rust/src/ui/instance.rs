//! Détail d'un profil : contenu installé (mods, packs, datapacks, shaders) et explorateur Modrinth / CurseForge.

use super::{browser, dialogs, images, lancer, maj_ligne, modele, modele_textes, App};
use crate::services::catalogue::{ProjetDistant, MODRINTH};
use crate::services::content::{self, ContenuInstalle, MajDispo};
use crate::services::instances::{self, Instance};
use crate::services::{client, export, loc, packs, system_info};
use crate::{BadgeItem, ContentItem, InstanceState};
use slint::{Color, ComponentHandle, ModelRc};
use std::collections::{HashMap, HashSet};
use std::rc::Rc;

#[derive(Default)]
pub struct Etat {
    pub instance: Option<Instance>,
    contenus: Vec<ContenuInstalle>,
    /// Index (dans `contenus`) des lignes affichées
    visibles: Vec<usize>,
    filtre_type: String,
    types: Vec<String>,
    modele: Option<ModelRc<ContentItem>>,
    /// Empreinte des dossiers au dernier chargement (voir surveiller)
    empreinte: u64,
    /// Chargement de la liste en cours
    chargement: bool,
    /// Mises à jour disponibles (chemin relatif → version), cherchées à l'ouverture du profil
    majs: HashMap<String, MajDispo>,
    /// Fichiers en cours de mise à jour (chemins relatifs)
    majs_en_cours: HashSet<String>,
}

fn s(app: &App) -> InstanceState<'_> {
    app.ui.global::<InstanceState>()
}

fn instance(app: &App) -> Option<Instance> {
    app.instance.borrow().instance.clone()
}

pub fn installer(app: &Rc<App>) {
    let st = s(app);
    let a = app.clone();
    st.on_back(move || super::profiles::rafraichir(&a));
    let a = app.clone();
    st.on_play(move || {
        if let Some(i) = instance(&a) {
            super::profiles::jouer(&a, i);
        }
    });
    let a = app.clone();
    st.on_update_pack(move || {
        let a = a.clone();
        lancer(async move { mettre_a_jour_pack(&a).await });
    });
    let a = app.clone();
    st.on_folder(move || {
        if let Some(i) = instance(&a) {
            system_info::ouvrir_dossier(&i.dossier());
        }
    });
    let a = app.clone();
    st.on_misty_toggled(move |on| {
        let mut e = a.instance.borrow_mut();
        if let Some(inst) = e.instance.as_mut() {
            inst.misty_client = Some(on);
            let _ = instances::sauver(inst);
        }
    });
    let a = app.clone();
    st.on_export(move || {
        let a = a.clone();
        lancer(async move { exporter(&a).await });
    });
    let a = app.clone();
    st.on_change_icon(move || {
        let a = a.clone();
        lancer(async move { changer_icone(&a).await });
    });
    let a = app.clone();
    st.on_rename(move || {
        if let Some(i) = instance(&a) {
            s(&a).set_rename_text(i.nom.into());
            s(&a).set_renaming(true);
        }
    });
    let a = app.clone();
    st.on_rename_ok(move || {
        let nom = s(&a).get_rename_text().trim().to_string();
        if !nom.is_empty() {
            let mut i = a.instance.borrow_mut();
            if let Some(inst) = i.instance.as_mut() {
                inst.nom = nom;
                let _ = instances::sauver(inst);
            }
        }
        s(&a).set_renaming(false);
        maj_en_tete(&a);
    });
    let a = app.clone();
    st.on_delete(move || {
        let a = a.clone();
        lancer(async move {
            let Some(i) = instance(&a) else { return };
            if super::profiles::supprimer(&a, &i).await {
                super::profiles::rafraichir(&a);
            }
        });
    });
    let a = app.clone();
    st.on_tab_changed(move |t| {
        if t == 1 {
            if let Some(i) = instance(&a) {
                browser::configurer_instance(&a, i);
            }
        } else {
            let a = a.clone();
            lancer(async move { charger_contenu(&a).await });
        }
    });
    let a = app.clone();
    st.on_filter_changed(move |i| {
        let t = a.instance.borrow().types.get(i as usize).cloned().unwrap_or_default();
        a.instance.borrow_mut().filtre_type = t;
        appliquer_filtre(&a);
    });
    let a = app.clone();
    st.on_filter_text_changed(move |_| appliquer_filtre(&a));
    let a = app.clone();
    st.on_toggle(move |i| {
        let a = a.clone();
        lancer(async move { basculer(&a, i as usize).await });
    });
    let a = app.clone();
    st.on_remove(move |i| {
        let a = a.clone();
        lancer(async move { supprimer_contenu(&a, i as usize).await });
    });
    let a = app.clone();
    st.on_update_item(move |i| {
        let a = a.clone();
        lancer(async move {
            if let Some((_, contenu)) = contenu_affiche(&a, i as usize) {
                mettre_a_jour(&a, vec![contenu]).await;
            }
        });
    });
    let a = app.clone();
    st.on_update_all(move || {
        let a = a.clone();
        lancer(async move {
            let liste: Vec<ContenuInstalle> = {
                let e = a.instance.borrow();
                let Some(inst) = e.instance.as_ref() else { return };
                e.contenus.iter().filter(|c| e.majs.contains_key(&content::relatif(inst, &c.chemin_fichier))).cloned().collect()
            };
            mettre_a_jour(&a, liste).await;
        });
    });

    // Fichiers ajoutés à la main (glissés dans le dossier mods, etc.) : la liste se recharge seule
    let a = Rc::downgrade(app);
    let minuterie = slint::Timer::default();
    minuterie.start(slint::TimerMode::Repeated, std::time::Duration::from_secs(2), move || {
        if let Some(a) = a.upgrade() {
            surveiller(&a);
        }
    });
    std::mem::forget(minuterie);
}

/// Page du profil ouverte sur son contenu : recharge la liste si les dossiers ont changé.
fn surveiller(app: &Rc<App>) {
    if app.ui.global::<crate::AppState>().get_page() != super::page::INSTANCE || s(app).get_tab() != 0 {
        return;
    }
    let Some(inst) = instance(app) else { return };
    if app.instance.borrow().chargement {
        return;
    }
    app.instance.borrow_mut().chargement = true;
    let a = app.clone();
    lancer(async move {
        let i = inst.clone();
        let empreinte = a.bloquant(move || content::empreinte(&i)).await;
        let change = instance(&a).map(|x| x.id) == Some(inst.id.clone()) && empreinte != a.instance.borrow().empreinte;
        a.instance.borrow_mut().chargement = false;
        if change {
            charger_contenu(&a).await;
        }
    });
}

/// Met à jour le modpack du profil : choix de la version (l'actuelle est marquée), confirmation,
/// puis remplacement des fichiers du pack. Mondes, options et ajouts du joueur restent.
async fn mettre_a_jour_pack(app: &Rc<App>) {
    use crate::services::modpack;
    use crate::ProfilesState;
    let Some(inst) = instance(app) else { return };
    let attente = app.ui.global::<ProfilesState>();
    attente.set_wait_title(inst.nom.to_uppercase().into());
    attente.set_wait_text(loc::t("Pack_Recherche").into());
    attente.set_waiting(true);
    let i = inst.clone();
    let resultat = app
        .bg(async move {
            let versions = modpack::versions_du_profil(&i).await?;
            let actuelle = modpack::version_installee(&i).await;
            anyhow::Ok((versions, actuelle))
        })
        .await;
    attente.set_waiting(false);
    let (versions, actuelle) = match resultat {
        Ok((v, a)) if !v.is_empty() => (v, a),
        Ok(_) => return dialogs::erreur(app, &loc::t("Pack_MajImpossible"), &loc::t("Erreur_ModpackSansVersion")).await,
        Err(e) => return dialogs::erreur(app, &loc::t("Pack_MajImpossible"), &format!("{e:#}")).await,
    };

    // déjà la plus récente : on le dit, et on propose quand même une autre version
    let actuelle_nom = inst.modpack_version.clone().unwrap_or_default();
    if actuelle.as_deref() == Some(versions[0].id.as_str()) {
        let autre = dialogs::confirmer(
            app,
            &loc::t("Pack_AJour"),
            &loc::tf("Pack_AJour_Texte", &[&actuelle_nom]),
            Some(&loc::t("Pack_AutreVersion")),
            Some(&loc::t("Commun_Fermer")),
            false,
        )
        .await;
        if !autre {
            return;
        }
    }
    let libelles: Vec<String> = versions
        .iter()
        .map(|v| if actuelle.as_deref() == Some(v.id.as_str()) { format!("{}  ·  {}", v.libelle, loc::t("Pack_Actuelle")) } else { v.libelle.clone() })
        .collect();
    let Some(choix) = dialogs::choisir(app, &loc::t("Pack_MettreAJour"), &loc::tf("Pack_Choisir_Texte", &[&inst.nom]), &libelles).await else {
        return;
    };
    let Some(version) = versions.get(choix).cloned() else { return };
    if actuelle.as_deref() == Some(version.id.as_str()) {
        return;
    }
    let ok = dialogs::confirmer(
        app,
        &loc::t("Pack_Confirmer_Titre"),
        &loc::tf("Pack_Confirmer_Texte", &[&inst.nom, &version.libelle]),
        Some(&loc::t("Pack_MettreAJour")),
        None,
        false,
    )
    .await;
    if !ok {
        return;
    }

    attente.set_wait_title(inst.nom.to_uppercase().into());
    attente.set_wait_text(loc::t("Commun_Preparation").into());
    attente.set_waiting(true);
    let progression = app.progression(|ui, texte| ui.global::<ProfilesState>().set_wait_text(texte.into()));
    let (i, id) = (inst.clone(), version.id.clone());
    crate::services::journal::info(&format!("Mise à jour du modpack {} vers {}", inst.nom, version.libelle));
    let resultat = app.bg(async move { modpack::mettre_a_jour(&i, id, progression).await }).await;
    attente.set_waiting(false);
    match resultat {
        Ok(r) => {
            ouvrir(app, r.instance.clone());
            if !r.bloques.is_empty() {
                super::profiles::proposer_telechargements_manuels(app, &r).await;
            } else {
                dialogs::info(app, &loc::t("Pack_MajFaite"), &loc::tf("Pack_MajFaite_Texte", &[&inst.nom, &version.libelle])).await;
            }
        }
        Err(e) => {
            // le profil a pu changer à moitié : on le relit tel qu'il est sur le disque
            if let Some(i) = instances::trouver(&inst.id) {
                ouvrir(app, i);
            }
            dialogs::erreur(app, &loc::t("Pack_MajImpossible"), &format!("{e:#}")).await;
        }
    }
}

/// Affiche un profil (appelé à chaque ouverture de la page).
pub fn ouvrir(app: &Rc<App>, inst: Instance) {
    {
        let mut e = app.instance.borrow_mut();
        e.instance = Some(inst);
        e.majs.clear();
        e.majs_en_cours.clear();
    }
    let st = s(app);
    st.set_updating_all(false);
    st.set_renaming(false);
    st.set_status("".into());
    st.set_tab(0);
    st.set_filter_text("".into());
    maj_en_tete(app);
    construire_filtres(app);
    browser::oublier_instance(app); // l'explorateur sera reconfiguré à la première ouverture de l'onglet
    super::naviguer(app, super::page::INSTANCE);
    let a = app.clone();
    lancer(async move {
        charger_contenu(&a).await;
        verifier_majs(&a).await;
    });
}

/// Cherche les mises à jour du contenu et le côté client / serveur des mods (en arrière-plan).
async fn verifier_majs(app: &Rc<App>) {
    let Some(inst) = instance(app) else { return };
    let contenus = app.instance.borrow().contenus.clone();
    if contenus.is_empty() {
        return;
    }
    let i = inst.clone();
    let resultat = app.bg(async move { content::verifier(&i, contenus).await }).await;
    if instance(app).map(|x| x.id) != Some(inst.id) {
        return; // un autre profil a été ouvert entre-temps
    }
    match resultat {
        Ok((majs, contenus)) => {
            {
                let mut e = app.instance.borrow_mut();
                e.majs = majs;
                // métadonnées complétées (client / serveur), si la liste n'a pas changé entre-temps
                if e.contenus.len() == contenus.len()
                    && e.contenus.iter().zip(&contenus).all(|(a, b)| a.chemin_fichier == b.chemin_fichier)
                {
                    e.contenus = contenus;
                }
            }
            appliquer_filtre(app);
        }
        Err(e) => crate::services::journal::info(&format!("Recherche des mises à jour impossible : {e:#}")),
    }
}

/// Installe les mises à jour connues de ces fichiers, l'un après l'autre.
async fn mettre_a_jour(app: &Rc<App>, liste: Vec<ContenuInstalle>) {
    let Some(inst) = instance(app) else { return };
    let taches: Vec<(ContenuInstalle, String, MajDispo)> = {
        let mut e = app.instance.borrow_mut();
        let taches: Vec<_> = liste
            .into_iter()
            .filter_map(|c| {
                let rel = content::relatif(&inst, &c.chemin_fichier);
                let maj = e.majs.get(&rel)?.clone();
                (!e.majs_en_cours.contains(&rel)).then_some((c, rel, maj))
            })
            .collect();
        e.majs_en_cours.extend(taches.iter().map(|(_, r, _)| r.clone()));
        taches
    };
    if taches.is_empty() {
        return;
    }
    let tout = taches.len() > 1;
    if tout {
        s(app).set_updating_all(true);
    }
    appliquer_filtre(app);

    let mut erreurs = Vec::new();
    let mut faits = Vec::new();
    for (contenu, rel, maj) in taches {
        let progression = app.progression(|ui, texte| ui.global::<InstanceState>().set_status(texte.into()));
        let (i, c, m) = (inst.clone(), contenu.clone(), maj.clone());
        let resultat = app.bg(async move { content::mettre_a_jour(&i, &c, &m, Some(progression)).await }).await;
        if instance(app).map(|x| x.id) != Some(inst.id.clone()) {
            return; // profil fermé : la liste sera relue à la prochaine ouverture
        }
        {
            let mut e = app.instance.borrow_mut();
            e.majs_en_cours.remove(&rel);
            if resultat.is_ok() {
                e.majs.remove(&rel);
            }
        }
        match resultat {
            Ok(_) => faits.push(contenu.titre()),
            Err(e) => erreurs.push(format!("{} : {e:#}", contenu.titre())),
        }
    }
    s(app).set_updating_all(false);

    let i = inst.clone();
    let contenus = app.bloquant(move || content::lister(&i)).await;
    app.instance.borrow_mut().contenus = contenus;
    appliquer_filtre(app);
    s(app).set_status(
        match faits.len() {
            0 => String::new(),
            1 => loc::tf("Instance_MisAJour", &[&faits[0]]),
            n => loc::tf("Instance_MisAJourN", &[&n]),
        }
        .into(),
    );
    if !erreurs.is_empty() {
        dialogs::erreur(app, &loc::t("Instance_MajImpossible"), &erreurs.join("\n\n")).await;
    }
}

pub fn langue_changee(app: &Rc<App>) {
    if instance(app).is_none() {
        return;
    }
    construire_filtres(app);
    appliquer_filtre(app);
}

fn maj_en_tete(app: &Rc<App>) {
    let Some(i) = instance(app) else { return };
    let st = s(app);
    st.set_name(i.nom.to_uppercase().into());

    let mut badges = vec![
        BadgeItem { text: i.version_mc.to_uppercase().into(), color: Color::from_rgb_u8(0x1C, 0x23, 0x35) },
        BadgeItem {
            text: format!("{}{}", i.loader, i.loader_version.as_ref().map(|v| format!(" {v}")).unwrap_or_default()).to_uppercase().into(),
            color: Color::from_rgb_u8(0x13, 0xB2, 0x4C),
        },
    ];
    if i.est_modpack() {
        badges.push(BadgeItem {
            text: format!("MODPACK {}", i.modpack_version.clone().unwrap_or_default()).to_uppercase().into(),
            color: Color::from_rgb_u8(0x2F, 0x8C, 0xFF),
        });
    }
    st.set_badges(modele(badges));
    st.set_is_modpack(i.est_modpack());
    st.set_misty_available(client::compatible(&i.version_mc, &i.loader));
    st.set_misty_on(client::voulu(&i));

    st.set_has_icon(false);
    let (fichier, url, id) = (i.chemin_icone(), i.icone_url.clone(), i.id.clone());
    let a = app.clone();
    lancer(async move {
        let image = match images::depuis_fichier(&fichier, 160) {
            Some(img) => Some(img),
            None => match url {
                Some(u) => images::depuis_url(&a, &u).await,
                None => None,
            },
        };
        if instance(&a).map(|i| i.id) == Some(id) {
            if let Some(img) = image {
                s(&a).set_icon(img);
                s(&a).set_has_icon(true);
            }
        }
    });
}

/// Filter labels with the number of items of each type, e.g. "Mods (12)".
fn libelles_filtres(app: &App) -> Vec<String> {
    let e = app.instance.borrow();
    let avec_nombre = |libelle: String, nb: usize| if nb > 0 { format!("{libelle} ({nb})") } else { libelle };
    let mut libelles = vec![avec_nombre(loc::t("Instance_Tout"), e.contenus.len())];
    for (t, l) in content::types() {
        libelles.push(avec_nombre(l, e.contenus.iter().filter(|c| c.type_ == t).count()));
    }
    libelles
}

/// Filtres par type (reconstruits au changement de langue).
fn construire_filtres(app: &Rc<App>) {
    let mut types = vec![String::new()];
    for (t, _) in content::types() {
        types.push(t);
    }
    let libelles = libelles_filtres(app);
    let actuel = app.instance.borrow().filtre_type.clone();
    let index = types.iter().position(|t| *t == actuel).unwrap_or(0);
    s(app).set_filters(modele_textes(&libelles));
    s(app).set_filter_index(index as i32);
    let mut e = app.instance.borrow_mut();
    e.types = types;
    if index == 0 {
        e.filtre_type.clear();
    }
}

// ───────────────────────── Contenu installé ─────────────────────────

async fn charger_contenu(app: &Rc<App>) {
    let Some(i) = instance(app) else { return };
    let inst = i.clone();
    app.instance.borrow_mut().chargement = true;
    let (contenus, empreinte) = app.bloquant(move || (content::lister(&inst), content::empreinte(&inst))).await;
    {
        let mut e = app.instance.borrow_mut();
        e.empreinte = empreinte;
        e.chargement = false;
    }
    let a_identifier = contenus.iter().any(|c| c.meta.is_none());
    app.instance.borrow_mut().contenus = contenus.clone();
    appliquer_filtre(app);

    // Identification des fichiers ajoutés à la main (nom + icône depuis Modrinth / CurseForge)
    if a_identifier {
        s(app).set_status(loc::t("Instance_Identification").into());
        let inst = i.clone();
        let resultat = app.bg(async move { content::identifier(&inst, contenus).await }).await;
        // hors-ligne : on garde les noms de fichiers
        if instance(app).map(|x| x.id) == Some(i.id.clone()) {
            if let Ok(c) = resultat {
                app.instance.borrow_mut().contenus = c;
                appliquer_filtre(app);
            }
            s(app).set_status("".into());
        }
    }
}

fn appliquer_filtre(app: &Rc<App>) {
    let texte = s(app).get_filter_text().trim().to_lowercase();
    let (visibles, items, nb, nb_majs) = {
        let e = app.instance.borrow();
        let rel = |c: &ContenuInstalle| e.instance.as_ref().map(|i| content::relatif(i, &c.chemin_fichier)).unwrap_or_default();
        let visibles: Vec<usize> = (0..e.contenus.len())
            .filter(|&i| {
                let c = &e.contenus[i];
                (e.filtre_type.is_empty() || c.type_ == e.filtre_type)
                    && (texte.is_empty() || c.titre().to_lowercase().contains(&texte) || c.nom_fichier().to_lowercase().contains(&texte))
            })
            .collect();
        let items: Vec<(ContentItem, Option<String>)> = visibles
            .iter()
            .map(|&i| {
                let c = &e.contenus[i];
                let r = rel(c);
                (
                    ContentItem {
                        title: c.titre().into(),
                        details: c.details().into(),
                        icon: images::vide(),
                        has_icon: false,
                        active: c.actif(),
                        side: c.environnement().code(),
                        update: e.majs.get(&r).map(|m| m.version_nom.clone()).unwrap_or_default().into(),
                        updating: e.majs_en_cours.contains(&r),
                    },
                    c.icone_url(),
                )
            })
            .collect();
        let nb_majs = e.contenus.iter().filter(|c| e.majs.contains_key(&rel(c))).count();
        (visibles, items, e.contenus.len(), nb_majs)
    };
    s(app).set_updates_count(nb_majs as i32);

    let urls: Vec<Option<String>> = items.iter().map(|(_, u)| u.clone()).collect();
    let m = modele(items.into_iter().map(|(c, _)| c).collect());
    s(app).set_items(m.clone());
    s(app).set_filters(modele_textes(&libelles_filtres(app)));
    s(app).set_content_tab_text(if nb > 0 { format!("{} ({nb})", loc::t("Instance_Contenu")) } else { loc::t("Instance_Contenu") }.into());
    {
        let mut e = app.instance.borrow_mut();
        e.visibles = visibles;
        e.modele = Some(m.clone());
    }

    for (index, url) in urls.into_iter().enumerate() {
        let Some(url) = url else { continue };
        let m = m.clone();
        let a = app.clone();
        lancer(async move {
            if let Some(img) = images::depuis_url(&a, &url).await {
                maj_ligne(&m, index, |c| {
                    c.icon = img;
                    c.has_icon = true;
                });
            }
        });
    }
}

fn contenu_affiche(app: &App, ligne: usize) -> Option<(usize, ContenuInstalle)> {
    let e = app.instance.borrow();
    let i = *e.visibles.get(ligne)?;
    Some((i, e.contenus.get(i)?.clone()))
}

async fn basculer(app: &Rc<App>, ligne: usize) {
    let Some((i, contenu)) = contenu_affiche(app, ligne) else { return };
    match content::basculer(&contenu) {
        Ok(nouveau) => {
            let actif = {
                let mut e = app.instance.borrow_mut();
                e.contenus[i].chemin_fichier = nouveau;
                e.contenus[i].actif()
            };
            let details = app.instance.borrow().contenus[i].details();
            if let Some(m) = app.instance.borrow().modele.clone() {
                maj_ligne(&m, ligne, |c| {
                    c.active = actif;
                    c.details = details.into();
                });
            }
        }
        Err(e) => {
            dialogs::erreur(app, &loc::t("Instance_ModifImpossible"), &format!("{e}\n\n{}", loc::t("Commun_JeuOuvert"))).await;
        }
    }
}

async fn supprimer_contenu(app: &Rc<App>, ligne: usize) {
    let Some((i, contenu)) = contenu_affiche(app, ligne) else { return };
    let Some(inst) = instance(app) else { return };
    let ok = dialogs::confirmer(
        app,
        &loc::t("Commun_Supprimer"),
        &loc::tf("Instance_SupprimerContenu", &[&contenu.titre()]),
        Some(&loc::t("Commun_Supprimer")),
        None,
        true,
    )
    .await;
    if !ok {
        return;
    }
    match content::supprimer(&inst, &contenu) {
        Ok(()) => {
            app.instance.borrow_mut().contenus.remove(i);
            appliquer_filtre(app);
        }
        Err(e) => {
            dialogs::erreur(app, &loc::t("Commun_SuppressionImpossible"), &format!("{e}\n\n{}", loc::t("Commun_JeuOuvert"))).await;
        }
    }
}

/// Le projet est-il installé dans le profil affiché ?
pub fn est_installe(app: &App, projet: &ProjetDistant) -> bool {
    content::est_installe(&app.instance.borrow().contenus, &projet.source, &projet.id)
}

// ───────────────────────── Installation depuis l'explorateur ─────────────────────────

/// Installation (avec les cas particuliers datapacks / shaders). Renvoie true si elle a réussi.
pub async fn installer_projet(app: &Rc<App>, projet: ProjetDistant, type_: String) -> bool {
    let Some(inst) = instance(app) else { return false };
    let progression = app.progression(|ui, texte| ui.global::<InstanceState>().set_status(texte.into()));

    let mut monde = None;
    if type_ == content::DATAPACK {
        let mondes = content::mondes(&inst);
        if mondes.is_empty() {
            dialogs::info(app, &loc::t("Instance_AucunMonde"), &loc::t("Instance_AucunMonde_Texte")).await;
            return false;
        }
        monde = if mondes.len() == 1 {
            Some(mondes[0].clone())
        } else {
            dialogs::choisir(app, &loc::t("Instance_ChoisirMonde"), &loc::tf("Instance_ChoisirMonde_Texte", &[&projet.titre]), &mondes)
                .await
                .map(|i| mondes[i].clone())
        };
        if monde.is_none() {
            return false;
        }
    }

    if type_ == content::SHADER {
        match preparer_shaders(app, &inst, progression.clone()).await {
            Ok(true) => {}
            Ok(false) => return false,
            Err(e) => {
                s(app).set_status("".into());
                dialogs::erreur(app, &loc::t("Instance_InstallationImpossible"), &format!("{e:#}")).await;
                return false;
            }
        }
    }

    let (i2, p2, t2) = (inst.clone(), projet.clone(), type_.clone());
    let resultat = app
        .bg(async move { content::installer(&i2, &p2.source, &p2.id, &t2, monde.as_deref(), Some(progression)).await })
        .await;
    match resultat {
        Ok(installes) => {
            s(app).set_status(
                if installes.len() > 1 {
                    loc::tf("Instance_InstalleDeps", &[&projet.titre, &(installes.len() - 1)])
                } else {
                    loc::tf("Instance_Installe", &[&projet.titre])
                }
                .into(),
            );
            let i3 = inst.clone();
            let contenus = app.bloquant(move || content::lister(&i3)).await;
            app.instance.borrow_mut().contenus = contenus;
            appliquer_filtre(app);
            true
        }
        Err(e) => {
            s(app).set_status("".into());
            dialogs::erreur(app, &loc::t("Instance_InstallationImpossible"), &format!("{e:#}")).await;
            false
        }
    }
}

/// Les shaders ont besoin d'Iris (Fabric/Quilt/NeoForge), Oculus (Forge) ou OptiFine.
async fn preparer_shaders(app: &Rc<App>, inst: &Instance, progression: crate::services::Progress) -> anyhow::Result<bool> {
    let Some(prerequis) = content::prerequis_shaders(inst, &content::lister(inst)) else { return Ok(true) };

    if prerequis.is_empty() {
        return Ok(dialogs::confirmer(
            app,
            &loc::t("Shaders_NonPris"),
            &loc::tf("Shaders_NonPris_Texte", &[&inst.loader]),
            Some(&loc::t("Shaders_QuandMeme")),
            None,
            false,
        )
        .await);
    }

    let nom = if prerequis == content::ID_IRIS { "Iris" } else { "Oculus" };
    let installer = dialogs::confirmer(
        app,
        &loc::t("Shaders_ModRequis"),
        &loc::tf("Shaders_ModRequis_Texte", &[&nom]),
        Some(&loc::tf("Shaders_Installer", &[&nom])),
        Some(&loc::t("Commun_Ignorer")),
        false,
    )
    .await;
    if !installer {
        return Ok(true);
    }
    let i = inst.clone();
    app.bg(async move { content::installer(&i, MODRINTH, prerequis, content::MOD, None, Some(progression)).await }).await?;
    Ok(true)
}

// ───────────────────────── Actions du profil ─────────────────────────

async fn changer_icone(app: &Rc<App>) {
    let Some(i) = instance(app) else { return };
    let Some(fichier) = dialogs::choisir_fichier(&loc::t("Instance_IconeTitre"), "Images", &["png", "jpg", "jpeg", "bmp", "gif"]).await else {
        return;
    };
    // Réencodé en PNG 128 px pour garder un fichier léger
    let destination = i.chemin_icone();
    let resultat = app.bloquant(move || packs::enregistrer_icone(&fichier, &destination)).await;
    match resultat {
        Ok(()) => {
            let _ = instances::sauver(&i);
            maj_en_tete(app);
        }
        Err(e) => dialogs::erreur(app, &loc::t("Instance_ImageIllisible"), &e.to_string()).await,
    }
}

/// Exporte le profil en modpack .mrpack, avec Misty Client si sa case est cochée.
async fn exporter(app: &Rc<App>) {
    let Some(i) = instance(app) else { return };
    let nom_fichier: String = i.nom.chars().map(|c| if c.is_alphanumeric() || c == ' ' || c == '-' || c == '_' { c } else { '_' }).collect();
    let nom_fichier = format!("{}.mrpack", nom_fichier.trim());
    let Some(destination) = dialogs::enregistrer_fichier(&loc::t("Export_Titre"), &nom_fichier, "Modpack Modrinth", "mrpack").await else {
        return;
    };
    let avec_misty = client::voulu(&i);
    s(app).set_status(loc::t("Export_EnCours").into());
    let (inst, chemin) = (i.clone(), destination.clone());
    let resultat = app.bloquant(move || export::exporter(&inst, &chemin, avec_misty)).await;
    s(app).set_status("".into());
    match resultat {
        Ok(()) => {
            let nom = destination.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default();
            if dialogs::confirmer(app, &loc::t("Export_Fini"), &loc::tf("Export_Fini_Texte", &[&nom]), Some(&loc::t("Export_OuvrirDossier")), Some(&loc::t("Commun_Fermer")), false).await {
                if let Some(dossier) = destination.parent() {
                    system_info::ouvrir_dossier(dossier);
                }
            }
        }
        Err(e) => dialogs::erreur(app, &loc::t("Export_Erreur"), &e.to_string()).await,
    }
}
