//! Interface : relie les pages Slint (ui/*.slint) aux services.
//!
//! Principe :
//! - toute la logique d'interface tourne sur le thread de l'interface, dans des tâches `slint::spawn_local`
//!   (l'équivalent des `async void` du C#) ;
//! - le travail lourd (réseau, fichiers) part sur le runtime tokio avec `bg(...)`, et on attend le résultat ;
//! - chaque page a son module, avec son état dans un `RefCell`.

pub mod browser;
pub mod changelog;
pub mod dialogs;
pub mod editor;
pub mod home;
pub mod images;
pub mod instance;
pub mod news;
pub mod packs;
pub mod profiles;
pub mod scene;
pub mod settings;
pub mod skin;

use crate::services::{discord, loc, paths, system_info, Progress};
use crate::*;
use slint::{ComponentHandle, Model, ModelRc, SharedString, VecModel};
use std::cell::RefCell;
use std::future::Future;
use std::rc::Rc;
use std::sync::Arc;

/// Pages (doivent correspondre à AppState.page dans state.slint).
pub mod page {
    pub const ACCUEIL: i32 = 0;
    pub const PROFILS: i32 = 1;
    pub const OPTIONS: i32 = 2;
    #[allow(dead_code)]
    pub const NOTES: i32 = 3;
    pub const PACKS: i32 = 4;
    pub const INSTANCE: i32 = 5;
    pub const EDITEUR: i32 = 6;
}

/// L'application : la fenêtre, le runtime et l'état de chaque page.
pub struct App {
    pub ui: MainWindow,
    pub rt: tokio::runtime::Handle,
    pub dialog: RefCell<Option<futures::channel::oneshot::Sender<Option<i32>>>>,
    pub images: images::Cache,
    pub home: RefCell<home::Etat>,
    pub profiles: RefCell<profiles::Etat>,
    pub instance: RefCell<instance::Etat>,
    pub browser: RefCell<browser::Etat>,
    pub settings: RefCell<settings::Etat>,
    pub packs: RefCell<packs::Etat>,
    pub editor: RefCell<editor::Etat>,
    pub skin: RefCell<skin::Etat>,
    /// Lien de la news affichée sur l'accueil (vide = notes de version du launcher)
    pub news_url: RefCell<String>,
}

impl App {
    pub fn new(ui: MainWindow, rt: tokio::runtime::Handle) -> Rc<App> {
        Rc::new(App {
            ui,
            rt,
            dialog: RefCell::new(None),
            images: images::Cache::default(),
            home: RefCell::default(),
            profiles: RefCell::default(),
            instance: RefCell::default(),
            browser: RefCell::default(),
            settings: RefCell::default(),
            packs: RefCell::default(),
            editor: RefCell::default(),
            news_url: RefCell::default(),
            skin: RefCell::default(),
        })
    }

    /// Lance un travail sur le runtime tokio et attend son résultat.
    pub fn bg<T, F>(&self, f: F) -> impl Future<Output = T>
    where
        F: Future<Output = T> + Send + 'static,
        T: Send + 'static,
    {
        let tache = self.rt.spawn(f);
        async move { tache.await.expect("tâche de fond interrompue") }
    }

    /// Lance un calcul bloquant (fichiers, images…) hors du thread de l'interface.
    pub fn bloquant<T, F>(&self, f: F) -> impl Future<Output = T>
    where
        F: FnOnce() -> T + Send + 'static,
        T: Send + 'static,
    {
        let tache = self.rt.spawn_blocking(f);
        async move { tache.await.expect("tâche de fond interrompue") }
    }

    /// Fonction de progression qui affiche le texte avec `afficher` (sur le thread de l'interface).
    pub fn progression(&self, afficher: impl Fn(&MainWindow, String) + Send + Sync + Clone + 'static) -> Progress {
        let faible = self.ui.as_weak();
        Arc::new(move |texte: String| {
            let afficher = afficher.clone();
            let _ = faible.upgrade_in_event_loop(move |ui| afficher(&ui, texte));
        })
    }
}

/// Lance une tâche asynchrone sur le thread de l'interface.
pub fn lancer(f: impl Future<Output = ()> + 'static) {
    let _ = slint::spawn_local(f);
}

/// Vec<String> → modèle Slint de textes.
pub fn modele_textes(textes: &[String]) -> ModelRc<SharedString> {
    ModelRc::new(VecModel::from(textes.iter().map(SharedString::from).collect::<Vec<_>>()))
}

/// Vec<T> → modèle Slint.
pub fn modele<T: Clone + 'static>(v: Vec<T>) -> ModelRc<T> {
    ModelRc::new(VecModel::from(v))
}

/// Modifie une ligne d'un modèle (s'il a encore cette ligne).
pub fn maj_ligne<T: Clone + 'static>(m: &ModelRc<T>, i: usize, f: impl FnOnce(&mut T)) {
    if let Some(mut ligne) = m.row_data(i) {
        f(&mut ligne);
        m.set_row_data(i, ligne);
    }
}

// ───────────────────────── Démarrage ─────────────────────────

pub fn installer(app: &Rc<App>) {
    let ui = &app.ui;

    // Traductions
    ui.global::<Tr>().on_lookup(|cle| loc::t(&cle).into());
    ui.global::<Assets>().set_logo(images::logo_nuage());
    ui.global::<AppState>().set_version_text(format!("v{}", crate::services::app_info::VERSION).into());
    ui.global::<AppState>().set_particles(modele(particules()));

    // Paysage du site en fond
    let paysage = scene::generer();
    let etat = ui.global::<AppState>();
    etat.set_scene_ridges(modele(paysage.cretes));
    etat.set_scene_stars(paysage.etoiles);
    etat.set_scene_twinkle(paysage.etoiles_scintillantes);
    etat.set_scene_moon(paysage.lune);
    etat.set_moon_x(paysage.lune_x);
    etat.set_moon_y(paysage.lune_y);
    etat.set_moon_size(paysage.lune_taille);
    etat.set_scene_puffs(modele(paysage.nappes));

    // Fenêtre
    let a = app.clone();
    ui.global::<AppState>().on_drag(move || fenetre(&a, |w| {
        let _ = w.drag_window();
    }));
    let a = app.clone();
    ui.global::<AppState>().on_minimize(move || fenetre(&a, |w| w.set_minimized(true)));
    let a = app.clone();
    ui.global::<AppState>().on_toggle_maximize(move || {
        fenetre(&a, |w| w.set_maximized(!w.is_maximized()));
        suivre_agrandissement(&a);
    });
    // La fenêtre peut aussi être agrandie ou restaurée par Windows (Win+flèches, glisser en haut de l’écran) :
    // on suit son état pour arrondir ou non les coins.
    let a = Rc::downgrade(app);
    let minuterie = slint::Timer::default();
    minuterie.start(slint::TimerMode::Repeated, std::time::Duration::from_millis(300), move || {
        if let Some(a) = a.upgrade() {
            suivre_agrandissement(&a);
        }
    });
    std::mem::forget(minuterie);

    // Apparence enregistrée
    let theme = ui.global::<crate::Theme>();
    theme.set_mode(crate::services::settings::theme());
    theme.set_particles(crate::services::settings::get_bool(crate::services::settings::PARTICULES, true));
    theme.set_blur(crate::services::settings::get_bool(crate::services::settings::FLOU, true));
    appliquer_saison(app);
    ui.global::<AppState>().on_close(|| {
        let _ = slint::quit_event_loop();
    });
    ui.global::<AppState>().on_open_data_folder(|| system_info::ouvrir_dossier(&paths::data_dir()));
    let a = app.clone();
    ui.global::<AppState>().on_nav_clicked(move |i| nav_clicked(&a, i));

    dialogs::installer(app);
    home::installer(app);
    profiles::installer(app);
    instance::installer(app);
    browser::installer(app);
    settings::installer(app);
    changelog::installer(app);
    packs::installer(app);
    editor::installer(app);
    skin::installer(app);
    news::installer(app);
}

/// Thème de saison (Halloween, Noël) si on est à ±3 jours de la fête et que l'option est active.
/// En développement, MISTY_EVENT=1 ou 2 force une saison pour la tester.
pub fn appliquer_saison(app: &App) {
    let mut saison = if crate::services::settings::get_bool(crate::services::settings::SAISON, true) {
        crate::services::settings::saison_du_jour()
    } else {
        0
    };
    #[cfg(debug_assertions)]
    if let Ok(force) = std::env::var("MISTY_EVENT") {
        saison = force.trim().parse().unwrap_or(saison);
    }
    app.ui.global::<crate::Theme>().set_event(saison);
}

/// Met à jour l’indicateur « fenêtre agrandie » (coins carrés, icône du bouton).
fn suivre_agrandissement(app: &App) {
    let mut agrandie = false;
    fenetre(app, |w| agrandie = w.is_maximized());
    let etat = app.ui.global::<AppState>();
    if etat.get_maximized() != agrandie {
        etat.set_maximized(agrandie);
    }
}

/// Accès à la fenêtre winit (déplacement, réduction).
fn fenetre(app: &App, f: impl FnOnce(&slint::winit_030::winit::window::Window)) {
    use slint::winit_030::WinitWindowAccessor;
    app.ui.window().with_winit_window(f);
}

/// Petits carrés blancs translucides qui flottent lentement en fond.
fn particules() -> Vec<Particle> {
    let mut graine: u32 = 7;
    let mut hasard = move || {
        graine = graine.wrapping_mul(1_103_515_245).wrapping_add(12_345);
        ((graine >> 16) & 0x7fff) as f32 / 32768.0
    };
    (0..14)
        .map(|_| {
            let taille = 8.0 + hasard() * 22.0;
            Particle {
                x: 120.0 + hasard() * 1000.0,
                y: 20.0 + hasard() * 640.0,
                size: taille,
                opacity: 0.06 + hasard() * 0.14,
                amp: 10.0 + hasard() * 10.0,
                period: (8000.0 + hasard() * 8000.0) as i64,
            }
        })
        .collect()
}

/// Démarrage (après l'affichage de la fenêtre) : migration, mises à jour, accueil.
pub async fn demarrer(app: Rc<App>) {
    // Ancien dossier C:\TEXT
    match app.bloquant(crate::services::update::migrer_ancien_dossier).await {
        Ok(true) => dialogs::info(&app, &loc::t("Demarrage_Migre"), &loc::t("Demarrage_Migre_Texte")).await,
        Ok(false) => {}
        Err(e) => {
            dialogs::erreur(&app, &loc::t("Demarrage_MigrationImpossible"), &loc::tf("Demarrage_MigrationImpossible_Texte", &[&e])).await
        }
    }

    crate::services::settings::initialiser();
    crate::services::pseudos::initialiser();
    news::charger(&app);

    verifier_mise_a_jour(&app, false).await;
    match app.bg(crate::services::update::finaliser()).await {
        Ok(true) => {
            dialogs::info(&app, &loc::t("Maj_Terminee"), &loc::tf("Maj_Terminee_Texte", &[&crate::services::app_info::VERSION])).await
        }
        Ok(false) => {}
        Err(e) => dialogs::erreur(&app, &loc::t("Maj_Nettoyage"), &loc::tf("Maj_Nettoyage_Texte", &[&e])).await,
    }

    home::initialiser(&app).await;
    app.ui.global::<AppState>().set_ready(true);
}

/// `manuel` : lancé depuis les options (on affiche aussi « à jour » et les erreurs).
async fn verifier_mise_a_jour(app: &Rc<App>, manuel: bool) {
    if manuel {
        app.ui.global::<SettingsState>().set_checking_update(true);
    }
    let resultat = app.bg(crate::services::update::verifier()).await;
    if manuel {
        app.ui.global::<SettingsState>().set_checking_update(false);
    }
    let release = match resultat {
        Ok(Some(r)) => r,
        Ok(None) => {
            if manuel {
                dialogs::info(app, &loc::t("Maj_AJour"), &loc::tf("Maj_AJour_Texte", &[&crate::services::app_info::VERSION])).await;
            }
            return;
        }
        Err(e) => {
            if manuel {
                dialogs::erreur(app, &loc::t("Maj_VerifImpossible"), &e.to_string()).await;
            }
            return;
        }
    };
    let installer = dialogs::confirmer(
        app,
        &loc::t("Maj_Disponible"),
        &loc::tf("Maj_Disponible_Texte", &[&release.version, &crate::services::app_info::VERSION]),
        Some(&loc::t("Maj_Installer")),
        Some(&loc::t("Commun_PlusTard")),
        false,
    )
    .await;
    if !installer {
        return;
    }
    match app.bg(async move { crate::services::update::installer(&release).await }).await {
        Ok(()) => {
            let _ = slint::quit_event_loop();
        }
        Err(e) => dialogs::erreur(app, &loc::t("Maj_Impossible"), &e.to_string()).await,
    }
}

// ───────────────────────── Navigation ─────────────────────────

fn nav_clicked(app: &Rc<App>, i: i32) {
    // Les options s’ouvrent dans une fenêtre par-dessus la page en cours
    if i == page::OPTIONS {
        settings::ouvrir(app);
        return;
    }

    let etat = app.ui.global::<AppState>();
    let page = etat.get_page();

    // Re-cliquer sur l'onglet actif depuis une sous-page (profil, éditeur) ramène à sa page principale
    if i == etat.get_nav() {
        if i == page::PROFILS && page == page::INSTANCE {
            profiles::rafraichir(app);
            naviguer(app, page::PROFILS);
        } else if i == page::PACKS && page == page::EDITEUR {
            naviguer(app, page::PACKS);
            packs::rafraichir(app);
        }
        return;
    }

    etat.set_nav(i);
    match i {
        page::ACCUEIL => {
            let recharger = settings::consommer_changement_types(app);
            home::rafraichir(app, recharger);
        }
        page::OPTIONS => settings::rafraichir(app),
        page::PROFILS => profiles::rafraichir(app),
        _ => {}
    }
    naviguer(app, i);

    if i == page::PACKS {
        packs::rafraichir(app); // après naviguer : l'éditeur a pu enregistrer une texture
        discord::definir(discord::Statut::PackEditeur);

    } else if !app.home.borrow().jeu_en_cours {
        discord::definir(discord::Statut::Launcher);
    }
}

/// Affiche une page (et la barre latérale correspondante).
pub fn naviguer(app: &Rc<App>, p: i32) {
    let etat = app.ui.global::<AppState>();
    if etat.get_page() == p {
        return;
    }
    // On quitte l'éditeur : la texture en cours ne doit pas être perdue
    if etat.get_page() == page::EDITEUR {
        editor::quitter(app);
    }
    etat.set_nav(match p {
        page::INSTANCE => page::PROFILS,
        page::EDITEUR => page::PACKS,
        autre => autre,
    });
    etat.set_page(p);
}

/// Langue changée : recalcule les textes posés depuis Rust.
pub fn langue_changee(app: &Rc<App>) {
    let tr = app.ui.global::<Tr>();
    tr.set_rev(tr.get_rev() + 1);
    discord::reappliquer();
    home::langue_changee(app);
    profiles::rafraichir_liste(app);
    instance::langue_changee(app);
    browser::langue_changee(app);
    settings::maj_textes(app);
    editor::langue_changee(app);
    skin::langue_changee(app);
}
