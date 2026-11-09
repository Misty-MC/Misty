//! Misty — launcher Minecraft, version Rust.
//!
//! - `services/`  : la logique (réglages, profils, Modrinth, CurseForge, modpacks, packs de textures…)
//! - `minecraft/` : installation et lancement du jeu, mod loaders
//! - `ui/`        : le lien entre l'interface Slint (dossier `ui/` à la racine) et les services

// Pas de console noire derrière la fenêtre (en release)
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod minecraft;
mod ms_login;
mod services;
#[cfg(test)]
mod tests;
mod ui;

use services::{discord, loc};
use slint::ComponentHandle;
use std::cell::RefCell;
use std::rc::Rc;

slint::include_modules!();

thread_local! {
    /// L'application, accessible depuis les rappels qui reviennent sur le thread de l'interface.
    pub static APP: RefCell<Option<Rc<ui::App>>> = const { RefCell::new(None) };
}

fn main() {
    // Mode « fenêtre de connexion Microsoft » (lancé par le launcher lui-même)
    if std::env::args().any(|a| a == "--ms-login") {
        ms_login::executer();
        return;
    }

    // Runtime pour le réseau et les fichiers (le thread principal reste à l'interface)
    let runtime = tokio::runtime::Builder::new_multi_thread().enable_all().build().expect("runtime tokio");

    let _ = std::fs::create_dir_all(services::paths::data_dir());
    services::journal::capturer_paniques();
    services::journal::info(&format!("Démarrage de Misty {} ({})", services::app_info::VERSION, services::system_info::version_windows()));
    services::telemetry::init(runtime.handle().clone());
    loc::initialiser();
    discord::definir(discord::Statut::Launcher(discord::Page::Accueil));

    let fenetre = MainWindow::new().expect("création de la fenêtre");
    let app = ui::App::new(fenetre, runtime.handle().clone());
    APP.with(|a| *a.borrow_mut() = Some(app.clone()));
    ui::installer(&app);

    app.ui.show().expect("affichage de la fenêtre");
    preparer_fenetre(&app);

    let a = app.clone();
    ui::lancer(async move { ui::demarrer(a).await });

    // Outil de développement : MISTY_CAPTURE=<dossier> enregistre une image de chaque page puis quitte
    #[cfg(debug_assertions)]
    if let Ok(dossier) = std::env::var("MISTY_CAPTURE") {
        capture(&app, std::path::PathBuf::from(dossier));
    }

    slint::run_event_loop().expect("boucle de l'interface");
    services::journal::info("Fermeture de Misty");

    discord::arreter();
    APP.with(|a| *a.borrow_mut() = None);
    runtime.shutdown_timeout(std::time::Duration::from_secs(1));
}

/// Capture de chaque page (fenêtre placée hors de l'écran).
#[cfg(debug_assertions)]
fn capture(app: &Rc<ui::App>, dossier: std::path::PathBuf) {
    use slint::winit_030::WinitWindowAccessor;
    app.ui.window().with_winit_window(|w| w.set_outer_position(slint::winit_030::winit::dpi::PhysicalPosition::new(-6000, -6000)));
    let _ = std::fs::create_dir_all(&dossier);

    async fn attendre(ms: u64) {
        let (tx, rx) = futures::channel::oneshot::channel::<()>();
        slint::Timer::single_shot(std::time::Duration::from_millis(ms), move || {
            let _ = tx.send(());
        });
        let _ = rx.await;
    }
    let photo = |app: &ui::App, nom: &str, dossier: &std::path::Path| {
        if let Ok(b) = app.ui.window().take_snapshot() {
            if let Some(img) = image::RgbaImage::from_raw(b.width(), b.height(), b.as_bytes().to_vec()) {
                let _ = img.save(dossier.join(format!("{nom}.png")));
            }
        }
    };

    let app = app.clone();
    ui::lancer(async move {
        attendre(6000).await;
        photo(&app, "0-accueil", &dossier);
        // MISTY_CAPTURE_RAPIDE : seulement le choix de version en fenêtre minimale et la console
        if std::env::var("MISTY_CAPTURE_RAPIDE").is_ok() {
            if app.ui.global::<DialogState>().get_visible() {
                app.ui.global::<DialogState>().invoke_cancel();
            }
            app.ui.window().set_size(slint::LogicalSize::new(1040.0, 680.0));
            attendre(1000).await;
            let accueil = app.ui.global::<HomeState>();
            accueil.set_picker_tab(0);
            accueil.set_picker_open(true);
            attendre(2500).await;
            photo(&app, "r1-versions-etroit", &dossier);
            accueil.set_picker_open(false);
            use services::journal;
            journal::nouvelle_partie("Minecraft 1.21.4 · fabric");
            for (t, n) in [
                ("[10:42:01] [main/INFO]: Loading Minecraft 1.21.4 with Fabric Loader 0.16.9", 0),
                ("[10:42:02] [main/WARN]: Mod 'sodium' uses a deprecated API", 1),
                ("[10:42:05] [Render thread/INFO]: Backend library: LWJGL version 3.3.3", 0),
                ("[10:42:07] [Render thread/ERROR]: Failed to load texture minecraft:textures/x.png", 2),
                ("java.io.FileNotFoundException: minecraft:textures/x.png", 2),
                ("	at net.minecraft.class_1060.method_4625(class_1060.java:85)", 2),
                ("[10:42:09] [Render thread/INFO]: Sound engine started", 0),
            ] {
                journal::ajouter(t, n);
            }
            ui::console::ouvrir(&app);
            attendre(1200).await;
            photo(&app, "r2-console", &dossier);
            app.ui.global::<Theme>().set_mode(1);
            attendre(800).await;
            photo(&app, "r2b-console-sombre", &dossier);
            app.ui.global::<Theme>().set_mode(0);
            journal::fin(Some(1));
            attendre(800).await;
            photo(&app, "r3-console-plante", &dossier);
            app.ui.global::<ConsoleState>().set_filter(2);
            app.ui.global::<ConsoleState>().invoke_filter_changed();
            attendre(600).await;
            photo(&app, "r4-console-erreurs", &dossier);
            let _ = slint::quit_event_loop();
            return;
        }
        if std::env::var("MISTY_CAPTURE_SOMBRE").is_ok() {
            app.ui.global::<Theme>().set_mode(1);
        }
        app.ui.global::<SkinState>().set_tab(1);
        attendre(1500).await;
        photo(&app, "0b-skin", &dossier);
        // Studio : la cape Misty essayée
        app.ui.global::<SkinState>().set_misty_equip(1);
        app.ui.global::<SkinState>().invoke_look_changed();
        attendre(3000).await;
        photo(&app, "0c-skin-misty", &dossier);
        // Studio en direct : vol (élytres) et accroupi
        for (anim, nom) in [(3, "0d-skin-vol"), (4, "0f-skin-accroupi")] {
            app.ui.global::<SkinState>().set_anim(anim);
            if anim == 3 {
                app.ui.global::<SkinState>().set_elytra(true);
                app.ui.global::<SkinState>().invoke_elytra_toggled();
            }
            attendre(2500).await;
            photo(&app, nom, &dossier);
        }
        // Cadrages de la caméra par catégorie (capes, chapeaux, compagnons, tags)
        {
            let st = app.ui.global::<SkinState>();
            st.set_anim(0);
            st.set_elytra(false);
            st.invoke_elytra_toggled();
            for (cat, nom) in [(0, "0j-cadre-capes"), (2, "0k-cadre-chapeaux"), (5, "0l-cadre-compagnons"), (7, "0m-cadre-tags")] {
                st.set_category(cat);
                st.set_focus(true);
                st.set_focus_count(st.get_focus_count() + 1);
                attendre(1800).await;
                photo(&app, nom, &dossier);
            }
            st.set_focus(false);
        }
        // Fenêtre de choix de la version, puis des profils
        let accueil = app.ui.global::<HomeState>();
        accueil.set_picker_tab(0);
        accueil.set_picker_open(true);
        attendre(1500).await;
        photo(&app, "0g-versions", &dossier);
        // filtre Fabric : seules les versions compatibles restent
        accueil.set_picker_loader(1);
        accueil.invoke_picker_changed();
        attendre(900).await;
        photo(&app, "0g2-versions-fabric", &dossier);
        accueil.set_picker_loader(0);
        accueil.invoke_picker_changed();
        accueil.set_picker_tab(1);
        attendre(900).await;
        photo(&app, "0h-profils", &dossier);
        // Horizon : son panneau (état lu sur le PC, sans rien lancer)
        accueil.set_picker_tab(0);
        accueil.set_picker_horizon(true);
        accueil.invoke_horizon_refresh();
        attendre(4000).await;
        photo(&app, "0i-horizon", &dossier);
        accueil.set_picker_horizon(false);
        accueil.set_picker_open(false);
        app.ui.global::<SkinState>().set_anim(0);
        app.ui.global::<SkinState>().set_elytra(false);
        app.ui.global::<SkinState>().invoke_elytra_toggled();
        app.ui.global::<SkinState>().set_misty_equip(0);
        app.ui.global::<SkinState>().invoke_look_changed();
        app.ui.global::<SkinState>().set_tab(0);
        let etat = app.ui.global::<AppState>();
        // Fenêtre des options : chaque catégorie, en thème clair puis sombre (sans rien enregistrer)
        let theme = app.ui.global::<Theme>();
        let mode_avant = theme.get_mode();
        for (mode, suffixe) in [(0, "clair"), (1, "sombre")] {
            theme.set_mode(mode);
            ui::settings::ouvrir(&app);
            for section in 0..8 {
                app.ui.global::<SettingsState>().set_section(section);
                attendre(900).await;
                photo(&app, &format!("2-options-{section}-{suffixe}"), &dossier);
            }
            ui::settings::fermer(&app);
            attendre(600).await;
            photo(&app, &format!("0-accueil-{suffixe}"), &dossier);
        }
        // MISTY_CAPTURE_SOMBRE : les autres pages aussi en thème sombre
        theme.set_mode(if std::env::var("MISTY_CAPTURE_SOMBRE").is_ok() { 1 } else { mode_avant });
        for (page, nom) in [(1, "1-profils"), (3, "3-notes"), (4, "4-packs"), (8, "7-amis")] {
            match page {
                1 => ui::profiles::rafraichir(&app),
                4 => ui::packs::rafraichir(&app),
                _ => {}
            }
            etat.set_nav(page);
            etat.set_page(page);
            attendre(2500).await;
            photo(&app, nom, &dossier);
        }
        if let Some(i) = services::instances::lister().into_iter().next() {
            ui::instance::ouvrir(&app, i);
            attendre(4000).await;
            photo(&app, "5-instance", &dossier);
            app.ui.global::<InstanceState>().set_tab(1);
            app.ui.global::<InstanceState>().invoke_tab_changed(1);
            attendre(5000).await;
            photo(&app, "6-decouvrir", &dossier);
        }
        // Données de démonstration (MISTY_DATA_DIR) : pas d'éditeur, il téléchargerait le jeu pour ses textures
        let demo = std::env::var_os("MISTY_DATA_DIR").is_some();
        if let Some(p) = services::packs::lister().into_iter().find(|p| !demo && !p.version_mc.is_empty()) {
            ui::editor::ouvrir(&app, p);
            attendre(5000).await;
            app.ui.global::<EditorState>().invoke_select(0);
            attendre(1500).await;
            photo(&app, "7-editeur", &dossier);
        }
        // Fenêtre agrandie : la mise en page doit suivre
        app.ui.global::<AppState>().set_page(0);
        app.ui.global::<AppState>().set_nav(0);
        app.ui.window().set_size(slint::LogicalSize::new(1600.0, 960.0));
        attendre(1500).await;
        photo(&app, "8-grand-accueil", &dossier);
        ui::settings::ouvrir(&app);
        attendre(900).await;
        photo(&app, "8-grand-options", &dossier);
        ui::settings::fermer(&app);
        let _ = slint::quit_event_loop();
    });
}

/// Centre la fenêtre et demande à Windows 11 des coins arrondis (la fenêtre n'a pas de bordure).
fn preparer_fenetre(app: &ui::App) {
    use slint::winit_030::WinitWindowAccessor;
    app.ui.window().with_winit_window(|w| {
        if let Some(ecran) = w.current_monitor() {
            let (taille, pos) = (ecran.size(), ecran.position());
            let fenetre = w.outer_size();
            let x = pos.x + (taille.width as i32 - fenetre.width as i32) / 2;
            let y = pos.y + (taille.height as i32 - fenetre.height as i32) / 2;
            w.set_outer_position(slint::winit_030::winit::dpi::PhysicalPosition::new(x, y));
        }
        #[cfg(windows)]
        coins_arrondis(w);
    });
}

#[cfg(windows)]
fn coins_arrondis(w: &slint::winit_030::winit::window::Window) {
    use slint::winit_030::winit::raw_window_handle::{HasWindowHandle, RawWindowHandle};
    use windows_sys::Win32::Graphics::Dwm::{DwmSetWindowAttribute, DWMWA_WINDOW_CORNER_PREFERENCE, DWMWCP_ROUND};
    let Ok(handle) = w.window_handle() else { return };
    if let RawWindowHandle::Win32(h) = handle.as_raw() {
        let preference = DWMWCP_ROUND;
        // SAFETY : HWND valide (fenêtre affichée) et pointeur vers une valeur de 4 octets, comme l'exige l'API.
        unsafe {
            DwmSetWindowAttribute(
                h.hwnd.get() as _,
                DWMWA_WINDOW_CORNER_PREFERENCE as u32,
                &preference as *const _ as *const core::ffi::c_void,
                std::mem::size_of_val(&preference) as u32,
            );
        }
    }
}
