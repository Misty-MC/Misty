//! Fichiers du jeu partagés entre toutes les versions et tous les profils : la liste des serveurs
//! (servers.dat) et les barres d'objets sauvegardées (hotbar.nbt). Ajouter un serveur dans un
//! profil 1.21 le fait apparaître aussi en 1.8, en jeu rapide ou dans un modpack.
//!
//! La copie de référence est celle du dossier principal (%LOCALAPPDATA%\Misty, celui du jeu rapide).
//! Avant chaque partie, le profil reçoit la copie la plus récente ; après la partie, ce que le jeu a
//! modifié remonte dans le dossier principal. La plus récente gagne, pour que les suppressions
//! restent supprimées.
//!
//! Première fois qu'un dossier est partagé (fichier `.misty-partage` absent) : les listes de
//! serveurs sont fusionnées plutôt qu'écrasées, pour ne perdre aucun serveur ajouté avant. Le
//! dossier principal reprend aussi une fois les serveurs du launcher Minecraft officiel (.minecraft).

use super::{paths, settings};
use std::fs;
use std::path::{Path, PathBuf};

const SERVEURS: &str = "servers.dat";
const BARRES: &str = "hotbar.nbt";
const FICHIERS: &[&str] = &[SERVEURS, BARRES];
/// Présent dans un dossier de jeu déjà partagé (sa liste de serveurs a été fusionnée).
const MARQUE: &str = ".misty-partage";

/// Réglage : partage activé (Options → Comportement).
pub fn active() -> bool {
    settings::get_bool(settings::PARTAGE, true)
}

/// Dossier qui garde la copie de référence.
fn reference() -> PathBuf {
    paths::data_dir()
}

/// Le .minecraft du launcher officiel.
fn minecraft_officiel() -> Option<PathBuf> {
    dirs::data_dir().map(|d| d.join(".minecraft")).filter(|d| d.is_dir())
}

/// Avant de lancer le jeu dans `dossier_jeu`.
pub fn avant_partie(dossier_jeu: &Path) {
    if !active() {
        return;
    }
    let reference = reference();
    if !reference.join(MARQUE).exists() {
        if let Some(officiel) = minecraft_officiel() {
            fusionner_premiere_fois(&officiel, &reference, false);
        }
        marquer(&reference);
    }
    if meme_dossier(dossier_jeu, &reference) {
        return;
    }
    if dossier_jeu.join(MARQUE).exists() {
        for nom in FICHIERS {
            copier_si_plus_recent(&reference.join(nom), &dossier_jeu.join(nom));
        }
    } else {
        fusionner_premiere_fois(dossier_jeu, &reference, true);
        marquer(dossier_jeu);
    }
}

/// Après la partie : ce que le jeu a enregistré dans le profil remonte dans le dossier principal.
pub fn apres_partie(dossier_jeu: &Path) {
    let reference = reference();
    if !active() || meme_dossier(dossier_jeu, &reference) {
        return;
    }
    for nom in FICHIERS {
        copier_si_plus_recent(&dossier_jeu.join(nom), &reference.join(nom));
    }
}

fn marquer(dossier: &Path) {
    let _ = fs::write(dossier.join(MARQUE), b"");
}

fn meme_dossier(a: &Path, b: &Path) -> bool {
    match (fs::canonicalize(a), fs::canonicalize(b)) {
        (Ok(a), Ok(b)) => a == b,
        _ => a == b,
    }
}

fn date(chemin: &Path) -> Option<std::time::SystemTime> {
    fs::metadata(chemin).and_then(|m| m.modified()).ok()
}

/// Copie `source` sur `cible` si la cible manque ou est plus ancienne. fs::copy garde la date de
/// modification sous Windows : les deux copies ont ensuite la même date.
fn copier_si_plus_recent(source: &Path, cible: &Path) {
    let Some(d_source) = date(source) else { return };
    if date(cible).is_some_and(|d| d >= d_source) {
        return;
    }
    if let Some(parent) = cible.parent() {
        let _ = fs::create_dir_all(parent);
    }
    let _ = fs::copy(source, cible);
}

/// Premier partage de `autre` avec `reference` : serveurs fusionnés (ceux de la référence d'abord)
/// dans les deux dossiers ; barres d'objets de la référence si elle en a, sinon celles de `autre`.
/// `ecrire_autre` : false pour le .minecraft officiel, qu'on ne modifie jamais.
fn fusionner_premiere_fois(autre: &Path, reference: &Path, ecrire_autre: bool) {
    let (s_ref, s_autre) = (reference.join(SERVEURS), autre.join(SERVEURS));
    match (fs::read(&s_ref).ok(), fs::read(&s_autre).ok()) {
        (Some(a), Some(b)) => {
            if let Some(fusion) = nbt::fusionner_serveurs(&a, &b) {
                if fusion != a {
                    let _ = fs::write(&s_ref, &fusion);
                }
                if ecrire_autre {
                    let _ = fs::copy(&s_ref, &s_autre);
                }
            }
        }
        (None, Some(_)) => {
            let _ = fs::copy(&s_autre, &s_ref);
        }
        (Some(_), None) if ecrire_autre => {
            let _ = fs::copy(&s_ref, &s_autre);
        }
        _ => {}
    }

    let (b_ref, b_autre) = (reference.join(BARRES), autre.join(BARRES));
    match (b_ref.exists(), b_autre.exists()) {
        (false, true) => {
            let _ = fs::copy(&b_autre, &b_ref);
        }
        (true, _) if ecrire_autre => {
            // les barres du profil sont gardées à côté, au cas où
            if b_autre.exists() && fs::read(&b_autre).ok() != fs::read(&b_ref).ok() {
                let _ = fs::copy(&b_autre, autre.join("hotbar.nbt.bak"));
            }
            let _ = fs::copy(&b_ref, &b_autre);
        }
        _ => {}
    }
}

/// Lecture minimale du format NBT (non compressé, comme servers.dat) : juste ce qu'il faut pour
/// fusionner deux listes de serveurs sans rien perdre des autres champs.
mod nbt {
    const FIN: u8 = 0;
    const LISTE: u8 = 9;
    const COMPOSE: u8 = 10;
    const CHAINE: u8 = 8;

    struct Lecteur<'a> {
        d: &'a [u8],
        p: usize,
    }

    impl Lecteur<'_> {
        fn octets(&mut self, n: usize) -> Option<&[u8]> {
            let fin = self.p.checked_add(n)?;
            let tranche = self.d.get(self.p..fin)?;
            self.p = fin;
            Some(tranche)
        }
        fn u8(&mut self) -> Option<u8> {
            self.octets(1).map(|o| o[0])
        }
        fn u16(&mut self) -> Option<u16> {
            self.octets(2).map(|o| u16::from_be_bytes([o[0], o[1]]))
        }
        fn i32(&mut self) -> Option<i32> {
            self.octets(4).map(|o| i32::from_be_bytes([o[0], o[1], o[2], o[3]]))
        }
        fn chaine(&mut self) -> Option<String> {
            let n = self.u16()? as usize;
            Some(String::from_utf8_lossy(self.octets(n)?).into_owned())
        }
        fn longueur(&mut self) -> Option<usize> {
            usize::try_from(self.i32()?).ok()
        }

        /// Passe la valeur d'un tag de type `t`.
        fn sauter(&mut self, t: u8) -> Option<()> {
            match t {
                1 => self.octets(1).map(drop),
                2 => self.octets(2).map(drop),
                3 | 5 => self.octets(4).map(drop),
                4 | 6 => self.octets(8).map(drop),
                7 => {
                    let n = self.longueur()?;
                    self.octets(n).map(drop)
                }
                CHAINE => self.chaine().map(drop),
                LISTE => {
                    let t = self.u8()?;
                    for _ in 0..self.longueur()? {
                        self.sauter(t)?;
                    }
                    Some(())
                }
                COMPOSE => loop {
                    let t = self.u8()?;
                    if t == FIN {
                        return Some(());
                    }
                    self.chaine()?;
                    self.sauter(t)?;
                },
                11 => {
                    let n = self.longueur()?;
                    self.octets(n.checked_mul(4)?).map(drop)
                }
                12 => {
                    let n = self.longueur()?;
                    self.octets(n.checked_mul(8)?).map(drop)
                }
                _ => None,
            }
        }
    }

    /// Un fichier de serveurs découpé : les champs de la racine avant et après la liste, et chaque
    /// serveur (octets bruts du composé + son adresse).
    struct Serveurs {
        avant: Vec<u8>,
        apres: Vec<u8>,
        liste: Vec<(String, Vec<u8>)>,
    }

    fn lire(d: &[u8]) -> Option<Serveurs> {
        let mut l = Lecteur { d, p: 0 };
        if l.u8()? != COMPOSE {
            return None;
        }
        l.chaine()?;
        let mut resultat = None;
        loop {
            let debut = l.p;
            let t = l.u8()?;
            if t == FIN {
                break;
            }
            let nom = l.chaine()?;
            if t == LISTE && nom == "servers" && resultat.is_none() {
                let type_elements = l.u8()?;
                let n = l.longueur()?;
                if type_elements != COMPOSE && n > 0 {
                    return None;
                }
                let mut liste = Vec::with_capacity(n);
                for _ in 0..n {
                    let depart = l.p;
                    let ip = adresse(&mut l)?;
                    liste.push((ip, d[depart..l.p].to_vec()));
                }
                resultat = Some((debut, l.p, liste));
            } else {
                l.sauter(t)?;
            }
        }
        if l.p != d.len() {
            return None;
        }
        let fin_champs = l.p - 1; // avant le tag de fin de la racine
        let (debut, fin, liste) = resultat.unwrap_or((fin_champs, fin_champs, Vec::new()));
        Some(Serveurs { avant: d[..debut].to_vec(), apres: d[fin..fin_champs].to_vec(), liste })
    }

    /// Lit un serveur (composé) et renvoie son adresse, en minuscules.
    fn adresse(l: &mut Lecteur) -> Option<String> {
        let mut ip = String::new();
        loop {
            let t = l.u8()?;
            if t == FIN {
                return Some(ip.trim().to_lowercase());
            }
            let nom = l.chaine()?;
            if t == CHAINE && nom == "ip" {
                ip = l.chaine()?;
            } else {
                l.sauter(t)?;
            }
        }
    }

    /// Les serveurs de `a`, puis ceux de `b` dont l'adresse n'est pas déjà dans `a`.
    /// None si l'un des fichiers est illisible (on ne touche alors à rien).
    pub fn fusionner_serveurs(a: &[u8], b: &[u8]) -> Option<Vec<u8>> {
        let a = lire(a)?;
        let b = lire(b)?;
        let mut liste = a.liste;
        for (ip, octets) in b.liste {
            if ip.is_empty() || !liste.iter().any(|(i, _)| *i == ip) {
                liste.push((ip, octets));
            }
        }
        let mut sortie = a.avant;
        sortie.push(LISTE);
        sortie.extend_from_slice(&(b"servers".len() as u16).to_be_bytes());
        sortie.extend_from_slice(b"servers");
        sortie.push(COMPOSE);
        sortie.extend_from_slice(&(liste.len() as i32).to_be_bytes());
        for (_, octets) in &liste {
            sortie.extend_from_slice(octets);
        }
        sortie.extend_from_slice(&a.apres);
        sortie.push(FIN);
        Some(sortie)
    }

    #[cfg(test)]
    pub mod tests {
        use super::*;

        fn chaine(s: &mut Vec<u8>, texte: &str) {
            s.extend_from_slice(&(texte.len() as u16).to_be_bytes());
            s.extend_from_slice(texte.as_bytes());
        }

        /// servers.dat comme l'écrit Minecraft : { servers: [ { name, ip, hidden } … ] }
        pub fn fichier(serveurs: &[(&str, &str)]) -> Vec<u8> {
            let mut s = vec![COMPOSE];
            chaine(&mut s, "");
            s.push(LISTE);
            chaine(&mut s, "servers");
            s.push(COMPOSE);
            s.extend_from_slice(&(serveurs.len() as i32).to_be_bytes());
            for (nom, ip) in serveurs {
                s.push(CHAINE);
                chaine(&mut s, "name");
                chaine(&mut s, nom);
                s.push(CHAINE);
                chaine(&mut s, "ip");
                chaine(&mut s, ip);
                s.push(1);
                chaine(&mut s, "hidden");
                s.push(0);
                s.push(FIN);
            }
            s.push(FIN);
            s
        }

        fn adresses(d: &[u8]) -> Vec<String> {
            lire(d).unwrap().liste.into_iter().map(|(ip, _)| ip).collect()
        }

        #[test]
        fn fusion_sans_doublon() {
            let a = fichier(&[("Hypixel", "mc.hypixel.net"), ("Local", "localhost")]);
            let b = fichier(&[("hypixel", "MC.Hypixel.net "), ("Ami", "ami.example:25566")]);
            let f = fusionner_serveurs(&a, &b).unwrap();
            assert_eq!(adresses(&f), ["mc.hypixel.net", "localhost", "ami.example:25566"]);
            // le résultat se relit et refusionner ne change rien
            assert_eq!(fusionner_serveurs(&f, &b).unwrap(), f);
        }

        #[test]
        fn liste_vide_et_fichier_abime() {
            let vide = fichier(&[]);
            let b = fichier(&[("Ami", "ami.example")]);
            assert_eq!(fusionner_serveurs(&vide, &b).unwrap(), b);
            assert!(fusionner_serveurs(&b[..b.len() - 3], &vide).is_none());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn premier_partage_puis_plus_recent() {
        let racine = std::env::temp_dir().join(format!("misty-partage-{}", rand::random::<u32>()));
        let (reference, profil) = (racine.join("ref"), racine.join("profil"));
        fs::create_dir_all(&reference).unwrap();
        fs::create_dir_all(&profil).unwrap();
        fs::write(reference.join(SERVEURS), nbt::tests::fichier(&[("A", "a.net")])).unwrap();
        fs::write(profil.join(SERVEURS), nbt::tests::fichier(&[("B", "b.net")])).unwrap();
        fs::write(profil.join(BARRES), b"barres").unwrap();

        fusionner_premiere_fois(&profil, &reference, true);
        let attendu = nbt::tests::fichier(&[("A", "a.net"), ("B", "b.net")]);
        assert_eq!(fs::read(reference.join(SERVEURS)).unwrap(), attendu);
        assert_eq!(fs::read(profil.join(SERVEURS)).unwrap(), attendu);
        assert_eq!(fs::read(reference.join(BARRES)).unwrap(), b"barres");

        // le jeu modifie la liste du profil : elle remonte
        std::thread::sleep(std::time::Duration::from_millis(20));
        let nouvelle = nbt::tests::fichier(&[("B", "b.net")]);
        fs::write(profil.join(SERVEURS), &nouvelle).unwrap();
        copier_si_plus_recent(&profil.join(SERVEURS), &reference.join(SERVEURS));
        assert_eq!(fs::read(reference.join(SERVEURS)).unwrap(), nouvelle);
        let _ = fs::remove_dir_all(racine);
    }
}
