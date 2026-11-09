//! Journaux : celui du launcher (`logs/misty/launcher.log`) et la console du jeu en cours.
//!
//! La console garde les dernières lignes de la partie en mémoire (affichées par la fenêtre
//! Console) et les écrit aussi dans `logs/misty/game-<date>.log` (les 10 dernières parties).
//! Minecraft écrit ses journaux au format XML de log4j (argument `logging` de la version) :
//! chaque événement est remis en une ligne lisible « [12:34:56] [Render thread/INFO]: … ».

use super::paths;
use std::collections::VecDeque;
use std::fs;
use std::io::{BufRead, BufReader, Read, Write};
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::{Duration, Instant};

/// Niveau d'une ligne : 0 info, 1 avertissement, 2 erreur, 3 message de Misty.
pub const INFO: i32 = 0;
pub const AVERT: i32 = 1;
pub const ERREUR: i32 = 2;
pub const MISTY: i32 = 3;

/// Lignes gardées en mémoire pour la console.
const MAX_LIGNES: usize = 10_000;
/// Parties dont le journal est gardé sur le disque.
const MAX_FICHIERS: usize = 10;

#[derive(Clone, Debug)]
pub struct Ligne {
    pub texte: String,
    pub niveau: i32,
}

/// État de la partie suivie par la console.
#[derive(Clone, Default)]
pub struct Etat {
    /// Lignes reçues depuis le début de la partie (sert à savoir ce qui est nouveau)
    pub total: u64,
    /// Numéro de la partie (change à chaque lancement)
    pub session: u64,
    pub titre: String,
    pub en_cours: bool,
    /// Code de sortie, une fois la partie finie
    pub code: Option<i32>,
    pub duree: Duration,
    pub avertissements: u64,
    pub erreurs: u64,
}

struct Console {
    lignes: VecDeque<Ligne>,
    etat: Etat,
    debut: Option<Instant>,
    fichier: Option<fs::File>,
    pid: Option<u32>,
}

static CONSOLE: Mutex<Console> = Mutex::new(Console {
    lignes: VecDeque::new(),
    etat: Etat {
        total: 0,
        session: 0,
        titre: String::new(),
        en_cours: false,
        code: None,
        duree: Duration::ZERO,
        avertissements: 0,
        erreurs: 0,
    },
    debut: None,
    fichier: None,
    pid: None,
});

/// Dossier des journaux de Misty (à côté de ceux de Minecraft, sans s'y mêler).
pub fn dossier() -> PathBuf {
    paths::data_dir().join("logs").join("misty")
}

fn heure() -> String {
    chrono::Local::now().format("%H:%M:%S").to_string()
}

// ───────────────────────── Journal du launcher ─────────────────────────

/// Ajoute une ligne au journal du launcher (gardé sous 2 Mo : l'ancien devient launcher.old.log).
pub fn launcher(niveau: i32, message: &str) {
    static VERROU: Mutex<()> = Mutex::new(());
    let _v = VERROU.lock();
    let d = dossier();
    let _ = fs::create_dir_all(&d);
    let f = d.join("launcher.log");
    if fs::metadata(&f).is_ok_and(|m| m.len() > 2 * 1024 * 1024) {
        let _ = fs::rename(&f, d.join("launcher.old.log"));
    }
    let etiquette = match niveau {
        AVERT => "WARN",
        ERREUR => "ERROR",
        _ => "INFO",
    };
    let date = chrono::Local::now().format("%Y-%m-%d %H:%M:%S");
    if let Ok(mut sortie) = fs::OpenOptions::new().create(true).append(true).open(&f) {
        for ligne in message.lines() {
            let _ = writeln!(sortie, "[{date}] [{etiquette}] {ligne}");
        }
    }
}

pub fn info(message: &str) {
    launcher(INFO, message);
}

pub fn avert(message: &str) {
    launcher(AVERT, message);
}

pub fn erreur(message: &str) {
    launcher(ERREUR, message);
}

/// Les plantages du launcher finissent dans son journal (avec l'endroit du code).
pub fn capturer_paniques() {
    let defaut = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        launcher(ERREUR, &format!("PANIC : {info}"));
        defaut(info);
    }));
}

// ───────────────────────── Console du jeu ─────────────────────────

/// Commence une nouvelle partie : vide la console et ouvre son fichier.
pub fn nouvelle_partie(titre: &str) {
    let d = dossier();
    let _ = fs::create_dir_all(&d);
    menage(&d);
    let nom = format!("game-{}.log", chrono::Local::now().format("%Y-%m-%d_%H-%M-%S"));
    let fichier = fs::File::create(d.join(nom)).ok();
    {
        let mut c = CONSOLE.lock().unwrap();
        let session = c.etat.session + 1;
        c.lignes.clear();
        c.etat = Etat { session, titre: titre.to_string(), en_cours: true, ..Etat::default() };
        c.debut = Some(Instant::now());
        c.fichier = fichier;
        c.pid = None;
    }
    info(&format!("Lancement : {titre}"));
    ajouter(&format!("[{}] [Misty] {}", heure(), super::loc::tf("Console_Lancement", &[&titre])), MISTY);
}

/// Garde les MAX_FICHIERS journaux de partie les plus récents.
fn menage(d: &std::path::Path) {
    let Ok(lecture) = fs::read_dir(d) else { return };
    let mut parties: Vec<PathBuf> = lecture
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.file_name().is_some_and(|n| n.to_string_lossy().starts_with("game-")))
        .collect();
    parties.sort();
    let trop = parties.len().saturating_sub(MAX_FICHIERS - 1);
    for p in parties.into_iter().take(trop) {
        let _ = fs::remove_file(p);
    }
}

/// Ajoute une ligne à la console (et au fichier de la partie).
pub fn ajouter(texte: &str, niveau: i32) {
    let mut c = CONSOLE.lock().unwrap();
    if let Some(f) = c.fichier.as_mut() {
        let _ = writeln!(f, "{texte}");
    }
    c.etat.total += 1;
    match niveau {
        AVERT => c.etat.avertissements += 1,
        ERREUR => c.etat.erreurs += 1,
        _ => {}
    }
    if c.lignes.len() >= MAX_LIGNES {
        c.lignes.pop_front();
    }
    c.lignes.push_back(Ligne { texte: texte.to_string(), niveau });
}

/// Copie brute de la sortie d'erreur (pour résumer une erreur au joueur).
pub type Copie = std::sync::Arc<Mutex<String>>;

/// Suit la sortie du jeu (stdout et stderr), chacune sur son fil. `copie` reçoit aussi la
/// sortie d'erreur telle quelle. Renvoie les fils, à attendre après la fin du processus.
pub fn suivre(enfant: &mut std::process::Child, copie: Option<Copie>) -> Vec<std::thread::JoinHandle<()>> {
    CONSOLE.lock().unwrap().pid = Some(enfant.id());
    let mut fils = Vec::new();
    if let Some(sortie) = enfant.stdout.take() {
        fils.push(lire_flux(sortie, false, None));
    }
    if let Some(erreurs) = enfant.stderr.take() {
        fils.push(lire_flux(erreurs, true, copie));
    }
    fils
}

/// Lit un flux ligne par ligne dans la console, sur un fil à part.
fn lire_flux(flux: impl Read + Send + 'static, erreurs: bool, copie: Option<Copie>) -> std::thread::JoinHandle<()> {
    std::thread::spawn(move || {
        let mut lecteur = BufReader::new(flux);
        let mut analyseur = Analyseur::default();
        let mut octets = Vec::new();
        loop {
            octets.clear();
            match lecteur.read_until(b'\n', &mut octets) {
                Ok(0) | Err(_) => break,
                Ok(_) => {
                    let ligne = String::from_utf8_lossy(&octets);
                    if let Some(c) = &copie {
                        if let Ok(mut c) = c.lock() {
                            c.push_str(&ligne);
                            if c.len() > 64 * 1024 {
                                let mut coupe = c.len() - 32 * 1024;
                                while !c.is_char_boundary(coupe) {
                                    coupe += 1;
                                }
                                c.drain(..coupe);
                            }
                        }
                    }
                    for (texte, niveau) in analyseur.ligne(ligne.trim_end_matches(['\r', '\n']), erreurs) {
                        ajouter(&texte, niveau);
                    }
                }
            }
        }
    })
}

/// Fin de la partie.
pub fn fin(code: Option<i32>) {
    let titre;
    {
        let mut c = CONSOLE.lock().unwrap();
        c.etat.en_cours = false;
        c.etat.code = code;
        c.etat.duree = c.debut.map(|d| d.elapsed()).unwrap_or_default();
        c.pid = None;
        titre = c.etat.titre.clone();
    }
    let texte = match code {
        Some(0) | None => super::loc::t("Console_Ferme"),
        Some(n) => super::loc::tf("Console_Plantage", &[&n.to_string()]),
    };
    ajouter(&format!("[{}] [Misty] {texte}", heure()), if matches!(code, Some(n) if n != 0) { ERREUR } else { MISTY });
    let mut c = CONSOLE.lock().unwrap();
    c.fichier = None;
    drop(c);
    info(&format!("Fin : {titre} (code {})", code.map(|n| n.to_string()).unwrap_or_else(|| "?".into())));
}

/// État de la partie (durée à jour si elle tourne).
pub fn etat() -> Etat {
    let c = CONSOLE.lock().unwrap();
    let mut e = c.etat.clone();
    if e.en_cours {
        e.duree = c.debut.map(|d| d.elapsed()).unwrap_or_default();
    }
    e
}

/// Lignes arrivées après les `deja` premières de la partie `session`. Renvoie aussi si la console
/// doit tout réafficher (nouvelle partie, ou lignes déjà sorties de la mémoire).
pub fn nouvelles(session: u64, deja: u64) -> (Vec<Ligne>, bool) {
    let c = CONSOLE.lock().unwrap();
    let premier = c.etat.total - c.lignes.len() as u64;
    if session != c.etat.session || deja < premier || deja > c.etat.total {
        return (c.lignes.iter().cloned().collect(), true);
    }
    (c.lignes.iter().skip((deja - premier) as usize).cloned().collect(), false)
}

/// Les `n` dernières lignes (pour un rapport).
pub fn dernieres(n: usize) -> Vec<String> {
    let c = CONSOLE.lock().unwrap();
    c.lignes.iter().skip(c.lignes.len().saturating_sub(n)).map(|l| l.texte.clone()).collect()
}

/// Toute la console, en texte.
pub fn texte() -> String {
    dernieres(MAX_LIGNES).join("\n")
}

/// Arrête le jeu de force (et ses sous-processus).
pub fn arreter() {
    let pid = CONSOLE.lock().unwrap().pid;
    let Some(pid) = pid else { return };
    avert(&format!("Arrêt forcé du jeu (PID {pid})"));
    #[cfg(windows)]
    {
        let mut c = std::process::Command::new("taskkill");
        c.args(["/PID", &pid.to_string(), "/T", "/F"]);
        crate::minecraft::sans_console(&mut c);
        let _ = c.status();
    }
    #[cfg(not(windows))]
    {
        let _ = std::process::Command::new("kill").args(["-9", &pid.to_string()]).status();
    }
}

// ───────────────────────── Lecture des journaux ─────────────────────────

/// Remet les événements XML de log4j en lignes lisibles ; le texte simple passe tel quel.
#[derive(Default)]
struct Analyseur {
    /// Événement XML en cours de lecture (sur plusieurs lignes)
    xml: Option<String>,
    /// Niveau de la dernière ligne (une trace de pile garde le niveau de son message)
    dernier: i32,
}

impl Analyseur {
    fn ligne(&mut self, ligne: &str, erreurs: bool) -> Vec<(String, i32)> {
        if let Some(xml) = self.xml.as_mut() {
            xml.push('\n');
            xml.push_str(ligne);
            if ligne.contains("</log4j:Event>") {
                let xml = self.xml.take().unwrap_or_default();
                return self.evenement(&xml);
            }
            return Vec::new();
        }
        let debut = ligne.trim_start();
        if debut.starts_with("<log4j:Event") {
            if debut.contains("</log4j:Event>") {
                return self.evenement(debut);
            }
            self.xml = Some(debut.to_string());
            return Vec::new();
        }
        if ligne.trim().is_empty() {
            return Vec::new();
        }
        let niveau = niveau_texte(ligne, erreurs, self.dernier);
        self.dernier = niveau;
        vec![(ligne.to_string(), niveau)]
    }

    fn evenement(&mut self, xml: &str) -> Vec<(String, i32)> {
        let attribut = |nom: &str| -> String {
            let motif = format!("{nom}=\"");
            xml.find(&motif)
                .and_then(|i| {
                    let reste = &xml[i + motif.len()..];
                    reste.find('"').map(|j| deseschapper(&reste[..j]))
                })
                .unwrap_or_default()
        };
        let niveau_txt = attribut("level");
        let fil = attribut("thread");
        let moment = attribut("timestamp")
            .parse::<i64>()
            .ok()
            .and_then(chrono::DateTime::from_timestamp_millis)
            .map(|d| d.with_timezone(&chrono::Local).format("%H:%M:%S").to_string())
            .unwrap_or_else(heure);
        let niveau = match niveau_txt.as_str() {
            "WARN" => AVERT,
            "ERROR" | "FATAL" => ERREUR,
            _ => INFO,
        };
        self.dernier = niveau;
        let mut sortie = Vec::new();
        let message = cdata(xml, "log4j:Message");
        let mut lignes = message.lines();
        let premiere = lignes.next().unwrap_or("");
        sortie.push((format!("[{moment}] [{fil}/{niveau_txt}]: {premiere}"), niveau));
        sortie.extend(lignes.map(|l| (l.to_string(), niveau)));
        let pile = cdata(xml, "log4j:Throwable");
        sortie.extend(pile.lines().filter(|l| !l.trim().is_empty()).map(|l| (l.to_string(), niveau)));
        sortie
    }
}

/// Contenu d'une balise (dans un CDATA ou non).
fn cdata(xml: &str, balise: &str) -> String {
    let ouvre = format!("<{balise}>");
    let ferme = format!("</{balise}>");
    let Some(i) = xml.find(&ouvre) else { return String::new() };
    let reste = &xml[i + ouvre.len()..];
    let contenu = reste.find(&ferme).map(|j| &reste[..j]).unwrap_or(reste);
    match contenu.trim().strip_prefix("<![CDATA[") {
        Some(c) => c.strip_suffix("]]>").unwrap_or(c).to_string(),
        None => deseschapper(contenu.trim()),
    }
}

fn deseschapper(s: &str) -> String {
    s.replace("&lt;", "<").replace("&gt;", ">").replace("&quot;", "\"").replace("&apos;", "'").replace("&amp;", "&")
}

/// Niveau d'une ligne de texte simple (log4j en texte, journaux d'Horizon, traces Java).
fn niveau_texte(ligne: &str, erreurs: bool, precedent: i32) -> i32 {
    let haut = ligne.to_ascii_uppercase();
    if haut.contains("/ERROR]") || haut.contains("/FATAL]") || haut.contains(" ERROR ") || haut.contains("[ERROR]") || haut.starts_with("ERROR") {
        return ERREUR;
    }
    if haut.contains("/WARN]") || haut.contains(" WARN ") || haut.contains("[WARN]") || haut.starts_with("WARN") {
        return AVERT;
    }
    if haut.contains("/INFO]") || haut.contains(" INFO ") || haut.contains("[INFO]") || haut.contains(" DEBUG ") || haut.contains("/DEBUG]") {
        return INFO;
    }
    // suite d'une trace de pile : même niveau que le message qui la précède
    let t = ligne.trim_start();
    if t.starts_with("at ") || t.starts_with("Caused by") || t.starts_with("... ") || t.starts_with("Suppressed:") {
        return precedent.max(AVERT);
    }
    if t.contains("Exception") || t.contains("panicked at") {
        return ERREUR;
    }
    if erreurs { AVERT } else { INFO }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn evenements_log4j() {
        let mut a = Analyseur::default();
        assert!(a.ligne(r#"<log4j:Event logger="dhq" timestamp="1700000000000" level="WARN" thread="Render thread">"#, false).is_empty());
        assert!(a.ligne("  <log4j:Message><![CDATA[Missing sound for event: <x>]]></log4j:Message>", false).is_empty());
        let l = a.ligne("</log4j:Event>", false);
        assert_eq!(l.len(), 1);
        assert!(l[0].0.ends_with("[Render thread/WARN]: Missing sound for event: <x>"), "{}", l[0].0);
        assert_eq!(l[0].1, AVERT);

        let xml = r#"<log4j:Event logger="x" timestamp="1700000000000" level="ERROR" thread="main"><log4j:Message><![CDATA[Crash]]></log4j:Message><log4j:Throwable><![CDATA[java.lang.NullPointerException
	at a.b(C.java:1)
]]></log4j:Throwable></log4j:Event>"#;
        let mut sortie = Vec::new();
        for ligne in xml.lines() {
            sortie.extend(a.ligne(ligne, false));
        }
        assert_eq!(sortie.len(), 3);
        assert!(sortie.iter().all(|(_, n)| *n == ERREUR));
        assert_eq!(sortie[2].0, "\tat a.b(C.java:1)");
    }

    #[test]
    fn texte_simple() {
        let mut a = Analyseur::default();
        assert_eq!(a.ligne("[12:00:00] [main/INFO]: Hello", false)[0].1, INFO);
        assert_eq!(a.ligne("[12:00:00] [main/ERROR]: Boom", false)[0].1, ERREUR);
        assert_eq!(a.ligne("\tat net.minecraft.Main.main(Main.java:1)", false)[0].1, ERREUR);
        assert_eq!(a.ligne("2026-11-09T10:00:00Z  WARN horizon: slow chunk", true)[0].1, AVERT);
        assert_eq!(a.ligne("quelque chose", true)[0].1, AVERT);
        assert!(a.ligne("   ", false).is_empty());
    }
}
