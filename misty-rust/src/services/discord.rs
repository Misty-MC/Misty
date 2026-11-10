//! Statut Discord (Rich Presence).
//!
//! Un seul client pour toute l'application (plusieurs clients avec le même ID se volent la connexion).
//! Il vit dans son propre thread : Discord peut être lent ou fermé, l'interface ne doit jamais attendre.
//!
//! Le statut affiche le logo de Misty avec la tête du joueur en petit, ce que fait le joueur dans le
//! launcher (page ouverte) ou en jeu (« Joue en tant que <pseudo> », version et profil), et un
//! bouton vers le site.

use super::{app_info, loc, settings};
use discord_rich_presence::{activity, DiscordIpc, DiscordIpcClient};
use std::sync::mpsc::{self, Sender};
use std::sync::{Mutex, OnceLock};

/// Logo de Misty (Discord accepte une adresse d'image pour le Rich Presence).
const LOGO: &str = "https://misty-launcher.online/assets/icone.png";
const SITE: &str = "https://misty-launcher.online";

/// Tête du joueur en petite image : Discord la télécharge lui-même depuis mc-heads.net (le service
/// de skins déjà utilisé par le site), par UUID pour un compte Microsoft, par pseudo sinon.
fn tete(id: &str) -> String {
    format!("https://mc-heads.net/avatar/{}/128", urlencoding::encode(id))
}

/// Page du launcher affichée dans le statut.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Page {
    Accueil,
    Skin,
    Profils,
    Notes,
    Packs,
    Editeur,
    Options,
    Amis,
}

impl Page {
    fn cle(self) -> &'static str {
        match self {
            Page::Accueil => "Discord_Accueil",
            Page::Skin => "Discord_Skin",
            Page::Profils => "Discord_Profils",
            Page::Notes => "Discord_Notes",
            Page::Packs => "Discord_Packs",
            Page::Editeur => "Discord_Pack",
            Page::Options => "Discord_Options",
            Page::Amis => "Discord_Amis",
        }
    }
}

/// Statut demandé (le texte est traduit au moment de l'envoi).
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Statut {
    Launcher(Page),
    EnJeu { version: String, mode: String, pseudo: String },
}

enum Message {
    Definir(Statut),
    /// Compte choisi dans le launcher : (pseudo, UUID si compte Microsoft)
    Joueur(String, Option<String>),
    Reappliquer,
    Arreter,
}

fn canal() -> &'static Mutex<Sender<Message>> {
    static CANAL: OnceLock<Mutex<Sender<Message>>> = OnceLock::new();
    CANAL.get_or_init(|| {
        let (tx, rx) = mpsc::channel::<Message>();
        std::thread::spawn(move || boucle(rx));
        Mutex::new(tx)
    })
}

/// Option "Rich Presence Discord" (activée par défaut).
pub fn active() -> bool {
    settings::get_bool(settings::RICH_PRESENCE, true)
}

pub fn definir(statut: Statut) {
    let _ = canal().lock().unwrap().send(Message::Definir(statut));
}

/// Compte choisi dans le launcher : sa tête est la petite image du statut.
pub fn definir_joueur(pseudo: &str, uuid: Option<&str>) {
    let _ = canal().lock().unwrap().send(Message::Joueur(pseudo.to_string(), uuid.map(str::to_string)));
}

/// À appeler quand l'option ou la langue change.
pub fn reappliquer() {
    let _ = canal().lock().unwrap().send(Message::Reappliquer);
}

pub fn arreter() {
    let _ = canal().lock().unwrap().send(Message::Arreter);
}

fn boucle(rx: mpsc::Receiver<Message>) {
    let debut = chrono::Utc::now().timestamp();
    let mut debut_jeu = debut;
    let mut client: Option<DiscordIpcClient> = None;
    let mut actuel: Option<Statut> = None;
    let mut joueur: Option<(String, Option<String>)> = None;

    while let Ok(message) = rx.recv() {
        match message {
            Message::Definir(s) => {
                // le chrono repart quand une partie commence
                if matches!(s, Statut::EnJeu { .. }) && !matches!(actuel, Some(Statut::EnJeu { .. })) {
                    debut_jeu = chrono::Utc::now().timestamp();
                }
                actuel = Some(s);
            }
            Message::Joueur(pseudo, uuid) => {
                let nouveau = Some((pseudo, uuid));
                if joueur == nouveau {
                    continue;
                }
                joueur = nouveau;
            }
            Message::Reappliquer => {}
            Message::Arreter => {
                if let Some(c) = client.as_mut() {
                    let _ = c.close();
                }
                return;
            }
        }

        // Connexion à la demande (Discord a pu être lancé après Misty)
        if client.is_none() {
            if let Ok(mut c) = DiscordIpcClient::new(app_info::DISCORD_APP_ID) {
                if c.connect().is_ok() {
                    client = Some(c);
                }
            }
        }
        let Some(c) = client.as_mut() else { continue };

        let resultat = match (&actuel, active()) {
            (Some(statut), true) => {
                let (details, state, depuis) = match statut {
                    Statut::Launcher(page) => (loc::t("Discord_Launcher"), loc::t(page.cle()), debut),
                    Statut::EnJeu { version, mode, pseudo } => (
                        loc::tf("Discord_JoueEnTantQue", &[pseudo]),
                        format!("Minecraft {version} · {mode}"),
                        debut_jeu,
                    ),
                };
                // la tête : celle du compte en jeu (une partie peut avoir été lancée avec un autre
                // compte que celui choisi maintenant), sinon celle du compte du launcher
                let compte = match (statut, &joueur) {
                    (Statut::EnJeu { pseudo, .. }, Some((p, uuid))) if p == pseudo => Some((p.clone(), uuid.clone())),
                    (Statut::EnJeu { pseudo, .. }, _) => Some((pseudo.clone(), None)),
                    (Statut::Launcher(_), j) => j.clone(),
                };
                let petite = compte.as_ref().map(|(pseudo, uuid)| (tete(uuid.as_deref().unwrap_or(pseudo)), pseudo.clone()));
                let texte_logo = format!("Misty {}", app_info::VERSION);
                let bouton = loc::t("Discord_Bouton");
                let mut images = activity::Assets::new().large_image(LOGO).large_text(&texte_logo);
                if let Some((url, pseudo)) = &petite {
                    images = images.small_image(url).small_text(pseudo);
                }
                let a = activity::Activity::new()
                    .details(&details)
                    .state(&state)
                    .timestamps(activity::Timestamps::new().start(depuis))
                    .assets(images)
                    .buttons(vec![activity::Button::new(&bouton, SITE)]);
                c.set_activity(a)
            }
            _ => c.clear_activity(),
        };
        // Discord fermé entre-temps : on se reconnectera au prochain message
        if resultat.is_err() {
            client = None;
        }
    }
}
