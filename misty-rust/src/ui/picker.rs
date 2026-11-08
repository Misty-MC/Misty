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
    let mut img = vec![[0u8; 3]; (L * H) as usize];
    let mut pose = |x: i32, y: i32, c: [u8; 3]| {
        if (0..L as i32).contains(&x) && (0..H as i32).contains(&y) {
            img[(y as u32 * L + x as u32) as usize] = c;
        }
    };
    // ciel, par bandes de 6 pixels (effet pixel art)
    for y in 0..H as i32 {
        let t = ((y / 6) * 6) as f32 / H as f32;
        for x in 0..L as i32 {
            pose(x, y, melange(pal.ciel[0], pal.ciel[1], t));
        }
    }
    // étoiles la nuit
    if pal.nuit {
        for _ in 0..40 {
            let (x, y) = ((rng.float() * L as f32) as i32, (rng.float() * H as f32 * 0.5) as i32);
            pose(x, y, melange(pal.ciel[1], [255, 255, 255], 0.75));
        }
    }
    // astre : soleil carré le jour, lune la nuit
    let (ax, ay, r) = ((L as f32 * (0.62 + rng.float() * 0.25)) as i32, 14 + (rng.float() * 8.0) as i32, 6);
    let astre = if pal.nuit { [234, 241, 255] } else { [255, 244, 196] };
    for dy in -r..=r {
        for dx in -r..=r {
            if dx * dx + dy * dy <= r * r {
                pose(ax + dx, ay + dy, astre);
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
                pose(x, y, pal.cretes[i]);
            }
        }
    }
    // sol en blocs de 8 : dessus et terre, avec un peu de grain
    let sol = 74;
    for y in sol..H as i32 {
        for x in 0..L as i32 {
            let bloc = ((x / 4 + y / 4) % 3) as f32 * 0.04;
            let base = if y < sol + 4 { pal.sol[0] } else { pal.sol[1] };
            pose(x, y, melange(base, [0, 0, 0], bloc));
        }
    }
    // quelques arbres (troncs et feuillage) sur le sol
    let mut x = 6 + (rng.float() * 10.0) as i32;
    while x < L as i32 - 8 {
        let tronc = melange(pal.sol[1], [0, 0, 0], 0.3);
        let feuilles = melange(pal.cretes[1], pal.sol[0], 0.35);
        for y in sol - 8..sol {
            pose(x, y, tronc);
            pose(x + 1, y, tronc);
        }
        for dy in 0..6 {
            for dx in -3..=4 {
                if (dx as i32).abs() + dy / 2 < 5 {
                    pose(x + dx, sol - 14 + dy, feuilles);
                }
            }
        }
        x += 20 + (rng.float() * 30.0) as i32;
    }
    let mut tampon = SharedPixelBuffer::<Rgba8Pixel>::new(L, H);
    for (p, c) in tampon.make_mut_slice().iter_mut().zip(&img) {
        *p = Rgba8Pixel { r: c[0], g: c[1], b: c[2], a: 255 };
    }
    tampon
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
