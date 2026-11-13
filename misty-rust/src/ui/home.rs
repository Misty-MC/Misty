//! Page d'accueil : choix du compte, du profil (ou jeu rapide : version + mod loader) et lancement du jeu.

use super::{dialogs, images, lancer, modele, modele_textes, App};
use crate::minecraft::{install, launch, loaders, FileProgress, McDirs};
use crate::services::horizon;
use crate::services::instances::{self, Instance};
use crate::services::{account, content, discord, loc, pseudos, settings, journal, telemetry, client};
use crate::{AppState, HomeState};
use slint::ComponentHandle;
use std::rc::Rc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

/// Erreur « téléchargement annulé par le joueur » (pas affichée comme une erreur).
#[derive(Debug)]
struct Annule;

impl std::fmt::Display for Annule {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&loc::t("Home_Annule"))
    }
}

impl std::error::Error for Annule {}

fn est_annule(e: &anyhow::Error) -> bool {
    e.chain().any(|c| c.downcast_ref::<Annule>().is_some())
}

/// Téléchargement qu'on peut annuler : chaque tâche de fond lancée par `tache` s'arrête à la
/// demande (les fichiers à moitié téléchargés restent en .part et seront refaits).
#[derive(Clone, Default)]
struct Annulation(Rc<std::cell::RefCell<(Vec<tokio::task::AbortHandle>, bool)>>);

impl Annulation {
    fn annuler(&self) {
        let mut a = self.0.borrow_mut();
        a.1 = true;
        for t in a.0.drain(..) {
            t.abort();
        }
    }

    fn annulee(&self) -> bool {
        self.0.borrow().1
    }

    /// Erreur Annule si le joueur a annulé depuis l'étape précédente.
    fn verifier(&self) -> anyhow::Result<()> {
        if self.annulee() {
            Err(Annule.into())
        } else {
            Ok(())
        }
    }

    /// Lance une tâche de fond annulable et attend son résultat.
    async fn tache<T: Send + 'static>(
        &self,
        app: &App,
        f: impl std::future::Future<Output = anyhow::Result<T>> + Send + 'static,
    ) -> anyhow::Result<T> {
        self.verifier()?;
        let h = app.rt.spawn(f);
        self.0.borrow_mut().0.push(h.abort_handle());
        match h.await {
            Ok(r) => r,
            Err(e) if e.is_cancelled() => Err(Annule.into()),
            Err(e) => Err(anyhow::anyhow!("{e}")),
        }
    }
}

#[derive(Default)]
pub struct Etat {
    /// Téléchargement en cours (annulable par le bouton Annuler)
    annulation: Option<Annulation>,
    version_choisie: String,
    modes_charges: bool,
    installation_en_cours: bool,
    /// Parties lancées depuis Misty qui tournent encore (plusieurs à la fois possible).
    pub jeux_en_cours: u32,
    profil: Option<Instance>,
    /// Profils de la liste déroulante (None = jeu rapide).
    profils: Vec<Option<Instance>>,
    versions: Vec<String>,
    modes: Vec<String>,
    pseudos: Vec<String>,
    /// Toutes les versions de Mojang (type, année), pour la fenêtre de choix
    toutes: Vec<super::picker::Entree>,
    /// Builds d'Horizon publiés sur GitHub
    horizon_builds: Vec<horizon::Build>,
    /// Mises à jour affichées dans la fenêtre de choix, et celle qui est ouverte
    cles_groupes: Vec<String>,
    groupe_choisi: String,
    /// Mod loader à présélectionner quand ses modes seront chargés (filtre de la fenêtre de choix)
    loader_voulu: Option<String>,
}

fn h(app: &App) -> HomeState<'_> {
    app.ui.global::<HomeState>()
}

/// Début d'un téléchargement annulable.
fn debut_annulable(app: &App) -> Annulation {
    let an = Annulation::default();
    app.home.borrow_mut().annulation = Some(an.clone());
    h(app).set_can_cancel(true);
    an
}

/// Fin du téléchargement (réussi, raté ou annulé) : plus rien à annuler.
fn fin_annulable(app: &App) {
    app.home.borrow_mut().annulation = None;
    h(app).set_can_cancel(false);
}

pub fn installer(app: &Rc<App>) {
    let s = h(app);
    s.set_play_text(loc::t("Home_Bouton_Jouer").into());
    let a = app.clone();
    s.on_cancel_install(move || {
        let an = a.home.borrow().annulation.clone();
        if let Some(an) = an {
            an.annuler();
            fin_annulable(&a);
            statut(&a, &loc::t("Home_Annulation"));
        }
    });
    s.set_status(loc::t("Home_Chargement").into());

    let a = app.clone();
    s.on_account_selected(move |i| {
        let pseudo = a.home.borrow().pseudos.get(i as usize).cloned();
        if let Some(p) = pseudo {
            settings::set(settings::LAST_PSEUDO, &p);
        }
        maj_avatar(&a);
    });
    // Premier lancement : Premium (compte Microsoft) ou Hors ligne (un pseudo)
    let a = app.clone();
    a.ui.global::<AppState>().on_choose_premium({
        let a = a.clone();
        move || {
            settings::set(settings::MODE_COMPTE, "premium");
            a.ui.global::<AppState>().set_account_choice(false);
            if account::session().is_none() {
                let a = a.clone();
                lancer(async move { se_connecter(&a).await });
            }
        }
    });
    let a = app.clone();
    a.ui.global::<AppState>().on_choose_offline({
        let a = a.clone();
        move |pseudo| {
            let pseudo = nettoyer_pseudo(&pseudo);
            if pseudo.is_empty() {
                return;
            }
            pseudos::ajouter(&pseudo);
            settings::set(settings::LAST_PSEUDO, &pseudo);
            settings::set(settings::MODE_COMPTE, "horsligne");
            charger_pseudos(&a);
            a.ui.global::<AppState>().set_account_choice(false);
        }
    });
    let a = app.clone();
    s.on_ms_clicked(move || {
        let a = a.clone();
        lancer(async move {
            if account::comptes().is_empty() {
                se_connecter(&a).await;
            } else {
                ajouter_compte_microsoft(&a).await;
            }
        });
    });
    // liste unique : un compte Microsoft (on s'y connecte) ou un pseudo hors ligne
    let a = app.clone();
    s.on_all_selected(move |i| {
        let ms = account::comptes();
        let i = i.max(0) as usize;
        if let Some(c) = ms.get(i) {
            if account::session().is_some_and(|s| s.uuid == c.uuid) {
                return;
            }
            let (a, uuid) = (a.clone(), c.uuid.clone());
            lancer(async move { changer_compte_microsoft(&a, uuid).await });
            return;
        }
        let pseudo = a.home.borrow().pseudos.get(i - ms.len()).cloned();
        if let Some(p) = pseudo {
            settings::set(settings::LAST_PSEUDO, &p);
            h(&a).set_account_index((i - ms.len()) as i32);
            // hors ligne : la session Microsoft se ferme (le compte reste enregistré)
            if account::session().is_some() {
                account::deconnecter();
            }
            session_changee(&a);
        }
    });
    // Liste des comptes Microsoft de l'accueil : on change de compte
    let a = app.clone();
    s.on_ms_selected(move |i| {
        let uuid = account::comptes().get(i as usize).map(|c| c.uuid.clone());
        if let Some(uuid) = uuid {
            let a = a.clone();
            lancer(async move { changer_compte_microsoft(&a, uuid).await });
        }
    });
    let a = app.clone();
    s.on_profile_selected(move |i| {
        h(&a).set_horizon_selected(false);
        misty_quitter(&a);
        let profil = a.home.borrow().profils.get(i as usize).cloned().flatten();
        settings::set(settings::DERNIER_PROFIL, profil.as_ref().map(|p| p.id.as_str()).unwrap_or(""));
        a.home.borrow_mut().profil = profil;
        maj_mode_profil(&a);
    });
    // Section Misty Client : Fabric + Misty Client, sur une des versions de Minecraft du mod
    s.set_misty_available(client::disponible());
    let versions: Vec<String> = client::versions().iter().rev().map(|v| v.to_string()).collect();
    s.set_misty_versions(modele_textes(&versions));
    let misty = version_misty();
    s.set_misty_mc(misty.as_str().into());
    s.set_misty_art(super::picker::illustration(&super::picker::cle_groupe(&misty, 0)));
    let a = app.clone();
    s.on_misty_select(move |v| misty_choisir(&a, v.to_string()));
    let a = app.clone();
    s.on_misty_play(move || {
        let v = h(&a).get_misty_mc().to_string();
        misty_choisir(&a, v);
        h(&a).set_picker_open(false);
        let a = a.clone();
        lancer(async move { jouer(&a).await });
    });
    // Horizon : builds GitHub, lancés par Misty
    s.set_horizon_art(super::picker::illustration("horizon"));
    s.set_horizon_build(settings::get(settings::HORIZON_BUILD).into());
    let a = app.clone();
    s.on_horizon_refresh(move || {
        let a = a.clone();
        lancer(async move { horizon_rafraichir(&a).await });
    });
    let a = app.clone();
    s.on_horizon_filter_changed(move || horizon_afficher(&a));
    let a = app.clone();
    s.on_horizon_install(move |tag| {
        let a = a.clone();
        lancer(async move { horizon_installer(&a, tag.to_string()).await });
    });
    let a = app.clone();
    s.on_horizon_remove(move |tag| {
        let a = a.clone();
        lancer(async move { horizon_supprimer(&a, tag.to_string()).await });
    });
    let a = app.clone();
    s.on_horizon_select(move |tag| {
        settings::set(settings::HORIZON_BUILD, &tag);
        h(&a).set_horizon_build(tag);
        horizon_choisir(&a);
    });
    let a = app.clone();
    s.on_horizon_play(move || {
        horizon_choisir(&a);
        h(&a).set_picker_open(false);
        let a = a.clone();
        lancer(async move { horizon_jouer(&a).await });
    });
    s.on_horizon_link(|i| crate::services::system_info::ouvrir(if i == 0 { horizon::SITE } else { horizon::GITHUB }));

    // Fenêtre de choix de version
    let a = app.clone();
    s.on_picker_changed(move || filtrer_versions(&a));
    let a = app.clone();
    s.on_picker_group_selected(move |i| {
        let cle = a.home.borrow().cles_groupes.get(i as usize).cloned();
        if let Some(cle) = cle {
            a.home.borrow_mut().groupe_choisi = cle;
            filtrer_versions(&a);
        }
    });
    let a = app.clone();
    s.on_version_picked(move |id| {
        let a = a.clone();
        lancer(async move { version_choisie_par_id(&a, id.to_string()).await });
    });
    let a = app.clone();
    s.on_version_selected(move |i| {
        let a = a.clone();
        lancer(async move { version_choisie(&a, i).await });
    });
    let a = app.clone();
    s.on_mode_selected(move |i| {
        if a.home.borrow().profil.is_none() {
            let mode = a.home.borrow().modes.get(i as usize).cloned().unwrap_or_else(|| loaders::VANILLA.into());
            // Misty, c'est Fabric : un autre loader revient à une version normale
            if mode != loaders::FABRIC {
                misty_quitter(&a);
            }
            h(&a).set_card_loader(mode.to_uppercase().into());
        }
    });
    let a = app.clone();
    s.on_reload(move || {
        let a = a.clone();
        lancer(async move { recharger_versions(&a).await });
    });
    let a = app.clone();
    s.on_manage(move || {
        let profil = a.home.borrow().profil.clone();
        if let Some(p) = profil {
            a.ui.global::<AppState>().set_nav(super::page::PROFILS);
            super::profiles::rafraichir(&a);
            super::instance::ouvrir(&a, p);
        }
    });
    let a = app.clone();
    s.on_play(move || {
        if h(&a).get_horizon_selected() {
            let a = a.clone();
            lancer(async move { horizon_jouer(&a).await });
            return;
        }
        let a = a.clone();
        lancer(async move { jouer(&a).await });
    });
}

fn statut(app: &App, texte: &str) {
    h(app).set_status(texte.into());
}

// ───────────────────────── Initialisation ─────────────────────────

/// Appelé une fois au démarrage, après la préparation des fichiers de config.
/// Pseudo Minecraft valide : lettres, chiffres et _ seulement, 16 caractères au plus.
fn nettoyer_pseudo(texte: &str) -> String {
    texte.chars().filter(|c| c.is_ascii_alphanumeric() || *c == '_').take(16).collect()
}

/// Au démarrage, tant qu'aucune façon de jouer n'a été choisie : l'écran Premium | Hors ligne.
/// Un compte Microsoft déjà connecté vaut le choix « premium » (pas de question).
fn proposer_mode_compte(app: &Rc<App>) {
    if !settings::get(settings::MODE_COMPTE).is_empty() {
        return;
    }
    if account::session().is_some() || settings::get_bool(settings::PREMIUM, false) {
        settings::set(settings::MODE_COMPTE, "premium");
        return;
    }
    // captures d'écran de développement : seulement si on le demande (il cacherait les pages)
    let capture = cfg!(debug_assertions) && std::env::var_os("MISTY_CAPTURE").is_some();
    if capture && std::env::var_os("MISTY_CAPTURE_CHOIX").is_none() {
        return;
    }
    app.ui.global::<AppState>().set_account_choice(true);
}

pub async fn initialiser(app: &Rc<App>) {
    charger_pseudos(app);
    maj_compte(app);
    charger_profils(app);
    proposer_mode_compte(app);

    // Connexion automatique en parallèle du chargement des versions
    // (pas pendant les captures d'écran de développement : la fenêtre de connexion s'ouvrirait)
    let capture = cfg!(debug_assertions) && std::env::var_os("MISTY_CAPTURE").is_some();
    if !capture && settings::get_bool(settings::PREMIUM, false) && account::session().is_none() {
        let a = app.clone();
        lancer(async move { se_connecter(&a).await });
    }

    statut(app, &loc::t("Home_ChargementVersions"));
    charger_versions(app).await;
    // tout premier démarrage (rien de choisi avant) : Misty Client, sur sa version la plus récente
    let premier = settings::get(settings::LAST_VERSION).is_empty() && settings::get(settings::MISTY_CHOISI).is_empty();
    if (premier || settings::get_bool(settings::MISTY_CHOISI, false)) && client::disponible() && app.home.borrow().profil.is_none() {
        misty_choisir(app, version_misty());
    } else {
        charger_derniere_version(app);
    }
    if app.home.borrow().profil.is_some() {
        // un profil n'attend pas la liste des versions : « Chargement des versions… » ne doit pas
        // rester affiché (seul le jeu rapide remet le statut à jour une fois ses loaders trouvés)
        maj_mode_profil(app);
    } else if h(app).get_version_index() < 0 {
        statut(app, &loc::t("Home_ChoisisVersion"));
    }
}

/// Appelé quand on revient sur la page (les options ont pu changer).
pub fn rafraichir(app: &Rc<App>, recharger_versions_aussi: bool) {
    charger_pseudos(app);
    charger_profils(app);
    if recharger_versions_aussi {
        let a = app.clone();
        lancer(async move { recharger_versions(&a).await });
    }
}

pub fn langue_changee(app: &Rc<App>) {
    maj_compte(app);
    charger_profils(app); // "Jeu rapide" + résumé du profil
    if !app.home.borrow().installation_en_cours {
        h(app).set_play_text(loc::t("Home_Bouton_Jouer").into());
    }
}

// ───────────────────────── Comptes ─────────────────────────

/// Liste des comptes hors ligne de l'accueil, à recharger quand les paramètres en ajoutent ou en suppriment.
pub fn comptes_hors_ligne_changes(app: &Rc<App>) {
    charger_pseudos(app);
}

fn charger_pseudos(app: &Rc<App>) {
    let liste = pseudos::lire();
    let dernier = settings::get(settings::LAST_PSEUDO);
    let index = liste.iter().position(|p| *p == dernier).map(|i| i as i32).unwrap_or(if liste.is_empty() { -1 } else { 0 });
    h(app).set_accounts(modele_textes(&liste));
    h(app).set_account_index(index);
    app.home.borrow_mut().pseudos = liste;
    maj_liste_comptes(app);
    maj_avatar(app);
}

/// La liste unique de l'accueil : comptes Microsoft, puis pseudos hors ligne (un pseudo qui existe
/// aussi en compte Microsoft est marqué « hors ligne » pour les distinguer).
fn maj_liste_comptes(app: &Rc<App>) {
    let ms = account::comptes();
    let hors_ligne = app.home.borrow().pseudos.clone();
    let mut noms: Vec<String> = ms.iter().map(|c| c.pseudo.clone()).collect();
    for p in &hors_ligne {
        if ms.iter().any(|c| c.pseudo.eq_ignore_ascii_case(p)) {
            noms.push(format!("{p} ({})", loc::t("Home_Compte_HorsLigne").to_lowercase()));
        } else {
            noms.push(p.clone());
        }
    }
    let index = match account::session() {
        Some(s) => ms.iter().position(|c| c.uuid == s.uuid).map(|i| i as i32).unwrap_or(-1),
        None if hors_ligne.is_empty() => -1,
        None => ms.len() as i32 + h(app).get_account_index().max(0),
    };
    h(app).set_all_accounts(modele_textes(&noms));
    h(app).set_all_index(index);
}

/// Ajoute un compte Microsoft de plus (fenêtre de connexion), qui devient le compte utilisé.
async fn ajouter_compte_microsoft(app: &Rc<App>) {
    h(app).set_ms_busy(true);
    super::settings::occupe_microsoft(app, true);
    if let Err(e) = app.bg(account::ajouter()).await {
        dialogs::erreur(app, &loc::t("Compte_ConnexionImpossible"), &e.to_string()).await;
    }
    h(app).set_ms_busy(false);
    super::settings::occupe_microsoft(app, false);
    session_changee(app);
}

async fn se_connecter(app: &Rc<App>) {
    h(app).set_ms_busy(true);
    super::settings::occupe_microsoft(app, true);
    let ancien = h(app).get_status().to_string();
    statut(app, &loc::t("Home_ConnexionMicrosoft"));
    if let Err(e) = app.bg(account::connecter()).await {
        dialogs::erreur(app, &loc::t("Compte_ConnexionImpossible"), &e.to_string()).await;
    }
    h(app).set_ms_busy(false);
    super::settings::occupe_microsoft(app, false);
    statut(app, &ancien);
    session_changee(app);
}

/// Passe sur un autre compte Microsoft déjà ajouté.
pub async fn changer_compte_microsoft(app: &Rc<App>, uuid: String) {
    h(app).set_ms_busy(true);
    super::settings::occupe_microsoft(app, true);
    if let Err(e) = app.bg(async move { account::activer(&uuid).await }).await {
        dialogs::erreur(app, &loc::t("Compte_ConnexionImpossible"), &e.to_string()).await;
    }
    h(app).set_ms_busy(false);
    super::settings::occupe_microsoft(app, false);
    session_changee(app);
}

/// Connexion / déconnexion : met à jour l'accueil et les options.
pub fn session_changee(app: &Rc<App>) {
    maj_compte(app);
    super::settings::maj_microsoft(app);
    super::skin::session_changee(app);
}

fn maj_compte(app: &Rc<App>) {
    let session = account::session();
    h(app).set_premium(session.is_some());
    let courant = session.as_ref().map(|s| s.uuid.clone()).unwrap_or_default();
    h(app).set_premium_name(session.map(|s| s.pseudo).unwrap_or_default().into());
    let comptes = account::comptes();
    h(app).set_ms_index(comptes.iter().position(|c| c.uuid == courant).map(|i| i as i32).unwrap_or(-1));
    let noms: Vec<String> = comptes.into_iter().map(|c| c.pseudo).collect();
    h(app).set_ms_names(modele_textes(&noms));
    maj_liste_comptes(app);
    maj_avatar(app);
}

pub fn pseudo_actuel(app: &App) -> String {
    if let Some(s) = account::session() {
        return s.pseudo;
    }
    let i = h(app).get_account_index();
    app.home
        .borrow()
        .pseudos
        .get(i.max(0) as usize)
        .filter(|p| !p.is_empty())
        .cloned()
        .unwrap_or_else(|| "Misty".into())
}

/// Tête du skin (mc-heads.net). Si le téléchargement échoue, le bloc d'herbe reste visible.
fn maj_avatar(app: &Rc<App>) {
    let pseudo = pseudo_actuel(app);
    let a = app.clone();
    let premium = account::session().is_some();
    // petite image du statut Discord : la tête de ce compte
    let uuid = account::session().map(|s| s.uuid);
    discord::definir_joueur(&pseudo, uuid.as_deref());
    lancer(async move {
        // Compte Microsoft : tête tirée du dernier skin connu (disque), sans interroger Mojang.
        // L'onglet Skin la met à jour dès qu'il a reçu le vrai skin ; mc-heads renverrait Steve.
        if premium {
            let uuid = account::session().map(|s| s.uuid).unwrap_or_default();
            let tete = a
                .bg(async move {
                    let p = crate::services::skins::profil_en_cache(&uuid)?;
                    if p.skin_url.is_empty() {
                        return None;
                    }
                    let png = crate::services::skins::telecharger(&p.skin_url).await.ok()?;
                    crate::services::skins::rendre_tete(&png)
                })
                .await;
            if pseudo_actuel(&a) != pseudo {
                return;
            }
            if let Some(tete) = tete {
                h(&a).set_has_avatar(true);
                h(&a).set_avatar(slint::Image::from_rgba8(images::tampon(&tete)));
            }
            return;
        }
        let url = format!("https://mc-heads.net/avatar/{}/64", urlencoding::encode(&pseudo));
        let image = images::depuis_url(&a, &url).await;
        if pseudo_actuel(&a) != pseudo {
            return; // le compte a changé entre-temps
        }
        h(&a).set_has_avatar(image.is_some());
        h(&a).set_avatar(image.unwrap_or_default());
    });
}

// ───────────────────────── Profils ─────────────────────────

fn charger_profils(app: &Rc<App>) {
    let id_voulu = app.home.borrow().profil.as_ref().map(|p| p.id.clone()).unwrap_or_else(|| settings::get(settings::DERNIER_PROFIL));

    let mut profils: Vec<Option<Instance>> = vec![None];
    profils.extend(instances::lister().into_iter().map(Some));
    let noms: Vec<String> = profils
        .iter()
        .map(|p| p.as_ref().map(|i| i.nom.clone()).unwrap_or_else(|| loc::t("Home_JeuRapide")))
        .collect();
    let index = profils.iter().position(|p| p.as_ref().is_some_and(|i| i.id == id_voulu)).unwrap_or(0);

    h(app).set_profiles(modele_textes(&noms));
    h(app).set_profile_index(index as i32);
    // cartes de la fenêtre de choix
    let cartes: Vec<crate::ProfileItem> = profils
        .iter()
        .zip(&noms)
        .map(|(p, nom)| {
            let icone = p
                .as_ref()
                .and_then(|i| std::fs::read(i.chemin_icone()).ok())
                .and_then(|o| super::images::decoder(&o, 96))
                .map(slint::Image::from_rgba8);
            crate::ProfileItem {
                name: nom.clone().into(),
                detail: match p {
                    Some(i) => format!("{} · {}", i.version_mc, i.loader),
                    None => loc::t("Home_JeuRapide_Detail"),
                }
                .into(),
                has_icon: icone.is_some(),
                icon: icone.unwrap_or_default(),
                art: super::picker::illustration(&super::picker::cle_groupe(
                    &p.as_ref().map(|i| i.version_mc.clone()).unwrap_or_else(|| settings::get(settings::LAST_VERSION)),
                    0,
                )),
            }
        })
        .collect();
    h(app).set_profile_items(modele(cartes));
    // icônes distantes (modpacks sans icon.png) : chargées ensuite, sans bloquer la liste
    for (i, p) in profils.iter().enumerate() {
        let Some(instance) = p else { continue };
        let Some(url) = instance.icone_url.clone().filter(|_| !instance.chemin_icone().exists()) else { continue };
        let (a, nom) = (app.clone(), instance.nom.clone());
        lancer(async move {
            use slint::Model;
            let Some(image) = images::depuis_url(&a, &url).await else { return };
            let liste = h(&a).get_profile_items();
            // la liste a pu changer entre-temps : on vérifie que c'est toujours ce profil
            if let Some(mut item) = liste.row_data(i).filter(|item| item.name.as_str() == nom) {
                item.icon = image;
                item.has_icon = true;
                liste.set_row_data(i, item);
            }
        });
    }
    {
        let mut e = app.home.borrow_mut();
        e.profil = profils[index].clone();
        e.profils = profils;
    }
    maj_mode_profil(app);
}

/// Bascule l'affichage entre le jeu rapide et le profil sélectionné.
fn maj_mode_profil(app: &Rc<App>) {
    let (profil, version, modes_charges, installation, jeu) = {
        let e = app.home.borrow();
        (e.profil.clone(), e.version_choisie.clone(), e.modes_charges, e.installation_en_cours, e.jeux_en_cours > 0)
    };
    let s = h(app);
    s.set_quick(profil.is_none());

    match &profil {
        Some(p) => {
            s.set_card_label(format!("{} · {}", loc::t("Home_Profil"), p.nom.to_uppercase()).into());
            s.set_card_version(p.version_mc.clone().into());
            s.set_card_loader(p.loader.to_uppercase().into());
            s.set_summary(resume_contenu(p).into());
            // L'icône du profil remplace le bloc d'herbe
            s.set_has_card_icon(false);
            let (fichier, url) = (p.chemin_icone(), p.icone_url.clone());
            let id = p.id.clone();
            let a = app.clone();
            lancer(async move {
                let image = match images::depuis_fichier(&fichier, 256) {
                    Some(i) => Some(i),
                    None => match url {
                        Some(u) => images::depuis_url(&a, &u).await,
                        None => None,
                    },
                };
                if a.home.borrow().profil.as_ref().map(|p| p.id.as_str()) == Some(id.as_str()) {
                    if let Some(i) = image {
                        h(&a).set_card_icon(i);
                        h(&a).set_has_card_icon(true);
                    }
                }
            });
        }
        None if s.get_misty_selected() => {
            s.set_card_label("MISTY CLIENT".into());
            s.set_card_version(s.get_misty_mc());
            s.set_card_loader("MISTY".into());
            s.set_has_card_icon(false);
        }
        None => {
            s.set_card_label(loc::t("Home_Version_Selected").into());
            s.set_card_version(if version.is_empty() { "—".into() } else { version.into() });
            let mode = app.home.borrow().modes.get(s.get_mode_index().max(0) as usize).cloned().unwrap_or_else(|| loaders::VANILLA.into());
            s.set_card_loader(mode.to_uppercase().into());
            s.set_has_card_icon(false);
        }
    }

    if !installation {
        let pret = profil.is_some() || modes_charges;
        s.set_play_enabled(pret);
        if pret {
            statut(app, &loc::t(if jeu { "Home_EnCours" } else { "Home_Pret" }));
        }
    }
}

/// "12 mods · 2 shaders"
fn resume_contenu(instance: &Instance) -> String {
    let mut compte: Vec<(String, usize)> = Vec::new();
    for c in content::lister(instance).into_iter().filter(|c| c.actif()) {
        match compte.iter_mut().find(|(t, _)| *t == c.type_) {
            Some((_, n)) => *n += 1,
            None => compte.push((c.type_.clone(), 1)),
        }
    }
    if compte.is_empty() {
        return loc::t("Home_AucunContenu");
    }
    compte.iter().map(|(t, n)| format!("{n} {}", content::libelle_nombre(t, *n))).collect::<Vec<_>>().join(" · ")
}

/// Sélectionne un profil et lance la partie (depuis la page Profils).
pub fn selectionner_et_jouer(app: &Rc<App>, instance: Instance) {
    settings::set(settings::DERNIER_PROFIL, &instance.id);
    app.home.borrow_mut().profil = Some(instance);
    charger_profils(app);
    if !app.home.borrow().installation_en_cours {
        let a = app.clone();
        lancer(async move { jouer(&a).await });
    }
}

// ───────────────────────── Versions (jeu rapide) ─────────────────────────

async fn charger_versions(app: &Rc<App>) {
    match app.bg(install::versions_proposees()).await {
        Ok(v) => {
            h(app).set_versions(modele_textes(&v));
            app.home.borrow_mut().versions = v;
            // la fenêtre de choix propose toutes les versions, filtrées par type
            if let Ok(m) = app.bg(install::manifeste()).await {
                let toutes = m
                    .into_iter()
                    .filter_map(|v| {
                        let kind = match v.type_.as_str() {
                            "release" => 0,
                            "snapshot" => 1,
                            "old_beta" => 2,
                            "old_alpha" => 3,
                            _ => return None,
                        };
                        let annee = v.release_time.get(..4).unwrap_or("").to_string();
                        Some(super::picker::Entree { id: v.id, kind, annee })
                    })
                    .collect();
                app.home.borrow_mut().toutes = toutes;
                filtrer_versions(app);
            }
        }
        Err(e) => {
            statut(app, &loc::t("Home_VersionsImpossible"));
            dialogs::erreur(app, &loc::t("Home_VersionsIndisponibles"), &loc::tf("Home_VersionsErreur", &[&e])).await;
        }
    }
}

/// Fenêtre de choix : versions filtrées (types, mod loader compatible, recherche), regroupées par
/// mise à jour ; la mise à jour ouverte montre ses versions à droite.
fn filtrer_versions(app: &App) {
    use super::picker;
    let s = h(app);
    let mut types = [s.get_picker_release(), s.get_picker_snapshot(), s.get_picker_beta(), s.get_picker_alpha()];
    // aucun type coché : les releases reviennent (une liste vide n'aurait rien à proposer)
    if !types.iter().any(|&t| t) {
        s.set_picker_release(true);
        types[0] = true;
    }
    let loader = s.get_picker_loader();
    let recherche = s.get_picker_search().trim().to_lowercase();
    let (groupes, choisi) = {
        let e = app.home.borrow();
        let mut groupes: Vec<(String, Vec<picker::Entree>)> = Vec::new();
        for v in e.toutes.iter().filter(|v| {
            types[v.kind as usize] && picker::compatible(&v.id, loader) && (recherche.is_empty() || v.id.to_lowercase().contains(&recherche))
        }) {
            let cle = picker::cle_groupe(&v.id, v.kind);
            match groupes.iter_mut().find(|(c, _)| *c == cle) {
                Some((_, liste)) => liste.push(v.clone()),
                None => groupes.push((cle, vec![v.clone()])),
            }
        }
        // mise à jour ouverte : celle choisie, sinon celle de la version actuelle, sinon la première
        let actuelle = picker::cle_groupe(&e.version_choisie, 0);
        let choisi = groupes
            .iter()
            .position(|(c, _)| *c == e.groupe_choisi)
            .or_else(|| groupes.iter().position(|(c, _)| *c == actuelle))
            .unwrap_or(0);
        (groupes, choisi)
    };
    let cartes: Vec<crate::VersionGroup> = groupes
        .iter()
        .map(|(cle, liste)| {
            let (titre, nom) = picker::titre_groupe(cle);
            let annees: Vec<&str> = liste.iter().map(|v| v.annee.as_str()).filter(|a| !a.is_empty()).collect();
            let (min, max) = (annees.iter().min().copied().unwrap_or(""), annees.iter().max().copied().unwrap_or(""));
            crate::VersionGroup {
                key: cle.clone().into(),
                title: titre.into(),
                name: nom.into(),
                art: picker::illustration(cle),
                count: if liste.len() == 1 { loc::t("Ver_Nombre1") } else { loc::tf("Ver_Nombre", &[&liste.len()]) }.into(),
                years: if min == max { min.to_string() } else { format!("{min} – {max}") }.into(),
            }
        })
        .collect();
    let sous: Vec<crate::VersionItem> = groupes
        .get(choisi)
        .map(|(_, liste)| liste.iter().map(|v| crate::VersionItem { id: v.id.clone().into(), kind: v.kind }).collect())
        .unwrap_or_default();
    {
        let mut e = app.home.borrow_mut();
        e.cles_groupes = groupes.iter().map(|(c, _)| c.clone()).collect();
        if let Some((c, _)) = groupes.get(choisi) {
            e.groupe_choisi = c.clone();
        }
    }
    s.set_picker_groups(modele(cartes));
    s.set_picker_group(if groupes.is_empty() { -1 } else { choisi as i32 });
    s.set_picker_sub(modele(sous));
}

/// Une version choisie dans la grande fenêtre : on passe en jeu rapide sur cette version (même si
/// son type est masqué dans les options).
async fn version_choisie_par_id(app: &Rc<App>, id: String) {
    h(app).set_horizon_selected(false);
    misty_quitter(app);
    app.home.borrow_mut().loader_voulu = super::picker::nom_loader(h(app).get_picker_loader()).map(str::to_string);
    aller_a_version(app, id).await;
}

/// Jeu rapide sur la version `id` (le loader voulu est déjà noté).
async fn aller_a_version(app: &Rc<App>, id: String) {
    if app.home.borrow().profil.is_some() {
        settings::set(settings::DERNIER_PROFIL, "");
        app.home.borrow_mut().profil = None;
        h(app).set_profile_index(0);
        maj_mode_profil(app);
    }
    let index = {
        let mut e = app.home.borrow_mut();
        match e.versions.iter().position(|v| *v == id) {
            Some(i) => i,
            None => {
                e.versions.insert(0, id.clone());
                0
            }
        }
    };
    let versions = app.home.borrow().versions.clone();
    h(app).set_versions(modele_textes(&versions));
    h(app).set_version_index(index as i32);
    version_choisie(app, index as i32).await;
}

fn charger_derniere_version(app: &Rc<App>) {
    let derniere = settings::get(settings::LAST_VERSION);
    let index = {
        let e = app.home.borrow();
        e.versions.iter().position(|v| *v == derniere).or(if e.versions.is_empty() { None } else { Some(0) }) // la plus récente
    };
    if let Some(i) = index {
        h(app).set_version_index(i as i32);
        let a = app.clone();
        lancer(async move { version_choisie(&a, i as i32).await });
    }
}

async fn recharger_versions(app: &Rc<App>) {
    let actuelle = app.home.borrow().version_choisie.clone();
    h(app).set_reload_enabled(false);
    if app.home.borrow().profil.is_none() {
        statut(app, &loc::t("Home_ChargementVersions"));
    }
    charger_versions(app).await;

    let index = app.home.borrow().versions.iter().position(|v| *v == actuelle);
    match index {
        Some(i) => h(app).set_version_index(i as i32),
        None => {
            h(app).set_version_index(-1);
            if app.home.borrow().profil.is_none() {
                statut(app, &loc::t("Home_ChoisisVersion"));
            }
        }
    }
    h(app).set_reload_enabled(true);
}

async fn version_choisie(app: &Rc<App>, i: i32) {
    let Some(version) = app.home.borrow().versions.get(i.max(0) as usize).cloned() else { return };
    {
        let mut e = app.home.borrow_mut();
        e.modes_charges = false;
        e.modes.clear();
        e.version_choisie = version.clone();
    }
    h(app).set_modes_enabled(false);
    h(app).set_modes(modele_textes(&[]));
    h(app).set_mode_index(-1);
    settings::set(settings::LAST_VERSION, &version);

    let rapide = app.home.borrow().profil.is_none();
    if rapide {
        h(app).set_play_enabled(false);
        h(app).set_card_version(version.clone().into());
        statut(app, &loc::tf("Commun_RechercheLoaders", &[&version]));
    }

    let v = version.clone();
    let modes = app.bg(async move { loaders::modes_disponibles(&v).await }).await;

    // L'utilisateur a changé de version pendant le chargement : on laisse la nouvelle sélection gérer
    if app.home.borrow().version_choisie != version {
        return;
    }
    h(app).set_modes(modele_textes(&modes));
    // loader filtré dans la fenêtre de choix : présélectionné s'il existe pour cette version
    let voulu = app.home.borrow_mut().loader_voulu.take();
    let index = voulu.and_then(|l| modes.iter().position(|m| *m == l)).unwrap_or(0);
    h(app).set_mode_index(index as i32);
    if index > 0 && app.home.borrow().profil.is_none() {
        h(app).set_card_loader(modes[index].to_uppercase().into());
    }
    let installation = app.home.borrow().installation_en_cours;
    h(app).set_modes_enabled(!installation);
    {
        let mut e = app.home.borrow_mut();
        e.modes = modes;
        e.modes_charges = true;
    }
    if app.home.borrow().profil.is_none() {
        maj_mode_profil(app);
    }
}

// ───────────────────────── Lancement ─────────────────────────

async fn jouer(app: &Rc<App>) {
    let misty = h(app).get_misty_selected() && app.home.borrow().profil.is_none() && client::disponible();
    let (instance, version, mode) = if misty {
        (None, h(app).get_misty_mc().to_string(), loaders::FABRIC.to_string())
    } else {
        let e = app.home.borrow();
        let mode_rapide = e.modes.get(h(app).get_mode_index().max(0) as usize).cloned().unwrap_or_else(|| loaders::VANILLA.into());
        let instance = e.profil.clone();
        let version = instance.as_ref().map(|i| i.version_mc.clone()).unwrap_or_else(|| e.version_choisie.clone());
        let mode = instance.as_ref().map(|i| i.loader.clone()).unwrap_or(mode_rapide);
        (instance, version, mode)
    };
    if version.is_empty() || app.home.borrow().installation_en_cours {
        return;
    }

    let joueur = joueur_actuel(app);
    app.home.borrow_mut().installation_en_cours = true;
    h(app).set_play_enabled(false);
    h(app).set_selectors_enabled(false);
    let nom = match &instance {
        Some(i) => i.nom.clone(),
        None if misty => format!("Misty {version}"),
        None => format!("{version} {mode}"),
    };
    afficher_progression(app, 0.0, &loc::tf("Home_Preparation", &[&nom]));

    // Téléchargement de la version et du loader (annulable)
    let an = debut_annulable(app);
    let progression = progression_fichiers(app);
    let (v, m, lv) = (version.clone(), mode.clone(), instance.as_ref().and_then(|i| i.loader_version.clone()));
    let resultat = an
        .tache(app, async move {
            let dirs = McDirs::misty();
            let nom_version = loaders::installer(&dirs, &v, &m, lv.as_deref(), progression.clone()).await?;
            install::installer(&dirs, &nom_version, progression).await
        })
        .await;
    fin_annulable(app);

    match resultat {
        Ok(installee) => {
            afficher_progression(app, 100.0, &loc::t("Home_Lancement"));
            lancer_jeu(app, &version, &mode, installee, instance, joueur, misty).await;
        }
        Err(e) if est_annule(&e) => {
            crate::services::journal::info(&format!("Téléchargement annulé : {nom}"));
            statut(app, &loc::t("Home_Annule"));
            h(app).set_progress(0.0);
        }
        Err(e) => {
            statut(app, &loc::t("Home_EchecTelechargement"));
            dialogs::erreur(app, &loc::t("Home_TelechargementImpossible"), &format!("{e:#}")).await;
        }
    }

    app.home.borrow_mut().installation_en_cours = false;
    let s = h(app);
    s.set_play_text(loc::t("Home_Bouton_Jouer").into());
    s.set_play_enabled(true);
    s.set_selectors_enabled(true);
    s.set_modes_enabled(app.home.borrow().modes_charges);
    s.set_percent_text("".into());
}

/// Le compte choisi maintenant : la session Microsoft, sinon le pseudo hors ligne.
fn joueur_actuel(app: &App) -> launch::Joueur {
    match account::session() {
        Some(s) => launch::Joueur { pseudo: s.pseudo, uuid: s.uuid, jeton: s.jeton, xuid: s.xuid, type_compte: "msa".into() },
        None => {
            let pseudo = pseudo_actuel(app);
            launch::Joueur { uuid: account::uuid_hors_ligne(&pseudo), pseudo, jeton: "0".into(), xuid: String::new(), type_compte: "legacy".into() }
        }
    }
}

async fn lancer_jeu(app: &Rc<App>, version: &str, mode: &str, installee: install::VersionInstallee, instance: Option<Instance>, joueur: launch::Joueur, misty: bool) {
    let pseudo_joueur = joueur.pseudo.clone();
    // Un profil a son propre dossier de jeu (mods, options, mondes) ; le jeu rapide utilise le dossier principal
    let options = launch::Options {
        joueur,
        ram_mo: settings::ram_mo(),
        dossier_jeu: instance.as_ref().map(|i| i.dossier()).unwrap_or_else(crate::services::paths::data_dir),
    };

    let titre = match &instance {
        Some(i) => format!("{} · {version} {mode}", i.nom),
        None if misty => format!("Misty {version} · Misty Client"),
        None => format!("Minecraft {version} · {mode}"),
    };
    // Misty Client : posé dans les mods du profil s'il le supporte, retiré sinon (toujours posé
    // quand on lance la version Misty)
    let voulu = misty || instance.as_ref().map(client::voulu).unwrap_or_else(client::active);
    client::preparer(&options.dossier_jeu, version, mode, voulu);
    client::ecrire_lien();
    // serveurs et barres d'objets : les mêmes dans toutes les versions et tous les profils
    crate::services::partage::avant_partie(&options.dossier_jeu);
    let dossier_partie = options.dossier_jeu.clone();
    let partie = journal::nouvelle_partie(&titre);
    if super::console::auto() {
        super::console::ouvrir(app);
    }
    let resultat = app.bloquant(move || launch::lancer(&McDirs::misty(), &installee, &options)).await;
    let mut processus = match resultat {
        Ok(p) => p,
        Err(e) => {
            journal::ajouter(partie, &format!("{e:#}"), journal::ERREUR);
            journal::fin(partie, Some(-1));
            statut(app, &loc::t("Home_EchecLancement"));
            dialogs::erreur(app, &loc::t("Home_LancementImpossible"), &format!("{e:#}")).await;
            return;
        }
    };
    let fils = journal::suivre(partie, &mut processus, None);
    telemetry::evenement("game_start", serde_json::json!({ "minecraft": version, "loader": mode, "profile": instance.is_some() }));
    let (v_tel, m_tel, pseudo_tel) = (version.to_string(), mode.to_string(), pseudo_joueur.clone());

    let detail = match &instance {
        Some(i) => format!("{mode} · {}", i.nom),
        None => mode.to_string(),
    };
    discord::definir(discord::Statut::EnJeu { version: version.to_string(), mode: detail, pseudo: pseudo_joueur });
    app.home.borrow_mut().jeux_en_cours += 1;
    statut(app, &loc::tf("Home_Lance", &[&instance.as_ref().map(|i| i.nom.clone()).unwrap_or_else(|| format!("Minecraft {version}"))]));

    if let Some(mut i) = instance {
        i.dernier_lancement = Some(chrono::Local::now());
        let _ = instances::sauver(&i);
    }

    // Fin du jeu : on revient à l'état « prêt »
    let faible = app.ui.as_weak();
    std::thread::spawn(move || {
        let code = processus.wait().ok().and_then(|s| s.code());
        for f in fils {
            let _ = f.join();
        }
        journal::fin(partie, code);
        crate::services::partage::apres_partie(&dossier_partie);
        let etat = journal::etat(partie).unwrap_or_default();
        let plante = matches!(code, Some(n) if n != 0);
        telemetry::evenement(
            "game_end",
            serde_json::json!({ "minecraft": v_tel, "loader": m_tel, "code": code, "seconds": etat.duree.as_secs(), "crashed": plante, "errors": etat.erreurs }),
        );
        if plante {
            telemetry::plantage(partie, &pseudo_tel);
        }
        let _ = faible.upgrade_in_event_loop(|_| {
            if let Some(a) = crate::APP.with(|a| a.borrow().clone()) {
                let reste = {
                    let mut e = a.home.borrow_mut();
                    e.jeux_en_cours = e.jeux_en_cours.saturating_sub(1);
                    e.jeux_en_cours
                };
                // retour au statut de la page affichée (quand plus aucune partie ne tourne)
                if reste == 0 {
                    super::maj_discord(&a, true);
                }
                statut(&a, &loc::t("Home_Pret"));
                let profil = a.home.borrow().profil.clone();
                if let Some(p) = profil {
                    h(&a).set_summary(resume_contenu(&p).into());
                }
            }
        });
    });
}

// ───────────────────────── Affichage ─────────────────────────

/// Avancement des téléchargements (limité à ~20 mises à jour par seconde).
fn progression_fichiers(app: &Rc<App>) -> FileProgress {
    let faible = app.ui.as_weak();
    let derniere = Arc::new(AtomicU64::new(0));
    Arc::new(move |fait: usize, total: usize, nom: &str| {
        if total == 0 {
            return;
        }
        let maintenant = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_millis() as u64).unwrap_or(0);
        if fait < total && maintenant.saturating_sub(derniere.load(Ordering::Relaxed)) < 50 {
            return;
        }
        derniere.store(maintenant, Ordering::Relaxed);
        let pourcentage = fait as f32 * 100.0 / total as f32;
        let texte = loc::tf("Home_Telechargement", &[&nom]);
        let _ = faible.upgrade_in_event_loop(move |ui| {
            if let Some(a) = crate::APP.with(|a| a.borrow().clone()) {
                if a.home.borrow().annulation.as_ref().is_some_and(|an| an.annulee()) {
                    return;
                }
                afficher_progression(&a, pourcentage, &texte);
            } else {
                let _ = ui;
            }
        });
    })
}

fn afficher_progression(app: &App, pourcentage: f32, texte: &str) {
    let s = h(app);
    s.set_status(texte.into());
    s.set_percent_text(format!("{pourcentage:.0}%").into());
    s.set_progress(pourcentage / 100.0);
    if app.home.borrow().installation_en_cours {
        s.set_play_text(format!("{pourcentage:.0}%").into());
    }
}

// ───────────────────────── Horizon ─────────────────────────

fn taille_mo(octets: u64) -> String {
    format!("{:.0} {}", octets as f64 / 1_048_576.0, loc::t("Unite_Mo"))
}

/// Liste des builds : ceux de GitHub, plus ceux installés qui n'y sont plus (les nightlies ne restent
/// que quelques jours sur GitHub).
fn horizon_afficher(app: &App) {
    let installes = horizon::installes();
    let mut lignes: Vec<crate::HorizonBuild> = app
        .home
        .borrow()
        .horizon_builds
        .iter()
        .map(|b| crate::HorizonBuild {
            tag: b.tag.clone().into(),
            date: b.date.clone().into(),
            size: taille_mo(b.taille).into(),
            nightly: b.nightly,
            installed: installes.contains(&b.tag),
        })
        .collect();
    for tag in &installes {
        if !lignes.iter().any(|l| l.tag == tag.as_str()) {
            lignes.push(crate::HorizonBuild {
                tag: tag.clone().into(),
                date: "".into(),
                size: "".into(),
                nightly: tag.starts_with("nightly"),
                installed: true,
            });
        }
    }
    let s = h(app);
    // build choisi : toujours un build installé
    let choisi = s.get_horizon_build().to_string();
    if !installes.contains(&choisi) {
        let nouveau = installes.first().cloned().unwrap_or_default();
        settings::set(settings::HORIZON_BUILD, &nouveau);
        s.set_horizon_build(nouveau.into());
    }
    // place de chaque build dans la grille selon le filtre (Tous, Nightly, Stables)
    let filtre = s.get_horizon_filter();
    let mut n = 0;
    let rangs: Vec<i32> = lignes
        .iter()
        .map(|l| {
            if filtre == 0 || (filtre == 1) == l.nightly {
                n += 1;
                n - 1
            } else {
                -1
            }
        })
        .collect();
    s.set_horizon_builds(modele(lignes));
    s.set_horizon_ranks(modele(rangs));
    s.set_horizon_visibles(n);
}

async fn horizon_rafraichir(app: &Rc<App>) {
    let s = h(app);
    horizon_afficher(app);
    s.set_horizon_loading(true);
    match app.bg(horizon::builds_disponibles()).await {
        Ok(b) => {
            app.home.borrow_mut().horizon_builds = b;
            s.set_horizon_message("".into());
        }
        Err(_) => s.set_horizon_message(loc::t("Hz_ListeImpossible").into()),
    }
    s.set_horizon_loading(false);
    horizon_afficher(app);
}

/// Télécharge un build depuis GitHub (barre d'avancement sur sa ligne).
async fn horizon_installer(app: &Rc<App>, tag: String) {
    let Some(build) = app.home.borrow().horizon_builds.iter().find(|b| b.tag == tag).cloned() else { return };
    let s = h(app);
    s.set_horizon_busy(true);
    s.set_horizon_busy_tag(tag.clone().into());
    s.set_horizon_progress(0.0);
    s.set_horizon_message(loc::tf("Hz_TelechargementBuild", &[&tag]).into());
    let fenetre = app.ui.as_weak();
    let dernier = std::sync::Arc::new(std::sync::atomic::AtomicU32::new(0));
    let resultat = app
        .bg(async move {
            horizon::installer_build(&build, move |f| {
                let pourcent = (f * 100.0) as u32;
                if dernier.swap(pourcent, std::sync::atomic::Ordering::SeqCst) != pourcent {
                    let _ = fenetre.upgrade_in_event_loop(move |ui| ui.global::<HomeState>().set_horizon_progress(f));
                }
            })
            .await
        })
        .await;
    s.set_horizon_busy(false);
    s.set_horizon_busy_tag("".into());
    match resultat {
        Ok(()) => {
            s.set_horizon_message("".into());
            if s.get_horizon_build().is_empty() {
                settings::set(settings::HORIZON_BUILD, &tag);
                s.set_horizon_build(tag.into());
            }
        }
        Err(e) => s.set_horizon_message(e.to_string().into()),
    }
    horizon_afficher(app);
}

async fn horizon_supprimer(app: &Rc<App>, tag: String) {
    let ok = dialogs::confirmer(app, &loc::t("Hz_Supprimer"), &tag, Some(&loc::t("Commun_Supprimer")), None, true).await;
    if !ok {
        return;
    }
    if let Err(e) = app.bloquant(move || horizon::supprimer_build(&tag)).await {
        h(app).set_horizon_message(e.to_string().into());
    }
    horizon_afficher(app);
}

/// La version Misty choisie avant (gardée dans les réglages), sinon la plus récente.
fn version_misty() -> String {
    let choisie = settings::get(settings::MISTY_VERSION);
    if client::existe(&choisie) {
        choisie
    } else {
        client::derniere().to_string()
    }
}

/// Cette version Misty devient le jeu lancé par JOUER : jeu rapide sur cette version, en Fabric,
/// avec Misty Client (même si le réglage « Misty Client en jeu » est coupé).
fn misty_choisir(app: &Rc<App>, version: String) {
    if !client::existe(&version) {
        return;
    }
    let s = h(app);
    s.set_misty_mc(version.as_str().into());
    s.set_misty_art(super::picker::illustration(&super::picker::cle_groupe(&version, 0)));
    settings::set(settings::MISTY_VERSION, &version);
    s.set_horizon_selected(false);
    s.set_misty_selected(true);
    s.set_picker_misty(true);
    settings::set_bool(settings::MISTY_CHOISI, true);
    let a = app.clone();
    lancer(async move {
        if a.home.borrow().profil.is_some() {
            settings::set(settings::DERNIER_PROFIL, "");
            a.home.borrow_mut().profil = None;
            h(&a).set_profile_index(0);
        }
        a.home.borrow_mut().loader_voulu = Some(loaders::FABRIC.to_string());
        aller_a_version(&a, version).await;
        maj_mode_profil(&a);
    });
    maj_mode_profil(app);
}

/// Une autre version, un profil ou Horizon est choisi : JOUER ne lance plus Misty.
fn misty_quitter(app: &App) {
    let s = h(app);
    if s.get_misty_selected() {
        s.set_misty_selected(false);
        settings::set_bool(settings::MISTY_CHOISI, false);
    }
}

/// Horizon devient le jeu lancé par JOUER (barre du bas et carte de l'accueil).
fn horizon_choisir(app: &App) {
    let s = h(app);
    misty_quitter(app);
    s.set_horizon_selected(true);
    s.set_card_label(loc::t("Hz_Etiquette").into());
    s.set_card_version("26.3".into());
    s.set_card_loader("HORIZON".into());
}

fn horizon_ouvrir_panneau(app: &App) {
    let s = h(app);
    s.set_picker_tab(0);
    s.set_picker_horizon(true);
    s.set_picker_open(true);
    s.invoke_horizon_refresh();
}

/// Lance le build choisi. La première fois : Minecraft (la version qu'Horizon prépare) et son Java
/// sont installés par Misty, le server.jar est téléchargé, puis horizon-prepare prépare le build.
async fn horizon_jouer(app: &Rc<App>) {
    // compte Microsoft : session en ligne ; sinon hors ligne, sous le pseudo choisi
    let compte = account::session();
    let pseudo = compte.as_ref().map(|c| c.pseudo.clone()).unwrap_or_else(|| pseudo_actuel(app));
    let tag = h(app).get_horizon_build().to_string();
    let Some(build) = horizon::dossier_build(&tag) else {
        horizon_ouvrir_panneau(app);
        return;
    };
    let s = h(app);
    s.set_play_enabled(false);
    s.set_selectors_enabled(false);
    let an = debut_annulable(app);
    let resultat: anyhow::Result<()> = async {
        let b = build.clone();
        let exigences = app.bloquant(move || horizon::exigences(&b)).await?;
        // Minecraft et son Java, installés comme pour n'importe quelle version
        afficher_progression(app, 0.0, &loc::tf("Hz_InstallationMinecraft", &[&exigences.minecraft]));
        let progression = progression_fichiers(app);
        let mc = exigences.minecraft.clone();
        let installee = an.tache(app, async move { install::installer(&McDirs::misty(), &mc, progression).await }).await?;
        let b = build.clone();
        let ex = exigences.clone();
        if !app.bloquant(move || horizon::est_prepare(&b, &ex)).await {
            let json = installee.json.clone();
            let mc = exigences.minecraft.clone();
            let jar = an.tache(app, async move { horizon::server_jar(&json, &mc).await }).await?;
            let (b, ex) = (build.clone(), exigences.clone());
            an.tache(app, async move { horizon::manifeste_epingle(&b, &ex).await }).await?;
            an.verifier()?;
            let java_home = installee.java.parent().and_then(|bin| bin.parent()).map(|p| p.to_path_buf()).unwrap_or_default();
            afficher_progression(app, 0.0, &loc::t("Hz_Preparation"));
            let fenetre = app.ui.as_weak();
            let b = build.clone();
            let ex = exigences.clone();
            app.bloquant(move || {
                horizon::preparer(&b, &ex, &McDirs::misty().root, &jar, &java_home, move |f, message| {
                    let texte = if message.is_empty() { String::new() } else { message.to_string() };
                    let _ = fenetre.upgrade_in_event_loop(move |_| {
                        if let Some(a) = crate::APP.with(|a| a.borrow().clone()) {
                            let libelle = if texte.is_empty() { loc::t("Hz_Preparation") } else { texte };
                            afficher_progression(&a, f * 100.0, &libelle);
                        }
                    });
                })
            })
            .await?;
            an.verifier()?;
        }
        // la partie : plus rien à annuler
        an.verifier()?;
        fin_annulable(app);
        afficher_progression(app, 100.0, &loc::t("Hz_Lancement"));
        discord::definir(discord::Statut::EnJeu { version: exigences.minecraft.clone(), mode: format!("Horizon {tag}"), pseudo: pseudo.clone() });
        app.home.borrow_mut().jeux_en_cours += 1;
        let session = compte.as_ref().map(|c| horizon::Session { pseudo: c.pseudo.clone(), uuid: c.uuid.clone(), jeton: c.jeton.clone(), xuid: c.xuid.clone() });
        let b = build.clone();
        let ex = exigences.clone();
        let p = pseudo.clone();
        let partie = journal::nouvelle_partie(&format!("Horizon {tag} · Minecraft {}", exigences.minecraft));
        if super::console::auto() {
            super::console::ouvrir(app);
        }
        telemetry::evenement("game_start", serde_json::json!({ "minecraft": exigences.minecraft, "loader": "horizon", "build": tag }));
        // une autre partie peut être lancée pendant que celle-ci tourne
        s.set_play_enabled(true);
        s.set_selectors_enabled(true);
        let fin = app.bloquant(move || horizon::jouer(partie, &b, &ex, session.as_ref(), &p)).await;
        let etat = journal::etat(partie).unwrap_or_default();
        let plante = matches!(etat.code, Some(n) if n != 0);
        telemetry::evenement(
            "game_end",
            serde_json::json!({ "minecraft": exigences.minecraft, "loader": "horizon", "build": tag, "code": etat.code, "seconds": etat.duree.as_secs(), "crashed": plante, "errors": etat.erreurs }),
        );
        if plante {
            telemetry::plantage(partie, &pseudo);
        }
        let reste = {
            let mut e = app.home.borrow_mut();
            e.jeux_en_cours = e.jeux_en_cours.saturating_sub(1);
            e.jeux_en_cours
        };
        if reste == 0 {
            super::maj_discord(app, true);
        }
        fin
    }
    .await;
    fin_annulable(app);
    s.set_play_enabled(true);
    s.set_selectors_enabled(true);
    match resultat {
        Ok(()) => afficher_progression(app, 0.0, &loc::t("Hz_Ferme")),
        Err(e) if est_annule(&e) => {
            afficher_progression(app, 0.0, &loc::t("Home_Annule"));
            h(app).set_percent_text("".into());
        }
        Err(e) => {
            afficher_progression(app, 0.0, &loc::t("Hz_Erreur"));
            dialogs::erreur(app, &loc::t("Hz_Erreur"), &format!("{e:#}")).await;
        }
    }
}
