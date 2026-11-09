//! Console du jeu : relie ConsoleState (ui/console.slint) au journal de la partie
//! (services::journal). Une minuterie ajoute les nouvelles lignes toutes les 150 ms.

use super::{dialogs, lancer, App};
use crate::services::{journal, loc, settings, system_info, telemetry};
use crate::{ConsoleState, LogLine};
use slint::{ComponentHandle, Model, ModelRc, VecModel};
use std::cell::RefCell;
use std::rc::Rc;

/// Lignes affichées au plus (les plus anciennes sortent de l'écran, pas du fichier).
const AFFICHEES: usize = 10_000;

#[derive(Default)]
pub struct Etat {
    session: u64,
    /// Lignes de la partie déjà lues dans le journal
    lues: u64,
    modele: Option<Rc<VecModel<LogLine>>>,
    /// La partie était en cours au dernier passage (pour remarquer sa fin)
    en_cours: bool,
}

thread_local! {
    static ETAT: RefCell<Etat> = RefCell::new(Etat::default());
}

fn c(app: &App) -> ConsoleState<'_> {
    app.ui.global::<ConsoleState>()
}

pub fn installer(app: &Rc<App>) {
    let modele = Rc::new(VecModel::<LogLine>::default());
    c(app).set_lines(ModelRc::from(modele.clone()));
    ETAT.with(|e| e.borrow_mut().modele = Some(modele));

    let a = app.clone();
    c(app).on_filter_changed(move || tout_reafficher(&a));
    c(app).on_copy(|| copier(&journal::texte()));
    c(app).on_open_folder(|| {
        let d = journal::dossier();
        let _ = std::fs::create_dir_all(&d);
        system_info::ouvrir_dossier(&d);
    });
    c(app).on_kill(|| {
        std::thread::spawn(journal::arreter);
    });
    let a = app.clone();
    c(app).on_send_report(move || {
        let a = a.clone();
        lancer(async move { envoyer_rapport(&a).await });
    });

    let a = Rc::downgrade(app);
    let minuterie = slint::Timer::default();
    minuterie.start(slint::TimerMode::Repeated, std::time::Duration::from_millis(150), move || {
        if let Some(a) = a.upgrade() {
            rafraichir(&a);
        }
    });
    std::mem::forget(minuterie);
}

/// Ouvre la console (au lancement d'une partie si l'option est active, ou par le bouton).
pub fn ouvrir(app: &App) {
    c(app).set_report_text("".into());
    c(app).set_open(true);
}

/// La console s'ouvre-t-elle toute seule au lancement du jeu ?
pub fn auto() -> bool {
    settings::get_bool(settings::CONSOLE, true)
}

fn garde(filtre: i32, recherche: &str, l: &journal::Ligne) -> bool {
    let niveau_ok = match filtre {
        1 => l.niveau == journal::AVERT || l.niveau == journal::ERREUR,
        2 => l.niveau == journal::ERREUR,
        _ => true,
    };
    niveau_ok && (recherche.is_empty() || l.texte.to_lowercase().contains(recherche))
}

fn vers_slint(l: journal::Ligne) -> LogLine {
    // une tabulation s'affiche comme un carré : des espaces à la place
    LogLine { text: l.texte.replace('\t', "    ").into(), level: l.niveau }
}

/// Filtre ou recherche changés : on reprend tout depuis le journal.
fn tout_reafficher(app: &App) {
    ETAT.with(|e| e.borrow_mut().lues = u64::MAX);
    rafraichir(app);
}

fn rafraichir(app: &App) {
    let st = c(app);
    let etat = journal::etat();
    // nouvelles lignes
    let filtre = st.get_filter();
    let recherche = st.get_search().to_lowercase();
    ETAT.with(|e| {
        let mut e = e.borrow_mut();
        let Some(modele) = e.modele.clone() else { return };
        let (lignes, tout) = if e.lues == u64::MAX { (journal::nouvelles(u64::MAX, 0).0, true) } else { journal::nouvelles(e.session, e.lues) };
        if !tout && lignes.is_empty() {
            return;
        }
        let gardees: Vec<LogLine> = lignes.into_iter().filter(|l| garde(filtre, &recherche, l)).map(vers_slint).collect();
        if tout {
            let debut = gardees.len().saturating_sub(AFFICHEES);
            modele.set_vec(gardees[debut..].to_vec());
        } else {
            for l in gardees {
                modele.push(l);
            }
            if modele.row_count() > AFFICHEES + 1000 {
                let garder: Vec<LogLine> = modele.iter().skip(modele.row_count() - AFFICHEES).collect();
                modele.set_vec(garder);
            }
        }
        e.session = etat.session;
        e.lues = etat.total;
    });

    // en-tête
    st.set_title(etat.titre.clone().into());
    st.set_running(etat.en_cours);
    st.set_warnings(etat.avertissements as i32);
    st.set_errors(etat.erreurs as i32);
    let plante = matches!(etat.code, Some(n) if n != 0);
    st.set_crashed(plante);
    let duree = duree_texte(etat.duree);
    let texte = if etat.session == 0 {
        String::new()
    } else if etat.en_cours {
        loc::tf("Console_EnCours", &[&duree])
    } else if plante {
        loc::tf("Console_Plante", &[&etat.code.unwrap_or(0).to_string(), &duree])
    } else {
        loc::tf("Console_Termine", &[&duree])
    };
    st.set_state_text(texte.into());

    // la partie vient de planter : la console s'ouvre pour montrer pourquoi
    let fini = ETAT.with(|e| {
        let mut e = e.borrow_mut();
        let fini = e.en_cours && !etat.en_cours;
        e.en_cours = etat.en_cours;
        fini
    });
    if fini && plante {
        ouvrir(app);
    }
}

fn duree_texte(d: std::time::Duration) -> String {
    let s = d.as_secs();
    if s >= 3600 {
        format!("{}:{:02}:{:02}", s / 3600, (s / 60) % 60, s % 60)
    } else {
        format!("{:02}:{:02}", s / 60, s % 60)
    }
}

async fn envoyer_rapport(app: &Rc<App>) {
    let st = c(app);
    st.set_sending(true);
    st.set_report_text("".into());
    let lignes = journal::dernieres(2000);
    let pseudo = settings::get(settings::LAST_PSEUDO);
    let resultat = app.bg(async move { telemetry::rapport("manual", lignes, &pseudo).await }).await;
    st.set_sending(false);
    match resultat {
        Ok(code) => {
            copier(&code);
            st.set_report_text(loc::tf("Console_Envoye", &[&code]).into());
        }
        Err(e) => dialogs::erreur(app, &loc::t("Console_EnvoiImpossible"), &format!("{e:#}")).await,
    }
}

/// Copie un texte dans le presse-papiers (clip.exe lit l'UTF-16 avec BOM sans abîmer les accents).
fn copier(texte: &str) {
    let texte = texte.to_string();
    std::thread::spawn(move || {
        use std::io::Write;
        let mut cmd = std::process::Command::new("clip");
        cmd.stdin(std::process::Stdio::piped());
        crate::minecraft::sans_console(&mut cmd);
        let Ok(mut enfant) = cmd.spawn() else { return };
        if let Some(mut entree) = enfant.stdin.take() {
            let mut octets = vec![0xFF, 0xFE];
            for u in texte.encode_utf16() {
                octets.extend_from_slice(&u.to_le_bytes());
            }
            let _ = entree.write_all(&octets);
        }
        let _ = enfant.wait();
    });
}
