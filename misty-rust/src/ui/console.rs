//! Console des parties : relie ConsoleState (ui/console.slint) au journal (services::journal).
//! Un panneau par partie ; une minuterie ajoute les nouvelles lignes toutes les 150 ms.

use super::{dialogs, lancer, App};
use crate::services::{account, journal, loc, settings, system_info, telemetry};
use crate::{ConsoleState, LogLine, LogPane};
use slint::{ComponentHandle, Model, ModelRc, VecModel};
use std::cell::RefCell;
use std::rc::Rc;

/// Lignes affichées au plus par panneau (les plus anciennes sortent de l'écran, pas du fichier).
const AFFICHEES: usize = 10_000;

/// Ce que l'interface garde d'une partie, à côté de son journal.
struct Panneau {
    id: u64,
    /// Lignes du journal déjà lues (u64::MAX : tout réafficher)
    lues: u64,
    lignes: Rc<VecModel<LogLine>>,
    /// La partie tournait au dernier passage (pour remarquer qu'elle vient de planter)
    en_cours: bool,
    envoi: bool,
    rapport: String,
}

#[derive(Default)]
struct Etat {
    panneaux: Vec<Panneau>,
    modele: Option<Rc<VecModel<LogPane>>>,
}

thread_local! {
    static ETAT: RefCell<Etat> = RefCell::new(Etat::default());
}

fn c(app: &App) -> ConsoleState<'_> {
    app.ui.global::<ConsoleState>()
}

pub fn installer(app: &Rc<App>) {
    let modele = Rc::new(VecModel::<LogPane>::default());
    c(app).set_panes(ModelRc::from(modele.clone()));
    ETAT.with(|e| e.borrow_mut().modele = Some(modele));

    let a = app.clone();
    c(app).on_filter_changed(move || {
        ETAT.with(|e| e.borrow_mut().panneaux.iter_mut().for_each(|p| p.lues = u64::MAX));
        rafraichir(&a);
    });
    c(app).on_copy(|id| copier(&journal::texte(id as u64)));
    c(app).on_copy_lines(|id, debut, fin| {
        // les lignes telles qu'affichées (filtre et recherche compris)
        let texte = ETAT.with(|e| {
            let e = e.borrow();
            let pan = e.panneaux.iter().find(|p| p.id == id as u64)?;
            let n = pan.lignes.row_count();
            if n == 0 || debut < 0 {
                return None;
            }
            let (debut, fin) = (debut as usize, (fin.max(debut) as usize).min(n - 1));
            Some((debut..=fin).filter_map(|i| pan.lignes.row_data(i)).map(|l| l.text.to_string()).collect::<Vec<_>>().join("\r\n"))
        });
        if let Some(t) = texte.filter(|t| !t.is_empty()) {
            copier(&t);
        }
    });
    c(app).on_open_folder(|| {
        let d = journal::dossier();
        let _ = std::fs::create_dir_all(&d);
        system_info::ouvrir_dossier(&d);
    });
    c(app).on_kill(|id| {
        std::thread::spawn(move || journal::arreter(id as u64));
    });
    let a = app.clone();
    c(app).on_close_pane(move |id| {
        journal::fermer(id as u64);
        c(&a).set_focused(-1);
        rafraichir(&a);
    });
    let a = app.clone();
    c(app).on_send_report(move |id| {
        let a = a.clone();
        lancer(async move { envoyer_rapport(&a, id as u64).await });
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

/// Ouvre la console (au lancement d'une partie si l'option est active, ou par le bouton Logs).
/// Tous les panneaux sont montrés côte à côte.
pub fn ouvrir(app: &App) {
    c(app).set_focused(-1);
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

fn duree_texte(d: std::time::Duration) -> String {
    let s = d.as_secs();
    if s >= 3600 {
        format!("{}:{:02}:{:02}", s / 3600, (s / 60) % 60, s % 60)
    } else {
        format!("{:02}:{:02}", s / 60, s % 60)
    }
}

fn etat_texte(e: &journal::Etat) -> String {
    let duree = duree_texte(e.duree);
    if e.en_cours {
        loc::tf("Console_EnCours", &[&duree])
    } else if e.plante() {
        loc::tf("Console_Plante", &[&e.code.unwrap_or(0).to_string(), &duree])
    } else {
        loc::tf("Console_Termine", &[&duree])
    }
}

fn rafraichir(app: &App) {
    let st = c(app);
    let parties = journal::parties();
    let filtre = st.get_filter();
    let recherche = st.get_search().to_lowercase();
    let mut plantage = false;

    ETAT.with(|e| {
        let mut e = e.borrow_mut();
        let Some(modele) = e.modele.clone() else { return };

        // parties sorties de la console (fermées, ou remplacées par de nouvelles)
        let mut i = 0;
        while i < e.panneaux.len() {
            if parties.iter().any(|p| p.id == e.panneaux[i].id) {
                i += 1;
            } else {
                e.panneaux.remove(i);
                if i < modele.row_count() {
                    modele.remove(i);
                }
            }
        }
        // nouvelles parties
        for p in &parties {
            if !e.panneaux.iter().any(|x| x.id == p.id) {
                e.panneaux.push(Panneau {
                    id: p.id,
                    lues: u64::MAX,
                    lignes: Rc::new(VecModel::default()),
                    en_cours: p.en_cours,
                    envoi: false,
                    rapport: String::new(),
                });
            }
        }

        for (i, partie) in parties.iter().enumerate() {
            let Some(pan) = e.panneaux.iter_mut().find(|x| x.id == partie.id) else { continue };
            // lignes
            let tout = pan.lues == u64::MAX;
            if let Some((lignes, a_refaire)) = journal::nouvelles(pan.id, if tout { 0 } else { pan.lues }) {
                let gardees: Vec<LogLine> = lignes.into_iter().filter(|l| garde(filtre, &recherche, l)).map(vers_slint).collect();
                if tout || a_refaire {
                    let debut = gardees.len().saturating_sub(AFFICHEES);
                    pan.lignes.set_vec(gardees[debut..].to_vec());
                } else if !gardees.is_empty() {
                    for l in gardees {
                        pan.lignes.push(l);
                    }
                    if pan.lignes.row_count() > AFFICHEES + 1000 {
                        let garder: Vec<LogLine> = pan.lignes.iter().skip(pan.lignes.row_count() - AFFICHEES).collect();
                        pan.lignes.set_vec(garder);
                    }
                }
            }
            pan.lues = partie.total;
            // la partie vient de planter : la console s'ouvre pour montrer pourquoi
            let plante = partie.plante();
            if pan.en_cours && !partie.en_cours && plante {
                plantage = true;
            }
            pan.en_cours = partie.en_cours;

            let ligne = LogPane {
                id: pan.id as i32,
                title: partie.titre.clone().into(),
                state_text: etat_texte(partie).into(),
                running: partie.en_cours,
                crashed: plante,
                warnings: partie.avertissements as i32,
                errors: partie.erreurs as i32,
                report_text: pan.rapport.clone().into(),
                sending: pan.envoi,
                lines: ModelRc::from(pan.lignes.clone()),
            };
            if i < modele.row_count() {
                if modele.row_data(i).as_ref() != Some(&ligne) {
                    modele.set_row_data(i, ligne);
                }
            } else {
                modele.push(ligne);
            }
        }
    });

    // en-tête
    let en_cours = parties.iter().filter(|p| p.en_cours).count();
    let sous_titre = match (parties.len(), en_cours) {
        (0, _) => loc::t("Console_Aucune"),
        (_, 0) => loc::t("Console_AucuneEnCours"),
        (_, 1) => loc::t("Console_UneEnCours"),
        (_, n) => loc::tf("Console_PlusieursEnCours", &[&n.to_string()]),
    };
    st.set_subtitle(sous_titre.into());
    st.set_warnings(parties.iter().map(|p| p.avertissements).sum::<u64>() as i32);
    st.set_errors(parties.iter().map(|p| p.erreurs).sum::<u64>() as i32);
    if st.get_focused() >= parties.len() as i32 {
        st.set_focused(-1);
    }
    if plantage {
        st.set_open(true);
    }
}

fn maj_panneau(id: u64, f: impl FnOnce(&mut Panneau)) {
    ETAT.with(|e| {
        if let Some(p) = e.borrow_mut().panneaux.iter_mut().find(|p| p.id == id) {
            f(p);
        }
    });
}

async fn envoyer_rapport(app: &Rc<App>, id: u64) {
    maj_panneau(id, |p| {
        p.envoi = true;
        p.rapport.clear();
    });
    rafraichir(app);
    let lignes = journal::dernieres(id, 2000);
    let jeu = journal::etat(id).map(|e| e.titre).unwrap_or_default();
    let pseudo = account::session().map(|s| s.pseudo).unwrap_or_else(|| settings::get(settings::LAST_PSEUDO));
    let resultat = app.bg(async move { telemetry::rapport("manual", &jeu, lignes, &pseudo).await }).await;
    match resultat {
        Ok(code) => {
            copier(&code);
            maj_panneau(id, |p| {
                p.envoi = false;
                p.rapport = loc::tf("Console_Envoye", &[&code]);
            });
            rafraichir(app);
        }
        Err(e) => {
            maj_panneau(id, |p| p.envoi = false);
            rafraichir(app);
            dialogs::erreur(app, &loc::t("Console_EnvoiImpossible"), &format!("{e:#}")).await;
        }
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
