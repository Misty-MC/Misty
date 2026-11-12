//! Fenêtre de connexion Microsoft (mode `Misty.exe --ms-login`).
//!
//! Affiche la page de connexion de Microsoft dans une WebView (WebView2 sous Windows).
//! Quand Microsoft redirige vers l'URL de fin, on lit le code dans l'adresse,
//! on l'écrit sur la sortie standard ("CODE:...") et on ferme la fenêtre.
//! Le processus principal lit cette sortie (voir services::account).

use crate::services::{account, loc};
use std::io::Write;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use winit::application::ApplicationHandler;
use winit::dpi::LogicalSize;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop, EventLoopProxy};
use winit::window::{Window, WindowId};
use wry::{WebContext, WebView, WebViewBuilder};

enum Fin {
    Code(String),
    Erreur(String),
}

struct Connexion {
    proxy: EventLoopProxy<Fin>,
    contexte: WebContext,
    // La WebView doit être libérée avant sa fenêtre : ordre des champs = ordre de libération.
    webview: Option<WebView>,
    fenetre: Option<Window>,
}

impl ApplicationHandler<Fin> for Connexion {
    fn resumed(&mut self, boucle: &ActiveEventLoop) {
        if self.fenetre.is_some() {
            return;
        }
        let fenetre = boucle
            .create_window(
                Window::default_attributes()
                    .with_title(format!("Misty Launcher — {}", loc::t("Compte_SeConnecter")))
                    .with_inner_size(LogicalSize::new(520.0, 680.0)),
            )
            .expect("fenêtre de connexion");

        let deja_fini = Arc::new(AtomicBool::new(false));
        let proxy = self.proxy.clone();
        let navigation = move |url: String| -> bool {
            if !url.starts_with(account::REDIRECT) {
                return true; // on laisse la page de Microsoft naviguer
            }
            if !deja_fini.swap(true, Ordering::SeqCst) {
                let fin = match parametre(&url, "code") {
                    Some(code) => Fin::Code(code),
                    None => Fin::Erreur(parametre(&url, "error_description").unwrap_or_else(|| loc::t("Compte_Annule"))),
                };
                let _ = proxy.send_event(fin);
            }
            false
        };

        let webview = WebViewBuilder::new_with_web_context(&mut self.contexte)
            .with_url(account::url_connexion())
            .with_navigation_handler(navigation)
            .build(&fenetre)
            .expect("WebView (WebView2 est-il installé ?)");

        self.webview = Some(webview);
        self.fenetre = Some(fenetre);
    }

    fn user_event(&mut self, boucle: &ActiveEventLoop, fin: Fin) {
        let ligne = match fin {
            Fin::Code(c) => format!("CODE:{c}"),
            Fin::Erreur(e) => format!("ERREUR:{e}"),
        };
        let mut sortie = std::io::stdout();
        let _ = writeln!(sortie, "{ligne}");
        let _ = sortie.flush();
        boucle.exit();
    }

    fn window_event(&mut self, boucle: &ActiveEventLoop, _: WindowId, evenement: WindowEvent) {
        if let WindowEvent::CloseRequested = evenement {
            boucle.exit();
        }
    }
}

pub fn executer() {
    loc::initialiser();

    let boucle = EventLoop::<Fin>::with_user_event().build().expect("boucle d'événements");
    boucle.set_control_flow(ControlFlow::Wait);

    // Cookies stockés dans le dossier de Misty (pas à côté de l'exe)
    let _ = std::fs::create_dir_all(account::dossier_webview());
    let mut app = Connexion {
        proxy: boucle.create_proxy(),
        contexte: WebContext::new(Some(account::dossier_webview())),
        webview: None,
        fenetre: None,
    };
    let _ = boucle.run_app(&mut app);
}

/// Lit un paramètre de l'URL (après "?" ou "#").
fn parametre(url: &str, nom: &str) -> Option<String> {
    let requete = url.split_once(['?', '#'])?.1;
    requete.split(['&', '#']).find_map(|p| {
        let (k, v) = p.split_once('=')?;
        (k == nom).then(|| urlencoding::decode(&v.replace('+', " ")).map(|s| s.into_owned()).unwrap_or_default())
    })
}
