//! Fenêtre de choix de version : versions regroupées par mise à jour (1.21 « Tricky Trials »…),
//! chaque mise à jour illustrée par un petit paysage en pixel art à ses couleurs, et filtres
//! (types de versions, mod loader compatible, recherche).

use crate::services::{loc, version_compat};
use slint::{Image, Rgba8Pixel, SharedPixelBuffer};
use std::cell::RefCell;
use std::collections::HashMap;

/// Une version de Mojang : identifiant, type (0 release, 1 snapshot, 2 bêta, 3 alpha), année de sortie.
#[derive(Clone)]
pub struct Entree {
    pub id: String,
    pub kind: i32,
    pub annee: String,
}

/// Groupe d'une version : « 1.21 » pour 1.21.4, et un groupe par type pour les autres.
pub fn cle_groupe(id: &str, kind: i32) -> String {
    match kind {
        1 => "snapshots".into(),
        2 => "beta".into(),
        3 => "alpha".into(),
        _ => {
            let p: Vec<&str> = id.split('.').collect();
            match p.first().and_then(|m| m.parse::<u32>().ok()) {
                // numérotation à l'année (26.1, 26.2…) : un groupe par année
                Some(annee) if annee >= 20 => format!("{annee}.x"),
                _ if p.len() >= 2 => format!("{}.{}", p[0], p[1]),
                _ => id.to_string(),
            }
        }
    }
}

/// Titre et nom de la mise à jour (noms officiels de Mojang).
pub fn titre_groupe(cle: &str) -> (String, String) {
    let nom = match cle {
        "1.21" => "Tricky Trials",
        "1.20" => "Trails & Tales",
        "1.19" => "The Wild Update",
        "1.18" => "Caves & Cliffs II",
        "1.17" => "Caves & Cliffs I",
        "1.16" => "Nether Update",
        "1.15" => "Buzzy Bees",
        "1.14" => "Village & Pillage",
        "1.13" => "Update Aquatic",
        "1.12" => "World of Color",
        "1.11" => "Exploration Update",
        "1.10" => "Frostburn Update",
        "1.9" => "Combat Update",
        "1.8" => "Bountiful Update",
        "1.7" => "The Update that Changed the World",
        "1.6" => "Horse Update",
        "1.5" => "Redstone Update",
        "1.4" => "Pretty Scary Update",
        "1.0" => "Adventure Update",
        "snapshots" => return ("Snapshots".into(), loc::t("Ver_Groupe_Snapshots")),
        "beta" => return ("Beta".into(), "Beta 1.0 – 1.8".into()),
        "alpha" => return ("Alpha".into(), "Alpha · Infdev · Indev".into()),
        c if c.ends_with(".x") => return (c.to_string(), format!("20{}", c.trim_end_matches(".x"))),
        _ => "",
    };
    (cle.to_string(), nom.to_string())
}

/// Le mod loader filtré (0 tous, 1 Fabric, 2 Forge, 3 NeoForge, 4 OptiFine, 5 LiteLoader) existe-t-il
/// pour cette version ? (plages connues : vérification rapide, sans réseau)
pub fn compatible(id: &str, loader: i32) -> bool {
    match loader {
        1 => version_compat::fabric(id),
        2 => version_compat::forge(id),
        3 => version_compat::neoforge(id),
        4 => version_compat::optifine(id),
        5 => version_compat::liteloader(id),
        _ => true,
    }
}

/// Nom du loader filtré, tel qu'il apparaît dans la liste des modes.
pub fn nom_loader(loader: i32) -> Option<&'static str> {
    use crate::minecraft::loaders::{FABRIC, FORGE, LITELOADER, NEOFORGE, OPTIFINE};
    match loader {
        1 => Some(FABRIC),
        2 => Some(FORGE),
        3 => Some(NEOFORGE),
        4 => Some(OPTIFINE),
        5 => Some(LITELOADER),
        _ => None,
    }
}

// ───────────────────────── Illustrations ─────────────────────────

const L: u32 = 160;
const H: u32 = 90;

/// Couleurs d'une illustration : ciel (haut, bas), crêtes (loin, près), sol (dessus, terre), nuit.
struct Palette {
    ciel: [[u8; 3]; 2],
    cretes: [[u8; 3]; 2],
    sol: [[u8; 3]; 2],
    nuit: bool,
}

fn hex(c: u32) -> [u8; 3] {
    [(c >> 16) as u8, (c >> 8) as u8, c as u8]
}

fn palette(cle: &str) -> Palette {
    let p = |c: [u32; 6], nuit: bool| Palette {
        ciel: [hex(c[0]), hex(c[1])],
        cretes: [hex(c[2]), hex(c[3])],
        sol: [hex(c[4]), hex(c[5])],
        nuit,
    };
    match cle {
        "1.21" => p([0xE0874A, 0xF6D7A7, 0xB5651D, 0x8C4A1A, 0xC87533, 0x6B3A1E], false),
        "1.20" => p([0xF3A6C8, 0xFFE9F2, 0xE58FB5, 0xB85C8A, 0xF5A9C8, 0x7A4B3A], false),
        "1.19" => p([0x0B1F2A, 0x1C4A5E, 0x12303D, 0x0A1F28, 0x0F7C80, 0x0A1A20], true),
        "1.18" | "1.17" => p([0x87B9E8, 0xCFE7FA, 0x7D8A99, 0x5B6672, 0x5FA84B, 0x6E5A4A], false),
        "1.16" => p([0x3A0D12, 0x8A2A24, 0x5A1418, 0x3A0C10, 0x9B2335, 0x4A1014], true),
        "1.15" => p([0xFFD86A, 0xFFF3C4, 0xE8B13A, 0xC78A1E, 0x6DBF4B, 0x7A5A3A], false),
        "1.14" => p([0x8FC6F5, 0xD9EEFF, 0x9DB86A, 0x6E8E42, 0x7CBF4A, 0x8A6A44], false),
        "1.13" => p([0x2C7FB8, 0x7FD3E8, 0x1F6C8C, 0x155A75, 0x3FB8B0, 0x1C4C5C], false),
        "1.12" => p([0xE86F68, 0xF6C177, 0xC25B7A, 0x8E4A9E, 0xE6A34A, 0x7A4A3A], false),
        "1.11" | "1.10" => p([0xA9D6F0, 0xEEF8FF, 0xA9C4D6, 0x7E9AB0, 0xF2F7FA, 0x8A7A6A], false),
        "1.9" => p([0x140B26, 0x3B2A5C, 0x2E2148, 0x1E1533, 0xE8E4B0, 0xA6A27A], true),
        "1.8" => p([0x2E6E8E, 0x7FC0D8, 0x3E8A8A, 0x2A6A6A, 0x5DBF9F, 0x3A5A5A], false),
        "horizon" => p([0x2A1236, 0xF0795A, 0x7A2E3A, 0x3E1426, 0xF29A5C, 0x4A1A20], false),
        "snapshots" => p([0x221540, 0x5A3F8A, 0x3E2A6A, 0x2A1A4A, 0x9C7CFF, 0x3A2A5A], true),
        "beta" => p([0xC9B28A, 0xEAD9B5, 0x9A8460, 0x7A6648, 0x8FAF5A, 0x6A4E34], false),
        "alpha" => p([0x9FC49F, 0xD8EAD0, 0x789A6A, 0x587A4A, 0x6FA04A, 0x5A4030], false),
        c if c.starts_with("1.") => p([0x7FB3F5, 0xBFDDFF, 0x6FA36A, 0x4E8A48, 0x5FBF3F, 0x8A5A3A], false),
        // nouvelles versions (26.x…) : la nuit de Misty
        _ => p([0x0E1527, 0x2A4A7A, 0x24345A, 0x162140, 0x6F9CFF, 0x162140], true),
    }
}

/// Petit générateur pseudo-aléatoire (même graine = même dessin).
struct Rng(u64);
impl Rng {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 >> 12;
        self.0 ^= self.0 << 25;
        self.0 ^= self.0 >> 27;
        self.0.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }
    fn float(&mut self) -> f32 {
        (self.next() >> 40) as f32 / (1u64 << 24) as f32
    }
}

fn melange(a: [u8; 3], b: [u8; 3], t: f32) -> [u8; 3] {
    [0, 1, 2].map(|i| (a[i] as f32 + (b[i] as f32 - a[i] as f32) * t).round() as u8)
}

/// Paysage en pixel art : ciel en dégradé, astre, deux crêtes, sol en blocs, quelques arbres.
fn dessiner(cle: &str) -> SharedPixelBuffer<Rgba8Pixel> {
    let pal = palette(cle);
    let graine = cle.bytes().fold(0xcbf2_9ce4_8422_2325u64, |h, b| (h ^ b as u64).wrapping_mul(0x100_0000_01b3)) | 1;
    let mut rng = Rng(graine);
    let mut t = Toile { px: vec![[0u8; 3]; (L * H) as usize] };
    // ciel, par bandes de 6 pixels (effet pixel art)
    for y in 0..H as i32 {
        let k = ((y / 6) * 6) as f32 / H as f32;
        for x in 0..L as i32 {
            t.pose(x, y, melange(pal.ciel[0], pal.ciel[1], k));
        }
    }
    // étoiles la nuit
    if pal.nuit {
        for _ in 0..40 {
            let (x, y) = ((rng.float() * L as f32) as i32, (rng.float() * H as f32 * 0.5) as i32);
            t.pose(x, y, melange(pal.ciel[1], [255, 255, 255], 0.75));
        }
    }
    // astre : soleil carré le jour, lune la nuit
    let (ax, ay, r) = ((L as f32 * (0.62 + rng.float() * 0.25)) as i32, 14 + (rng.float() * 8.0) as i32, 6);
    let astre = if pal.nuit { [234, 241, 255] } else { [255, 244, 196] };
    for dy in -r..=r {
        for dx in -r..=r {
            if dx * dx + dy * dy <= r * r {
                t.pose(ax + dx, ay + dy, astre);
            }
        }
    }
    // deux crêtes, de la plus lointaine à la plus proche
    for (i, base) in [(0usize, 52.0f32), (1, 62.0)] {
        let (p1, p2) = (rng.float() * 6.3, rng.float() * 6.3);
        let amp = if i == 0 { 12.0 } else { 7.0 };
        let freq = if i == 0 { 0.045 } else { 0.08 };
        for x in 0..L as i32 {
            let v = 0.6 * (x as f32 * freq + p1).sin() + 0.4 * (x as f32 * freq * 2.3 + p2).sin();
            let haut = ((base - amp * v) / 2.0).round() as i32 * 2;
            for y in haut..H as i32 {
                t.pose(x, y, pal.cretes[i]);
            }
        }
    }
    // repère derrière le sol (falaise de 1.18)
    structure_fond(&mut t, cle);
    // sol en blocs de 8 : dessus et terre, avec un peu de grain
    let sol = 74;
    for y in sol..H as i32 {
        for x in 0..L as i32 {
            let bloc = ((x / 4 + y / 4) % 3) as f32 * 0.04;
            let base = if y < sol + 4 { pal.sol[0] } else { pal.sol[1] };
            t.pose(x, y, melange(base, [0, 0, 0], bloc));
        }
    }
    // quelques arbres (troncs et feuillage) sur le sol
    let mut x = 6 + (rng.float() * 10.0) as i32;
    let libre = emprise(cle);
    while x < L as i32 - 8 {
        // pas d'arbre sur la structure de la mise à jour
        if libre.is_some_and(|(a, b)| x + 5 >= a && x - 4 <= b) {
            x += 20 + (rng.float() * 30.0) as i32;
            continue;
        }
        let tronc = melange(pal.sol[1], [0, 0, 0], 0.3);
        let feuilles = melange(pal.cretes[1], pal.sol[0], 0.35);
        for y in sol - 8..sol {
            t.pose(x, y, tronc);
            t.pose(x + 1, y, tronc);
        }
        for dy in 0..6 {
            for dx in -3..=4 {
                if (dx as i32).abs() + dy / 2 < 5 {
                    t.pose(x + dx, sol - 14 + dy, feuilles);
                }
            }
        }
        x += 20 + (rng.float() * 30.0) as i32;
    }
    // le repère de la mise à jour (Warden, cerisier, portail…)
    structure(&mut t, cle, &mut rng);
    let mut tampon = SharedPixelBuffer::<Rgba8Pixel>::new(L, H);
    for (p, c) in tampon.make_mut_slice().iter_mut().zip(&t.px) {
        *p = Rgba8Pixel { r: c[0], g: c[1], b: c[2], a: 255 };
    }
    tampon
}

// ───────────── Structures : un repère de chaque mise à jour, posé à droite du paysage ─────────────

/// Toile de l'illustration (RGB), avec de quoi y dessiner.
struct Toile {
    px: Vec<[u8; 3]>,
}

impl Toile {
    fn pose(&mut self, x: i32, y: i32, c: [u8; 3]) {
        if (0..L as i32).contains(&x) && (0..H as i32).contains(&y) {
            self.px[(y as u32 * L + x as u32) as usize] = c;
        }
    }
    fn rect(&mut self, x: i32, y: i32, w: i32, h: i32, c: [u8; 3]) {
        for dy in 0..h {
            for dx in 0..w {
                self.pose(x + dx, y + dy, c);
            }
        }
    }
    fn disque(&mut self, cx: i32, cy: i32, r: i32, c: [u8; 3]) {
        for dy in -r..=r {
            for dx in -r..=r {
                if dx * dx + dy * dy <= r * r + r / 2 {
                    self.pose(cx + dx, cy + dy, c);
                }
            }
        }
    }
    /// Sprite en texte : une lettre par pixel, « . » transparent ; `y` est la ligne du bas.
    fn sprite(&mut self, x: i32, y: i32, lignes: &[&str], couleurs: &[(char, u32)]) {
        self.sprite_k(x, y, lignes, couleurs, 1);
    }
    /// Le même, agrandi `k` fois.
    fn sprite_k(&mut self, x: i32, y: i32, lignes: &[&str], couleurs: &[(char, u32)], k: i32) {
        let haut = lignes.len() as i32 * k;
        for (j, ligne) in lignes.iter().enumerate() {
            for (i, ch) in ligne.chars().enumerate() {
                if let Some((_, c)) = couleurs.iter().find(|(n, _)| *n == ch) {
                    self.rect(x + i as i32 * k, y - haut + 1 + j as i32 * k, k, k, hex(*c));
                }
            }
        }
    }
}

/// Où la structure se pose (de x à x + largeur), pour laisser la place libre aux arbres.
fn emprise(cle: &str) -> Option<(i32, i32)> {
    Some(match cle {
        "1.21" => (100, 150),
        "1.20" => (98, 140),
        "1.19" => (104, 146),
        "1.17" => (100, 150),
        "1.16" => (106, 140),
        "1.15" => (104, 146),
        "1.14" => (104, 140),
        "1.13" => (96, 156),
        "1.12" => (100, 150),
        "1.11" => (96, 150),
        "1.10" => (104, 146),
        "1.9" => (106, 146),
        "1.8" => (96, 156),
        "horizon" => (110, 140),
        _ => return None,
    })
}

/// Derrière le sol (montagnes) : seulement 1.18.
fn structure_fond(t: &mut Toile, cle: &str) {
    if cle == "1.18" {
        // falaise enneigée, en marches de 2 pixels
        let (pied, sommet, cx) = (74, 40, 128);
        for y in sommet..pied {
            let demi = ((y - sommet) as f32 * 1.15 + 3.0) as i32 / 2 * 2;
            for x in cx - demi..=cx + demi {
                let neige = y < sommet + 9 + ((x * 7) % 5) - 2;
                let ombre = x > cx + demi / 3;
                let c = if neige { if ombre { 0xD6E2EE } else { 0xF6FAFF } } else if ombre { 0x5B6672 } else { 0x7D8A99 };
                t.pose(x, y, hex(c));
            }
        }
    }
}

/// Devant le sol : les autres structures.
fn structure(t: &mut Toile, cle: &str, rng: &mut Rng) {
    let sol = 74;
    match cle {
        // Tricky Trials : trial spawner, blocs de cuivre et un breeze
        "1.21" => {
            let (x, y) = (116, sol - 14);
            t.rect(x, y, 14, 14, hex(0x2B2F3A));
            for i in (x + 1..x + 14).step_by(3) {
                t.rect(i, y + 1, 1, 12, hex(0x4A5060));
            }
            t.rect(x + 4, y + 4, 6, 6, hex(0xFF9A3D));
            t.rect(x + 5, y + 5, 4, 4, hex(0xFFD27A));
            for (fx, fy) in [(x + 3, y - 2), (x + 7, y - 4), (x + 10, y - 2), (x + 6, y - 1)] {
                t.rect(fx, fy, 1, 2, hex(0xFF9A3D));
            }
            for (bx, c1, c2) in [(102, 0xC87533, 0xE0955A), (134, 0x4FA38A, 0x6FC3A8), (142, 0xC87533, 0x4FA38A)] {
                t.rect(bx, sol - 8, 8, 8, hex(c1));
                t.rect(bx + 1, sol - 7, 3, 3, hex(c2));
            }
            t.sprite(104, sol - 20, &[".bb.", "bwwb", ".bb.", "bb..", ".bb."], &[('b', 0x8FD3F5), ('w', 0xEAF7FF)]);
        }
        // Trails & Tales : cerisier en fleurs, pétales qui tombent
        "1.20" => {
            let tronc = hex(0x4A2A2A);
            t.rect(118, sol - 18, 3, 18, tronc);
            t.rect(114, sol - 22, 2, 6, tronc);
            t.rect(116, sol - 18, 2, 2, tronc);
            t.rect(121, sol - 24, 2, 8, tronc);
            for (cx, cy, r) in [(112, 50, 9), (124, 46, 10), (135, 52, 8), (118, 55, 8), (103, 55, 6)] {
                t.disque(cx, cy + 1, r, hex(0xA8456E));
            }
            for (cx, cy, r) in [(112, 50, 8), (124, 46, 9), (135, 52, 7), (118, 55, 7), (103, 55, 5)] {
                t.disque(cx, cy, r, hex(0xE0699E));
            }
            for _ in 0..120 {
                let (x, y) = (96 + (rng.float() * 50.0) as i32, 36 + (rng.float() * 28.0) as i32);
                if t.px[(y as u32 * L + x as u32) as usize] == hex(0xE0699E) {
                    t.pose(x, y, hex(if rng.float() < 0.55 { 0xFFC2DC } else { 0xC24F84 }));
                }
            }
            for _ in 0..10 {
                t.pose(100 + (rng.float() * 50.0) as i32, 62 + (rng.float() * 12.0) as i32, hex(0xE0699E));
            }
        }
        // The Wild Update : le Warden sur un tapis de sculk
        "1.19" => {
            for x in (100..152).step_by(2) {
                if rng.float() < 0.7 {
                    t.rect(x, sol, 2, 2, hex(0x0B2E38));
                }
                if rng.float() < 0.15 {
                    t.pose(x, sol, hex(0x3FE0E0));
                }
            }
            t.sprite_k(
                114,
                sol - 1,
                &[
                    ".h..........h.",
                    ".hh........hh.",
                    "..hDDDDDDDDh..",
                    "..DDDDDDDDDD..",
                    "..DdDDDDDDdD..",
                    "..DDDDmmDDDD..",
                    "..DDDDDDDDDD..",
                    "DDDDcDDDDcDDDD",
                    "DDDcDcDDcDcDDD",
                    "DD.DDcccDDD.DD",
                    "DD.DDDcDDDD.DD",
                    "DD.DDDDDDDD.DD",
                    "dd.DDDDDDDD.dd",
                    "...DDD..DDD...",
                    "...DDD..DDD...",
                    "...ddd..ddd...",
                ],
                &[('D', 0x1E5A6E), ('d', 0x134050), ('h', 0x5FF5F0), ('c', 0x5FF5F0), ('m', 0x05141A)],
                2,
            );
            // hurleur de sculk
            t.rect(102, sol - 5, 6, 5, hex(0x1E3A40));
            t.rect(103, sol - 7, 4, 2, hex(0xD8E8D0));
        }
        // Caves & Cliffs I : géode d'améthyste et un axolotl
        "1.17" => {
            for (x, h, c) in [(104, 10, 0x9A5CC6), (108, 15, 0xC890FF), (112, 9, 0x6A3A9A), (138, 12, 0xC890FF), (142, 7, 0x9A5CC6), (146, 11, 0x6A3A9A)] {
                for dy in 0..h {
                    let w = if dy > h - 3 { 1 } else { 3 };
                    t.rect(x + (3 - w) / 2, sol - 1 - dy, w, 1, hex(c));
                }
            }
            t.sprite_k(
                118,
                sol - 1,
                &["p.p.......", ".pPPPPPP..", "pPePPPPPPp", ".PPPPPPPpp", "..P..P...."],
                &[('p', 0xE05A8A), ('P', 0xF7A6C4), ('e', 0x1A1A1A)],
                2,
            );
        }
        // Nether Update : portail en obsidienne
        "1.16" => {
            let (x, y) = (114, sol - 22);
            t.rect(x, y, 16, 22, hex(0x1B1030));
            t.rect(x + 3, y + 3, 10, 19, hex(0x8B3CF0));
            for _ in 0..30 {
                t.pose(x + 3 + (rng.float() * 10.0) as i32, y + 3 + (rng.float() * 19.0) as i32, hex(0xB070FF));
            }
            for _ in 0..8 {
                t.pose(x + 4 + (rng.float() * 8.0) as i32, y + 5 + (rng.float() * 16.0) as i32, hex(0x5A1AB0));
            }
            // champignon carmin
            t.rect(136, sol - 8, 2, 8, hex(0x6A2A3A));
            t.rect(132, sol - 12, 10, 4, hex(0xC0283A));
        }
        // Buzzy Bees : un grand arbre, son nid et des abeilles
        "1.15" => {
            t.rect(122, sol - 18, 3, 18, hex(0x6B4A2A));
            t.disque(123, 46, 10, hex(0x4E9A3A));
            t.disque(116, 50, 6, hex(0x5FB043));
            t.disque(131, 50, 6, hex(0x5FB043));
            t.rect(126, 58, 6, 6, hex(0xE8B13A));
            t.rect(126, 60, 6, 1, hex(0x7A5A1A));
            t.rect(128, 62, 2, 2, hex(0x3A2A10));
            for (bx, by) in [(108, 52), (140, 44), (112, 62), (144, 60)] {
                t.sprite(bx, by, &["ww..", "ykyk", "ykyk"], &[('y', 0xF2C230), ('k', 0x2A1A0A), ('w', 0xE8F4FF)]);
            }
        }
        // Village & Pillage : maison de village
        "1.14" => {
            let (x, y) = (110, sol - 16);
            t.rect(x, y + 12, 22, 4, hex(0x7A7A7A));
            t.rect(x, y + 2, 22, 10, hex(0xB8945A));
            t.rect(x, y + 2, 2, 14, hex(0x6B4A2A));
            t.rect(x + 20, y + 2, 2, 14, hex(0x6B4A2A));
            for i in 0..8 {
                t.rect(x - 2 + i, y + 2 - i, 26 - 2 * i, 1, hex(0x8A5A2A));
            }
            t.rect(x + 9, y + 8, 4, 8, hex(0x5A3A1A));
            t.rect(x + 3, y + 5, 4, 3, hex(0xA8D8F0));
            t.rect(x + 15, y + 5, 4, 3, hex(0xA8D8F0));
            // cloche
            t.rect(138, sol - 9, 1, 9, hex(0x5A3A1A));
            t.rect(136, sol - 8, 5, 4, hex(0xF2C230));
        }
        // Update Aquatic : un dauphin qui saute, des coraux
        "1.13" => {
            t.sprite_k(
                112,
                54,
                &["......dd....", "..dddddddd..", ".dDDDDDDDDdd", "ddDeDDDDDDd.", ".llllllldd..", "...ll...d..."],
                &[('d', 0x6F88A8), ('D', 0x8FA8C0), ('l', 0xC8D8E8), ('e', 0x1A1A2A)],
                2,
            );
            for (x, c) in [(100, 0xFF6F91), (106, 0xF2D84A), (140, 0x4A7AFF), (146, 0xFF6F91), (152, 0xB46CFF)] {
                t.rect(x + 1, sol - 7, 1, 7, hex(c));
                t.rect(x - 1, sol - 5, 1, 3, hex(c));
                t.rect(x + 3, sol - 4, 1, 3, hex(c));
            }
        }
        // World of Color : béton de toutes les couleurs, et un perroquet
        "1.12" => {
            let couleurs = [0xE03A3A, 0xF28A2A, 0xF2D84A, 0x5FBF3F, 0x2FA8C8, 0x3A5ACF, 0x8A3ACF, 0xD94FA8];
            let mut k = 0;
            for (rang, n) in [(0, 6), (1, 4), (2, 2)] {
                for i in 0..n {
                    t.rect(106 + rang * 6 + i * 6, sol - 6 - rang * 6, 6, 6, hex(couleurs[k % 8]));
                    k += 1;
                }
            }
            t.sprite(140, sol - 1, &[".rr.", "rrer", ".rr.", ".bb.", ".yb.", "..y."], &[('r', 0xE03A3A), ('e', 0x1A1A1A), ('b', 0x3A5ACF), ('y', 0xF2D84A)]);
        }
        // Exploration Update : le manoir des bois
        "1.11" => {
            let (x, y) = (104, sol - 20);
            t.rect(x, y + 6, 40, 14, hex(0x4A3424));
            t.rect(x + 12, y, 16, 6, hex(0x4A3424));
            t.rect(x - 1, y + 5, 42, 2, hex(0x2E2018));
            t.rect(x + 11, y - 1, 18, 2, hex(0x2E2018));
            t.rect(x, y + 18, 40, 2, hex(0x7A7A7A));
            for i in 0..5 {
                t.rect(x + 3 + i * 8, y + 9, 3, 4, hex(0xE8D08A));
            }
            t.rect(x + 18, y + 2, 4, 3, hex(0xE8D08A));
            t.rect(x + 18, y + 14, 4, 4, hex(0x2E2018));
        }
        // Frostburn Update : un ours polaire sur la neige
        "1.10" => {
            t.sprite_k(
                110,
                sol - 1,
                &["..........WW..", "..WWWWWWWWWWW.", ".WWWWWWWWWWWeW", "WWWWWWWWWWWWWn", "WWWWWWWWWWWW..", ".WW.WW...WW.WW", ".WW.WW...WW.WW"],
                &[('W', 0xF0F4F8), ('e', 0x1A1A1A), ('n', 0x2A2A2A)],
                2,
            );
            for (x, h) in [(140, 14), (144, 9), (104, 10)] {
                for dy in 0..h {
                    let w = if dy > h - 4 { 1 } else { 3 };
                    t.rect(x + (3 - w) / 2, sol - 1 - dy, w, 1, hex(0xA8D8F0));
                }
            }
        }
        // Combat Update : tour de l'End et un shulker
        "1.9" => {
            let violet = hex(0xA97BA9);
            t.rect(122, sol - 30, 8, 30, violet);
            t.rect(118, sol - 34, 16, 6, hex(0xC9A0C9));
            t.rect(116, sol - 18, 20, 4, hex(0xC9A0C9));
            for y in (sol - 26..sol).step_by(6) {
                t.rect(125, y, 2, 3, hex(0x2A1A3A));
            }
            t.rect(125, sol - 40, 2, 6, hex(0xF4F0FF));
            t.rect(108, sol - 6, 6, 6, hex(0x8A5A9A));
            t.rect(109, sol - 3, 4, 2, hex(0xE8E4B0));
        }
        // Bountiful Update : monument océanique
        "1.8" => {
            for (i, (w, c)) in [(48, 0x3A7A70), (36, 0x4A8A80), (24, 0x5AA89A), (12, 0x6FC0B0)].iter().enumerate() {
                t.rect(126 - w / 2, sol - 5 - i as i32 * 5, *w, 5, hex(*c));
            }
            for x in [108, 120, 132, 144] {
                t.rect(x, sol - 4, 2, 2, hex(0xD8F0E8));
            }
            t.rect(125, sol - 18, 2, 2, hex(0xD8F0E8));
        }
        // Horizon : Ferris, le crabe de Rust
        "horizon" => {
            t.sprite_k(
                113,
                sol - 1,
                &["o..........o", "oo........oo", ".oOOOOOOOOo.", "OOOeOOOOeOOO", ".OOOOOOOOOO.", "o.o......o.o"],
                &[('o', 0xB04A2A), ('O', 0xF0795A), ('e', 0x1A1A1A)],
                2,
            );
        }
        _ => {}
    }
}

thread_local! {
    static ILLUSTRATIONS: RefCell<HashMap<String, Image>> = RefCell::new(HashMap::new());
}

/// Illustration d'une mise à jour (dessinée une fois, puis gardée).
pub fn illustration(cle: &str) -> Image {
    ILLUSTRATIONS.with(|c| c.borrow_mut().entry(cle.to_string()).or_insert_with(|| Image::from_rgba8(dessiner(cle))).clone())
}

#[cfg(test)]
mod tests {
    #[test]
    fn groupes() {
        assert_eq!(super::cle_groupe("1.21.4", 0), "1.21");
        assert_eq!(super::cle_groupe("26.1.2", 0), "26.x");
        assert_eq!(super::cle_groupe("24w14a", 1), "snapshots");
        assert!(super::compatible("1.21.4", 1));
        assert!(!super::compatible("1.12.2", 3));
        assert!(!super::compatible("24w14a", 1));
    }
}

/// Planche de toutes les illustrations (×3), dans MISTY_PLANCHE : `cargo test planche_illustrations -- --ignored`.
#[cfg(test)]
#[test]
#[ignore]
fn planche_illustrations() {
    let cles = ["1.21", "1.20", "1.19", "1.18", "1.17", "1.16", "1.15", "1.14", "1.13", "1.12", "1.11", "1.10", "1.9", "1.8", "horizon", "26.x"];
    let (k, col) = (3u32, 4u32);
    let mut planche = image::RgbaImage::new(L * k * col, H * k * cles.len().div_ceil(col as usize) as u32);
    for (i, cle) in cles.iter().enumerate() {
        let t = dessiner(cle);
        let (ox, oy) = ((i as u32 % col) * L * k, (i as u32 / col) * H * k);
        for (j, p) in t.as_slice().iter().enumerate() {
            let (x, y) = (j as u32 % L, j as u32 / L);
            for dy in 0..k {
                for dx in 0..k {
                    planche.put_pixel(ox + x * k + dx, oy + y * k + dy, image::Rgba([p.r, p.g, p.b, 255]));
                }
            }
        }
    }
    planche.save(std::env::var("MISTY_PLANCHE").unwrap_or_else(|_| "planche.png".into())).unwrap();
}
